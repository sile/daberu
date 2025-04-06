use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use orfail::OrFail;

#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub model: Option<String>,
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
            Ok(())
        })
    }
}

impl<'text> nojson::FromRawJsonValue<'text> for Message {
    fn from_raw_json_value(
        value: nojson::RawJsonValue<'text, '_>,
    ) -> Result<Self, nojson::JsonParseError> {
        let ([role, content], [model]) = value.to_fixed_object(["role", "content"], ["model"])?;
        Ok(Self {
            role: match role.as_raw_str() {
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
            content: content.try_to()?,
            model: model.map(|m| m.try_to()).transpose()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn read_input(&mut self, resources: &[PathBuf]) -> orfail::Result<()> {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input).or_fail()?;
        (!input.is_empty()).or_fail_with(|()| "empty input message".to_owned())?;

        let resources = resources
            .iter()
            .map(|path| {
                let content = std::fs::read_to_string(path).or_fail_with(|e| {
                    format!("failed to read resource file {}: {e}", path.display())
                })?;
                Ok(nojson::json(move |f| {
                    f.object(|f| {
                        f.member("type", "file")?;
                        // TODO: use nojson's implementation when it becomes available
                        f.member("path", path.display().to_string())?;
                        f.member("content", &content)?;
                        Ok(())
                    })
                }))
            })
            .collect::<orfail::Result<Vec<_>>>()?;
        if !resources.is_empty() {
            input.push_str(
                r#"

------

# Resources

Please consider the following JSON array as the resources:
"#,
            );
            input.push_str(&format!("```json\n{}\n```", nojson::Json(&resources)));
        }

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
