use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("YOUR_API_KEY")
        .take(args)
        .then(|a| a.value().parse())?;
    let custom_source_only: bool = noargs::flag("custom-source-only")
        .short('c')
        .doc("TODO")
        .take(args)
        .is_present();
    if args.metadata().help_mode {
        return Ok(());
    }

    let url = if custom_source_only {
        "https://api.anthropic.com/v1/skills?source=custom"
    } else {
        "https://api.anthropic.com/v1/skills"
    };
    let response = crate::curl::CurlRequest::new(url)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "skills-2025-10-02")
        .header("X-Api-Key", &api_key)
        .get()
        .or_fail()?;

    let response = response.check_success().or_fail()?;
    crate::json::pretty_print_reader(response).or_fail()?;

    Ok(())
}
