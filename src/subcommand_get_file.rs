use std::path::PathBuf;

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
        .doc("ID of the file to download")
        .take(args)
        .then(|a| a.value().parse())?;
    let output_path: Option<PathBuf> = noargs::opt("output-file")
        .short('o')
        .ty("PATH")
        .doc("Output file path (if not specified, writes to stdout)")
        .take(args)
        .present_and_then(|a| a.value().parse())?;
    if args.metadata().help_mode {
        return Ok(());
    }

    let mut response = crate::curl::CurlRequest::new(format!(
        "https://api.anthropic.com/v1/files/{file_id}/content"
    ))
    .header("anthropic-version", "2023-06-01")
    .header("anthropic-beta", "files-api-2025-04-14")
    .header("X-Api-Key", &api_key)
    .get()
    .or_fail()?
    .check_success()
    .or_fail()?;

    if let Some(output_path) = output_path {
        let mut file = std::fs::File::create(&output_path).or_fail()?;
        std::io::copy(&mut response, &mut file).or_fail()?;
        eprintln!("Downloaded to: {}", output_path.display());
    } else {
        std::io::copy(&mut response, &mut std::io::stdout()).or_fail()?;
    }

    Ok(())
}
