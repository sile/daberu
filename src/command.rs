use std::path::PathBuf;

use orfail::OrFail;

use crate::{chat_gpt::ChatGpt, claude::Claude, gist, message::MessageLog};

#[derive(Debug)]
pub struct Command {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub log: Option<PathBuf>,
    pub oneshot_log: Option<PathBuf>,
    pub model: String,
    pub system: Option<String>,
    pub gist: Option<String>,
}

impl Command {
    pub fn run(mut self) -> orfail::Result<()> {
        self.check_api_key().or_fail()?;

        let mut gist_offset = 0;
        let mut log = self
            .log_file_path()
            .filter(|path| path.exists())
            .map(MessageLog::load)
            .transpose()
            .or_fail()?
            .unwrap_or_default();
        if self.log.is_none() && self.oneshot_log.is_some() {
            log.messages.clear();
        }
        if let Some(id) = self.gist.as_ref().filter(|id| *id != "new") {
            log = gist::load(id).or_fail()?;
            gist_offset = log.messages.len();
        }
        if let Some(system) = &self.system {
            log.set_system_message_if_empty(system);
        }
        log.read_input().or_fail()?;

        let output = if self.model.starts_with("gpt") || self.model.starts_with("openai:") {
            self.model = self
                .model
                .strip_prefix("openai:")
                .unwrap_or(&self.model)
                .to_owned();
            let c = ChatGpt::new(&self).or_fail()?;
            let log = log.strip_model_name();
            c.run(&log).or_fail()?
        } else if self.model.starts_with("claude") || self.model.starts_with("anthropic:") {
            self.model = self
                .model
                .strip_prefix("anthropic:")
                .unwrap_or(&self.model)
                .to_owned();
            let c = Claude::new(&self).or_fail()?;
            let log = log.strip_model_name();
            c.run(&log).or_fail()?
        } else {
            unreachable!()
        };

        log.messages.push(output);
        if let Some(path) = self.log_file_path() {
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

    fn log_file_path(&self) -> Option<&PathBuf> {
        self.log.as_ref().or(self.oneshot_log.as_ref())
    }

    fn check_api_key(&self) -> orfail::Result<()> {
        if self.model.starts_with("gpt") || self.model.starts_with("openai:") {
            self.openai_api_key
                .is_some()
                .or_fail_with(|()| "OpenAI API key is not specified".to_owned())?;
        } else if self.model.starts_with("claude") || self.model.starts_with("anthropic:") {
            self.anthropic_api_key
                .is_some()
                .or_fail_with(|()| "Anthropic API key is not specified".to_owned())?;
        } else {
            return Err(orfail::Failure::new("unknown model"));
        }
        Ok(())
    }
}
