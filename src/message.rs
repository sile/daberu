use std::{io::Write, path::Path};

use orfail::OrFail;

use crate::resource::Resource;

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub model: Option<String>,
    pub container_id: Option<String>,
    // TODO: files_ids: Vec<String>
}

impl nojson::DisplayJson for Message {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member(
                "role",
                match self.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                },
            )?;
            f.member("content", &self.content)?;
            if let Some(model) = &self.model {
                f.member("model", model)?;
            }
            if let Some(container_id) = &self.container_id {
                f.member("container_id", container_id)?;
            }
            Ok(())
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Message {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let role = value.to_member("role")?.required()?;
        let content = value.to_member("content")?.required()?;
        let model = value.to_member("model")?;
        let container_id = value.to_member("container_id")?;

        Ok(Self {
            role: match role.to_unquoted_string_str()?.as_ref() {
                "system" => Role::System,
                "user" => Role::User,
                "assistant" => Role::Assistant,
                role_str => {
                    return Err(nojson::JsonParseError::invalid_value(
                        role,
                        format!("unknown role: {role_str}"),
                    ));
                }
            },
            content: content.try_into()?,
            model: model.try_into()?,
            container_id: container_id.try_into()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Default, Clone)]
pub struct MessageLog {
    pub messages: Vec<Message>,
}

impl MessageLog {
    pub fn load<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        let text = std::fs::read_to_string(&path).or_fail_with(|e| {
            format!("failed to open log file {}: {e}", path.as_ref().display())
        })?;
        let nojson::Json(messages) = text.parse::<nojson::Json<_>>().or_fail_with(|e| {
            format!("failed to load log file {}: {e}", path.as_ref().display())
        })?;
        Ok(Self { messages })
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> orfail::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .or_fail_with(|e| {
                format!("failed to create log file {}: {e}", path.as_ref().display())
            })?;
        write!(file, "{}", nojson::Json(&self.messages)).or_fail_with(|e| {
            format!("failed to save log file {}: {e}", path.as_ref().display())
        })?;
        Ok(())
    }

    pub fn latest_container_id(&self) -> Option<&str> {
        self.messages
            .iter()
            .rev()
            .find_map(|m| m.container_id.as_ref().map(|c| c.as_str()))
    }

    pub fn read_input(&mut self, mut input: String, resources: &[Resource]) -> orfail::Result<()> {
        if !resources.is_empty() {
            input.push_str(
                r#"

------

# Resources

Please consider the following JSON array as the resources:
"#,
            );
            input.push_str(&format!("```json\n{}\n```", nojson::Json(resources)));
        }

        self.messages.push(Message {
            role: Role::User,
            content: input,
            model: None,
            container_id: None,
        });
        Ok(())
    }

    pub fn set_system_message_if_empty(&mut self, system: &str) {
        if self.messages.is_empty() {
            self.messages.push(Message {
                role: Role::System,
                content: system.to_owned(),
                model: None,
                container_id: None,
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
