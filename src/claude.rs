use orfail::OrFail;

use crate::{
    command::Command,
    message::{Message, MessageLog},
};

#[derive(Debug)]
pub struct Claude {
    api_key: String,
    model: String,
}

impl Claude {
    pub fn new(command: &Command) -> orfail::Result<Self> {
        Ok(Self {
            api_key: command.anthropic_api_key.clone().or_fail()?,
            model: command.model.clone(),
        })
    }

    pub fn run(&self, output_header: &str, log: &MessageLog) -> orfail::Result<Message> {
        todo!()
    }
}
