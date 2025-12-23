use std::path::PathBuf;

use crate::message::MessageLog;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let log: PathBuf = noargs::opt("log")
        .short('l')
        .ty("PATH")
        .env("DABERU_LOG_PATH")
        .doc(concat!("TODO"))
        .take(args)
        .then(|a| a.value().parse())?;
    let log = MessageLog::load(log)?;
    if let Some(m) = log.messages.last() {
        println!("{}", m.content);
    }
    Ok(())
}
