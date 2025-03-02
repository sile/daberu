use std::{io::Read, path::Path};

use orfail::OrFail;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn gist_filename(self, i: usize) -> String {
        let name = match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        };
        format!("{:03}_{}.md", i, name)
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
        self.messages.push(Message {
            role: Role::User,
            content: input,
        });
        Ok(())
    }

    pub fn set_system_message_if_empty(&mut self, system: &str) {
        if self.messages.is_empty() {
            self.messages.push(Message {
                role: Role::System,
                content: system.to_owned(),
            });
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
