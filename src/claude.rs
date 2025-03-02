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

    pub fn run(&self, output_header: &str, log: &MessageLog) -> orfail::Result<Message> {
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
        print!("{output_header}");

        let reply = self.handle_stream_response(response).or_fail()?;
        Ok(reply)
    }

    fn handle_stream_response(
        &self,
        response: ureq::http::response::Response<ureq::Body>,
    ) -> orfail::Result<Message> {
        #[derive(Debug, serde::Deserialize)]
        struct Data {
            choices: Vec<Choice>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Choice {
            delta: Delta,
            finish_reason: Option<FinishReason>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Delta {
            #[serde(default)]
            content: String,
        }

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
            (!data.choices.is_empty()).or_fail()?;
            if let Some(reason) = data.choices[0].finish_reason {
                reason.check().or_fail()?;
            }

            content.push_str(&data.choices[0].delta.content);
            print!("{}", data.choices[0].delta.content);
            std::io::stdout().flush().or_fail()?;
        }
        println!();

        Ok(Message {
            role: Role::Assistant,
            content,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

impl FinishReason {
    pub fn check(self) -> orfail::Result<()> {
        match self {
            Self::Stop => Ok(()),
            Self::Length => Err(orfail::Failure::new(
                "Incomplete model output due to max_tokens parameter or token limit",
            )),
            Self::ContentFilter => Err(orfail::Failure::new(
                "Omitted content due to a flag from our content filters",
            )),
        }
    }
}
