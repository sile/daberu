use daberu::command::Command;
use orfail::OrFail;

fn main() -> noargs::Result<()> {
    let mut args = noargs::args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if noargs::HELP_FLAG.take(&mut args).is_present() {
        args.metadata_mut().help_mode = true;
    }

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let command = Command {
        openai_api_key: todo!(),
        anthropic_api_key: todo!(),
        log: todo!(),
        oneshot_log: todo!(),
        model: todo!(),
        system: todo!(),
        gist: todo!(),
    };
    command.run().or_fail()?;
    Ok(())
}
