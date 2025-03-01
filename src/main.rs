use clap::Parser;
use orfail::OrFail;

use daberu::chat_gpt::ChatGpt;

/// ChatGPT client tool that reads your message from stdin and writes the response to stdout.
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[clap(flatten)]
    chatgpt: ChatGpt,
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    args.chatgpt.call().or_fail()?;
    Ok(())
}
