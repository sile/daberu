use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("FOOBARBAZ")
        .take(args)
        .then(|a| a.value().parse())?;

    if args.metadata().help_mode {
        return Ok(());
    }

    let response = crate::curl::CurlRequest::new("https://api.anthropic.com/v1/skills")
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "skills-2025-10-02")
        .header("X-Api-Key", &api_key)
        .get()
        .or_fail()?;

    let mut reader = response.check_success().or_fail()?;
    let mut response = String::new();
    reader.read_to_string(&mut response).or_fail()?;
    println!("{response}");

    Ok(())
}
