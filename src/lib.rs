use orfail::{Failure, OrFail};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
};

#[derive(Debug, clap::Args)]
pub struct ChatGpt {
    /// OpenAI API key.
    #[arg(
        long,
        value_name = "OPENAI_API_KEY",
        env = "OPENAI_API_KEY",
        hide_env_values = true
    )]
    api_key: String,

    /// Log file path to save the conversation history. If the file already exists, the history will be considered in the next conversation.
    #[arg(long, value_name = "LOG_FILE_PATH")]
    log: Option<PathBuf>,

    /// ChatGPT model name.
    #[arg(long, env = "CHATGPT_MODEL", default_value = "gpt-4o")]
    model: String,

    /// If specified, the system role message will be added to the beginning of the conversation.
    #[arg(long, value_name = "SYSTEM_MESSAGE", env = "CHATGPT_SYSTEM_MESSAGE")]
    system: Option<String>,

    /// If specified, HTTP request and response body JSONs are printed to stderr.
    #[arg(long)]
    verbose: bool,

    #[arg(short, long)]
    echo_input: bool,
}

impl ChatGpt {
    pub fn call(&self) -> orfail::Result<()> {
        let request = RequestBody::new(self).or_fail()?;
        if self.verbose {
            eprintln!("{}", serde_json::to_string_pretty(&request).or_fail()?);
        }

        let response = ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_json(&request)
            .or_fail()?;

        if self.echo_input {
            println!("Input");
            println!("=====");
            println!();
            println!("```console");
            println!(
                "$ echo -e {:?} | daberu {}",
                request.messages.last().or_fail()?.content.trim(),
                std::env::args().skip(1).collect::<Vec<_>>().join(" ")
            );
            println!("```");
            println!();
            println!("Output");
            println!("======");
            println!();
        }

        let reply = if self.verbose {
            self.handle_response(response).or_fail()?
        } else {
            self.handle_stream_response(response).or_fail()?
        };

        if let Some(log) = &self.log {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(log)
                .or_fail()?;
            let mut log = request.messages;
            log.push(reply);
            serde_json::to_writer(file, &log).or_fail()?;
        }

        Ok(())
    }

    fn handle_stream_response(&self, response: ureq::Response) -> orfail::Result<Message> {
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
        let reader = BufReader::new(response.into_reader());
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

    fn handle_response(&self, response: ureq::Response) -> orfail::Result<Message> {
        #[derive(Debug, serde::Deserialize)]
        struct ResponseBody {
            choices: Vec<Choice>,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Choice {
            message: Message,
            finish_reason: FinishReason,
        }

        let response_json: serde_json::Value = response.into_json().or_fail()?;

        if self.verbose {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&response_json).or_fail()?
            );
        }

        let response: ResponseBody = serde_json::from_value(response_json).or_fail()?;
        let choice = response.choices.into_iter().next().or_fail()?;
        choice.finish_reason.check().or_fail()?;
        println!("{}", choice.message.content);
        Ok(choice.message)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct RequestBody {
    model: String,
    stream: bool,
    messages: Vec<Message>,
}

impl RequestBody {
    pub fn new(chatgpt: &ChatGpt) -> orfail::Result<Self> {
        let mut messages = Vec::new();
        if let Some(log) = &chatgpt.log {
            if let Ok(file) = std::fs::File::open(log) {
                messages = serde_json::from_reader(file).or_fail()?;
            }
        }

        if messages.is_empty() {
            if let Some(system) = &chatgpt.system {
                messages.push(Message {
                    role: Role::System,
                    content: system.clone(),
                });
            }
        }

        let mut message = String::new();
        std::io::stdin().read_to_string(&mut message).or_fail()?;
        messages.push(Message {
            role: Role::User,
            content: message.clone(),
        });
        Ok(Self {
            model: chatgpt.model.clone(),
            stream: !chatgpt.verbose,
            messages,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
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
