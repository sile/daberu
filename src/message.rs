use std::{io::Read, path::Path};

use orfail::OrFail;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn gist_filename(self, i: usize, model: Option<&String>) -> String {
        let name = match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        if let Some(model) = model {
            format!("{:03}_{}_{}.md", i, name, model)
        } else {
            format!("{:03}_{}.md", i, name)
        }
    }

    pub fn from_gist_filename(filename: &str, i: usize) -> orfail::Result<(Self, Option<String>)> {
        let prefix = format!("{:03}_", i);
        (filename.starts_with(&prefix) && filename.ends_with(".md"))
            .or_fail_with(|()| format!("unexpected gist filename: {filename}"))?;
        let mut tokens = filename[4..filename.len() - 3].splitn(2, '_');
        match tokens.next().expect("infallible") {
            "system" => Ok((Self::System, None)),
            "user" => Ok((Self::User, None)),
            "assistant" => {
                let model = tokens.next().map(|model| model.to_owned());
                Ok((Self::Assistant, model))
            }
            _ => Err(orfail::Failure::new(format!(
                "unexpected gist filename: {filename}"
            ))),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MessageLog {
    pub messages: Vec<Message>,
}

impl MessageLog {
    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let file = std::fs::File::open(&path).or_fail_with(|e| {
            format!("failed to open log file {}: {e}", path.as_ref().display())
        })?;
        let messages = serde_json::from_reader(file).or_fail_with(|e| {
            format!("failed to load log file {}: {e}", path.as_ref().display())
        })?;
        Ok(Self { messages })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> orfail::Result<()> {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .or_fail_with(|e| {
                format!("failed to create log file {}: {e}", path.as_ref().display())
            })?;
        serde_json::to_writer(file, &self.messages).or_fail_with(|e| {
            format!("failed to save log file {}: {e}", path.as_ref().display())
        })?;
        Ok(())
    }

    pub fn read_input(&mut self) -> orfail::Result<()> {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).or_fail()?;
        (!input.is_empty()).or_fail_with(|()| "empty input message".to_owned())?;
        self.messages.push(Message {
            role: Role::User,
            content: input,
            model: None,
        });
        Ok(())
    }

    pub fn set_system_message_if_empty(&mut self, system: &str) {
        if self.messages.is_empty() {
            self.messages.push(Message {
                role: Role::System,
                content: system.to_owned(),
                model: None,
            });
        }
    }

    pub fn strip_model_name(&self) -> Self {
        Self {
            messages: self
                .messages
                .iter()
                .cloned()
                .map(|mut m| {
                    m.model = None;
                    m
                })
                .collect(),
        }
    }

    pub fn strip_system_message(&self) -> (Self, Option<String>) {
        if matches!(
            self.messages.first(),
            Some(Message {
                role: Role::System,
                ..
            })
        ) {
            (
                Self {
                    messages: self.messages[1..].to_vec(),
                },
                Some(self.messages[0].content.clone()),
            )
        } else {
            (self.clone(), None)
        }
    }
}
