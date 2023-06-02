use clap::Parser;
use daberu::ChatGpt;
use orfail::OrFail;

/// A tool for conversing with ChatGPT.
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
