use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("YOUR_API_KEY")
        .take(args)
        .then(|a| a.value().parse())?;
    let file_id: String = noargs::arg("FILE_ID")
        .example("file_01AbCdEfGhIjKlMnOpQrStUv")
        .doc("ID of the file to retrieve metadata for")
        .take(args)
        .then(|a| a.value().parse())?;
    if args.metadata().help_mode {
        return Ok(());
    }

    let response =
        crate::curl::CurlRequest::new(format!("https://api.anthropic.com/v1/files/{file_id}"))
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .header("X-Api-Key", &api_key)
            .get()
            .or_fail()?;

    let response = response.check_success().or_fail()?;
    crate::json::pretty_print_reader(response).or_fail()?;

    Ok(())
}
