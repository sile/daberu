use orfail::{Failure, OrFail};

use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use crate::{
    command::Command,
    message::{Message, MessageLog, Role},
};

#[derive(Debug)]
pub struct ChatGpt {
    api_key: String,
    model: String,
}

impl ChatGpt {
    pub fn new(command: &Command, model: String) -> orfail::Result<Self> {
        Ok(Self {
            api_key: command.openai_api_key.clone().or_fail()?,
            model,
        })
    }

    pub fn run(&self, log: &MessageLog) -> orfail::Result<Message> {
        let request = nojson::json(|f| {
            f.object(|f| {
                f.member("model", &self.model)?;
                f.member("stream", true)?;
                f.member("messages", &log.messages)?;
                Ok(())
            })
        });

        let mut cmd = std::process::Command::new("curl");
        cmd.arg("https://api.openai.com/v1/chat/completions")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", self.api_key))
            .arg("-d")
            .arg("@-") // Read data from stdin
            .arg("--silent")
            .arg("--show-error")
            .arg("--no-buffer")
            .arg("--include");

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .or_fail()?;

        let stdin = child.stdin.take().or_fail()?;
        write!(BufWriter::new(stdin), "{}", request).or_fail()?;

        let stdout = child.stdout.take().or_fail()?;
        let reply = self.handle_stream_response(stdout).or_fail()?;

        let status = child.wait().or_fail()?;
        status
            .success()
            .or_fail_with(|()| format!("curl command failed with status: {}", status))?;

        Ok(reply)
    }

    fn handle_stream_response<R: Read>(&self, reader: R) -> orfail::Result<Message> {
        let mut reader = BufReader::new(reader);
        let mut first_line = String::new();
        reader.read_line(&mut first_line).or_fail()?;

        // Parse HTTP status line (e.g., "HTTP/1.1 200 OK")
        first_line.starts_with("HTTP/").or_fail()?;

        // Skip remaining headers until we find the empty line
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).or_fail()?;
            if line.trim().is_empty() {
                break;
            }
        }

        let parts: Vec<&str> = first_line.split_whitespace().collect();
        (parts.len() >= 2).or_fail()?;
        let status_code: u16 = parts[1]
            .parse::<u16>()
            .or_fail_with(|_| format!("Invalid HTTP status code: {}", parts[1]))?;

        if status_code != 200 {
            // Read response body for error details
            let mut error_body = String::new();
            reader.read_to_string(&mut error_body).or_fail()?;

            return Err(Failure::new(format!(
                "HTTP request failed with status {}: {}\n\nResponse body:\n{}",
                status_code,
                first_line.trim(),
                error_body.trim()
            )));
        }

        #[derive(Debug)]
        struct Data {
            choices: Vec<Choice>,
        }

        impl<'text> nojson::FromRawJsonValue<'text> for Data {
            fn from_raw_json_value(
                value: nojson::RawJsonValue<'text, '_>,
            ) -> Result<Self, nojson::JsonParseError> {
                let ([choices], []) = value.to_fixed_object(["choices"], [])?;
                let choices = choices
                    .to_array()?
                    .map(|choice| {
                        let ([delta], [finish_reason]) =
                            choice.to_fixed_object(["delta"], ["finish_reason"])?;
                        let ([], [content]) = delta.to_fixed_object([], ["content"])?;
                        Ok(Choice {
                            delta: Delta {
                                content: content.map(|c| c.try_to()).transpose()?,
                            },
                            finish_reason: finish_reason
                                .and_then(|x| (!x.kind().is_null()).then(|| x.try_to()))
                                .transpose()?,
                        })
                    })
                    .collect::<Result<_, _>>()?;
                Ok(Self { choices })
            }
        }

        #[derive(Debug)]
        struct Choice {
            delta: Delta,
            finish_reason: Option<FinishReason>,
        }

        #[derive(Debug)]
        struct Delta {
            content: Option<String>,
        }

        let mut content = String::new();
        let reader = BufReader::new(reader);
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
            (!data.choices.is_empty()).or_fail()?;
            if let Some(reason) = data.choices[0].finish_reason {
                reason.check().or_fail()?;
            }

            if let Some(c) = &data.choices[0].delta.content {
                content.push_str(c);
                print!("{c}");
                std::io::stdout().flush().or_fail()?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

impl FinishReason {
    pub fn check(self) -> orfail::Result<()> {
        match self {
            Self::Stop => Ok(()),
            Self::Length => Err(Failure::new(
                "Incomplete model output due to max_tokens parameter or token limit",
            )),
            Self::ContentFilter => Err(Failure::new(
                "Omitted content due to a flag from our content filters",
            )),
        }
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for FinishReason {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        match value.to_unquoted_string_str()?.as_ref() {
            "stop" => Ok(Self::Stop),
            "length" => Ok(Self::Length),
            "content_filter" => Ok(Self::ContentFilter),
            reason => Err(nojson::JsonParseError::invalid_value(
                value,
                format!("unexpected finish reason: {reason}"),
            )),
        }
    }
}
