use std::path::PathBuf;

use orfail::OrFail;

use crate::{chat_gpt::ChatGpt, claude::Claude, gist, message::MessageLog};

#[derive(Debug)]
pub struct Command {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub log: Option<PathBuf>,
    pub continue_from_log: bool,
    pub models: Vec<String>,
    pub system: Option<String>,
    pub gist: Option<String>,
    pub resources: Vec<PathBuf>,
}

impl Command {
    pub fn run(self) -> orfail::Result<()> {
        self.check_api_key().or_fail()?;

        let mut gist_offset = 0;
        let mut log = self
            .log
            .as_ref()
            .filter(|path| path.exists())
            .map(MessageLog::load)
            .transpose()
            .or_fail()?
            .unwrap_or_default();
        if !self.continue_from_log {
            log.messages.clear();
        }
        if let Some(id) = self.gist.as_ref().filter(|id| *id != "new") {
            log = gist::load(id).or_fail()?;
            gist_offset = log.messages.len();
        }
        if let Some(system) = &self.system {
            log.set_system_message_if_empty(system);
        }
        log.read_input(&self.resources).or_fail()?;

        for (i, model) in self.models.iter().enumerate() {
            let result = if model.starts_with("gpt") || model.starts_with("openai:") {
                let model = model.strip_prefix("openai:").unwrap_or(model).to_owned();
                let c = ChatGpt::new(&self, model).or_fail()?;
                let log = log.strip_model_name();
                c.run(&log).or_fail()
            } else if model.starts_with("claude") || model.starts_with("anthropic:") {
                let model = model.strip_prefix("anthropic:").unwrap_or(model).to_owned();
                let c = Claude::new(&self, model).or_fail()?;
                let log = log.strip_model_name();
                c.run(&log).or_fail()
            } else {
                unreachable!()
            };
            match result {
                Ok(output) => {
                    log.messages.push(output);
                    break;
                }
                Err(e) if i + 1 == self.models.len() => {
                    return Err(e);
                }
                Err(e) => {
                    eprintln!("[WARNING] {model}: {e}");
                }
            }
        }

        if let Some(path) = self.log {
            log.save(path).or_fail()?;
        }
        match self.gist.as_deref() {
            Some("new") => {
                eprintln!();
                gist::create(&log).or_fail()?;
            }
            Some(id) => {
                eprintln!();
                gist::update(id, &log, gist_offset).or_fail()?;
            }
            None => {}
        }

        Ok(())
    }

    fn check_api_key(&self) -> orfail::Result<()> {
        for model in &self.models {
            if model.starts_with("gpt") || model.starts_with("openai:") {
                self.openai_api_key
                    .is_some()
                    .or_fail_with(|()| "OpenAI API key is not specified".to_owned())?;
            } else if model.starts_with("claude") || model.starts_with("anthropic:") {
                self.anthropic_api_key
                    .is_some()
                    .or_fail_with(|()| "Anthropic API key is not specified".to_owned())?;
            } else {
                return Err(orfail::Failure::new(format!("unknown model: {model}")));
            }
        }
        Ok(())
    }
}
