use std::io::{BufRead, BufReader, Write};

use orfail::OrFail;

use crate::{
    command::Command,
    message::{Message, MessageLog, Role},
};

const API_END_POINT: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 10_000;

#[derive(Debug)]
pub struct Claude {
    api_key: String,
    model: String,
}

impl Claude {
    pub fn new(command: &Command) -> orfail::Result<Self> {
        Ok(Self {
            api_key: command.anthropic_api_key.clone().or_fail()?,
            model: command.model.clone(),
        })
    }

    pub fn run(&self, log: &MessageLog) -> orfail::Result<Message> {
        let (log, system_message) = log.strip_system_message();
        let mut request = serde_json::json!({
            "model": self.model,
            "stream": true,
            "max_tokens": MAX_TOKENS,
            "messages": log.messages,
        });
        if let Some(system_message) = system_message {
            request
                .as_object_mut()
                .or_fail()?
                .insert("system".to_owned(), system_message.into());
        }

        let response = ureq::post(API_END_POINT)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .send_json(&request)
            .or_fail()?;
        let reply = self.handle_stream_response(response).or_fail()?;
        Ok(reply)
    }

    fn handle_stream_response(
        &self,
        response: ureq::http::response::Response<ureq::Body>,
    ) -> orfail::Result<Message> {
        let mut content = String::new();
        let reader = BufReader::new(response.into_body().into_reader());
        for line in reader.lines() {
            let line = line.or_fail()?;
            if line.is_empty() {
                continue;
            }
            if !line.starts_with("data: ") {
                continue;
            }
            if line == "data: [DONE]" {
                break;
            }

            let data: Data = serde_json::from_str(&line["data: ".len()..])
                .or_fail_with(|e| format!("failed to parse line: {line} ({e})"))?;
            match data {
                Data::MessageStart { stop_reason } | Data::MessageDelta { stop_reason } => {
                    if let Some(reason) = stop_reason {
                        (reason == "end_turn").or_fail_with(|()| format!("API error: {reason}"))?;
                    }
                }
                Data::MessageStop {} => {}
                Data::Ping => {}
                Data::ContentBlockStart { content_block } => {
                    content.push_str(&content_block.text);
                    print!("{}", content_block.text);
                    std::io::stdout().flush().or_fail()?;
                }
                Data::ContentBlockDelta { delta } => {
                    content.push_str(&delta.text);
                    print!("{}", delta.text);
                    std::io::stdout().flush().or_fail()?;
                }
                Data::ContentBlockStop {} => {}
                Data::Error { error } => {
                    return Err(orfail::Failure::new(format!(
                        "Claude API error: reason={error}"
                    )));
                }
            }
        }
        println!();

        Ok(Message {
            role: Role::Assistant,
            content,
            model: Some(self.model.clone()),
        })
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Data {
    MessageStart { stop_reason: Option<String> },
    MessageDelta { stop_reason: Option<String> },
    MessageStop {},
    ContentBlockStart { content_block: ContentBlock },
    ContentBlockDelta { delta: Delta },
    ContentBlockStop {},
    Ping,
    Error { error: serde_json::Value },
}

#[derive(Debug, serde::Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct Delta {
    text: String,
}
