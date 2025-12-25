use std::io::{BufRead, Write};

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
    skill_ids: Vec<SkillId>,
}

impl Claude {
    pub fn new(command: &Command, model: String) -> orfail::Result<Self> {
        Ok(Self {
            api_key: command.anthropic_api_key.clone().or_fail()?,
            model,
            skill_ids: command.skill_ids.clone(),
        })
    }

    pub fn run(&self, log: &MessageLog) -> orfail::Result<Message> {
        let (log, system_message) = log.strip_system_message();
        let stream = self.skill_ids.is_empty(); // I do not know why, but this is needed
        let request = nojson::json(|f| {
            f.object(|f| {
                f.member("model", &self.model)?;
                f.member("stream", stream)?;
                f.member("max_tokens", MAX_TOKENS)?;
                f.member("messages", &log.messages)?;
                if let Some(system_message) = &system_message {
                    f.member("system", system_message)?;
                }
                if self.skill_ids.is_empty() {
                    return Ok(());
                }

                // Add skill related fields (container, tools) if skill_ids is not empty
                f.member(
                    "container",
                    nojson::object(|f| {
                        // TODO: id handling to continue conversation

                        f.member(
                            "skills",
                            nojson::array(|f| {
                                for skill_id in &self.skill_ids {
                                    f.element(nojson::object(|f| {
                                        f.member("type", skill_id.source())?;
                                        f.member("skill_id", &skill_id.0)?;
                                        f.member("version", "latest")
                                    }))?;
                                }
                                Ok(())
                            }),
                        )?;
                        Ok(())
                    }),
                )?;
                f.member(
                    "tools",
                    [nojson::object(|f| {
                        f.member("type", "code_execution_20250825")?;
                        f.member("name", "code_execution")
                    })],
                )?;
                Ok(())
            })
        });

        let mut request_builder = crate::curl::CurlRequest::new(API_END_POINT)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION);

        // Add skill related headers if skill_ids is not empty
        if !self.skill_ids.is_empty() {
            request_builder = request_builder.header(
                "anthropic-beta",
                "code-execution-2025-08-25,skills-2025-10-02",
            );
        }

        let response = request_builder.post(request)?;

        let reader = response.check_success()?;
        let reply = if stream {
            self.handle_stream_response(reader).or_fail()?
        } else {
            self.handle_response(reader).or_fail()?
        };

        Ok(reply)
    }

    fn handle_response<R: BufRead>(&self, reader: R) -> orfail::Result<Message> {
        let mut text = String::new();
        let mut reader = reader;
        reader.read_to_string(&mut text).or_fail()?;

        let nojson::Json(response) = text
            .parse::<nojson::Json<ApiResponse>>()
            .or_fail_with(|e| format!("failed to parse response: {e}"))?;

        let content = response
            .content
            .into_iter()
            .filter_map(|block| match block {
                ContentBlock::Text(text) => Some(text),
                ContentBlock::ServerToolUse { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(Message {
            role: Role::Assistant,
            content,
            model: Some(self.model.clone()),
        })
    }

    fn handle_stream_response<R: BufRead>(&self, reader: R) -> orfail::Result<Message> {
        let mut content = String::new();
        for line in reader.lines() {
            let line = line.or_fail()?;
            dbg!(&line);
            if line.is_empty() {
                continue;
            }
            if !line.starts_with("data: ") {
                continue;
            }
            if line == "data: [DONE]" {
                break;
            }

            let nojson::Json(data) = line[(("data: ").len())..]
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
                Data::ContentBlockStart { content_block } => match content_block {
                    ContentBlock::Text(text) => {
                        content.push_str(&text);
                        print!("{}", text);
                        std::io::stdout().flush().or_fail()?;
                    }
                    ContentBlock::ServerToolUse { id, name, input } => {
                        eprintln!("Server tool use: id={}, name={}, input={}", id, name, input);
                    }
                },
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
struct ApiResponse {
    content: Vec<ContentBlock>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ApiResponse {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, nojson::JsonParseError> {
        let content = value.to_member("content")?.required()?;
        Ok(Self {
            content: content.try_into()?,
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

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Data {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, nojson::JsonParseError> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "message_start" => Ok(Self::MessageStart {
                stop_reason: value.to_member("stop_reason")?.try_into()?,
            }),
            "message_delta" => Ok(Self::MessageDelta {
                stop_reason: value.to_member("stop_reason")?.try_into()?,
            }),
            "message_stop" => Ok(Self::MessageStop),
            "content_block_start" => {
                let content_block = value.to_member("content_block")?.required()?;
                let block_type = content_block.to_member("type")?.required()?;

                match block_type.to_unquoted_string_str()?.as_ref() {
                    "text" => {
                        let text = content_block.to_member("text")?.required()?;
                        Ok(Self::ContentBlockStart {
                            content_block: ContentBlock::Text(text.try_into()?),
                        })
                    }
                    "server_tool_use" => {
                        let id = content_block.to_member("id")?.required()?;
                        let name = content_block.to_member("name")?.required()?;
                        let input = content_block.to_member("input")?.required()?;

                        Ok(Self::ContentBlockStart {
                            content_block: ContentBlock::ServerToolUse {
                                id: id.try_into()?,
                                name: name.try_into()?,
                                input: input.extract().into_owned(),
                            },
                        })
                    }
                    ty => Err(content_block.invalid(format!("unknown content block type: {ty}"))),
                }
            }
            "content_block_delta" => {
                let delta = value.to_member("delta")?.required()?;
                let text = delta.to_member("text")?.required()?;
                Ok(Self::ContentBlockDelta {
                    delta: Delta {
                        text: text.try_into()?,
                    },
                })
            }
            "content_block_stop" => Ok(Self::ContentBlockStop),
            "ping" => Ok(Self::Ping),
            "error" => {
                let error = value.to_member("error")?.required()?;
                Ok(Self::Error {
                    error: error.to_string(),
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
enum ContentBlock {
    Text(String),
    ServerToolUse {
        id: String,
        name: String,
        input: nojson::RawJsonOwned,
    },
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ContentBlock {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, nojson::JsonParseError> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "text" => {
                let text = value.to_member("text")?.required()?;
                Ok(Self::Text(text.try_into()?))
            }
            "server_tool_use" => {
                let id = value.to_member("id")?.required()?;
                let name = value.to_member("name")?.required()?;
                let input = value.to_member("input")?.required()?;
                Ok(Self::ServerToolUse {
                    id: id.try_into()?,
                    name: name.try_into()?,
                    input: input.extract().into_owned(),
                })
            }
            ty => Err(value.invalid(format!("unknown content block type: {ty}"))),
        }
    }
}

#[derive(Debug)]
struct Delta {
    text: String,
}

#[derive(Debug, Clone)]
pub struct SkillId(String);

impl SkillId {
    pub fn source(&self) -> &'static str {
        if self.0.starts_with("skill_") {
            "custom"
        } else {
            "anthropic"
        }
    }
}

impl std::str::FromStr for SkillId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_owned()))
    }
}
