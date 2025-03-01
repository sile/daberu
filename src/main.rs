use clap::Parser;
use orfail::OrFail;

/// ChatGPT / Claude client tool that reads your message from stdin and writes the response to stdout.
#[derive(Debug, Parser)]
#[command(version)]
struct Args {
    #[clap(flatten)]
    command: daberu::command::Command,
}

fn main() -> orfail::Result<()> {
    let args = Args::parse();
    args.command.run().or_fail()?;
    Ok(())
}
