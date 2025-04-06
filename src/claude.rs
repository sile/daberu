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
    pub fn new(command: &Command, model: String) -> orfail::Result<Self> {
        Ok(Self {
            api_key: command.anthropic_api_key.clone().or_fail()?,
            model,
        })
    }

    pub fn run(&self, log: &MessageLog) -> orfail::Result<Message> {
        let (log, system_message) = log.strip_system_message();
        let request = nojson::json(|f| {
            f.object(|f| {
                f.member("model", &self.model)?;
                f.member("stream", true)?;
                f.member("max_tokens", MAX_TOKENS)?;
                f.member("messages", &log.messages)?;
                if let Some(system_message) = &system_message {
                    f.member("system", system_message)?;
                }
                Ok(())
            })
        });
        let response = ureq::post(API_END_POINT)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .send(request.to_string())
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

            let nojson::Json(data) = line["data: ".len()..]
                .parse::<nojson::Json<Data>>()
                .or_fail_with(|e| format!("failed to parse line: {line} ({e})"))?;
            match data {
                Data::MessageStart { stop_reason } | Data::MessageDelta { stop_reason } => {
                    if let Some(reason) = stop_reason {
                        (reason == "end_turn").or_fail_with(|()| format!("API error: {reason}"))?;
                    }
                }
                Data::MessageStop => {}
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
                Data::ContentBlockStop => {}
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

#[derive(Debug)]
enum Data {
    MessageStart { stop_reason: Option<String> },
    MessageDelta { stop_reason: Option<String> },
    MessageStop,
    ContentBlockStart { content_block: ContentBlock },
    ContentBlockDelta { delta: Delta },
    ContentBlockStop,
    Ping,
    Error { error: String },
}

impl<'text> nojson::FromRawJsonValue<'text> for Data {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([ty], []) = value.to_fixed_object(["type"], [])?;
        match ty.as_raw_str() {
            "message_start" => {
                let ([], [stop_reason]) = value.to_fixed_object([], ["stop_reason"])?;
                Ok(Self::MessageStart {
                    stop_reason: stop_reason.map(|x| x.as_raw_str().to_owned()),
                })
            }
            "message_delta" => {
                let ([], [stop_reason]) = value.to_fixed_object([], ["stop_reason"])?;
                Ok(Self::MessageDelta {
                    stop_reason: stop_reason.map(|x| x.as_raw_str().to_owned()),
                })
            }
            "message_stop" => Ok(Self::MessageStop),
            "content_block_start" => {
                let ([content_block], []) = value.to_fixed_object(["content_block"], [])?;
                let ([text], []) = content_block.to_fixed_object(["text"], [])?;
                Ok(Self::ContentBlockStart {
                    content_block: ContentBlock {
                        text: text.as_raw_str().to_owned(),
                    },
                })
            }
            "content_block_delta" => {
                let ([delta], []) = value.to_fixed_object(["delta"], [])?;
                let ([text], []) = delta.to_fixed_object(["text"], [])?;
                Ok(Self::ContentBlockDelta {
                    delta: Delta {
                        text: text.as_raw_str().to_owned(),
                    },
                })
            }
            "content_block_stop" => Ok(Self::ContentBlockStop),
            "ping" => Ok(Self::Ping),
            "error" => {
                let ([error], []) = value.to_fixed_object(["error"], [])?;
                Ok(Self::Error {
                    error: error.as_raw_str().to_owned(),
                })
            }
            ty => Err(nojson::JsonParseError::invalid_value(
                value,
                format!("unknown message type: {ty}"),
            )),
        }
    }
}

#[derive(Debug)]
struct ContentBlock {
    text: String,
}

#[derive(Debug)]
struct Delta {
    text: String,
}
