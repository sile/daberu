use orfail::OrFail;

pub fn run(args: &mut noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("anthropic-api-key")
        .ty("STRING")
        .env("ANTHROPIC_API_KEY")
        .doc("Anthropic API key")
        .example("YOUR_API_KEY")
        .take(args)
        .then(|a| a.value().parse())?;
    if args.metadata().help_mode {
        return Ok(());
    }

    // Get list of files
    let response = crate::curl::CurlRequest::new("https://api.anthropic.com/v1/files")
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "files-api-2025-04-14")
        .header("x-api-key", &api_key)
        .get()
        .or_fail()?
        .check_success()
        .or_fail()?;

    // Read response into string
    let response_text = std::io::read_to_string(response).or_fail()?;
    let raw = nojson::RawJson::parse(&response_text).or_fail()?;

    // Extract files array
    let files_value = raw.value().to_member("data")?.required().or_fail()?;

    // Iterate over files
    for file in files_value.to_array().or_fail()? {
        let file_id = file
            .to_member("id")?
            .required()
            .or_fail()?
            .to_unquoted_string_str()
            .or_fail()?;

        let filename = file
            .to_member("filename")?
            .required()
            .or_fail()?
            .to_unquoted_string_str()
            .or_fail()?;

        println!("Deleting file: {filename} ({file_id})");

        // Delete the file
        let delete_endpoint = format!("https://api.anthropic.com/v1/files/{file_id}");
        let _response = crate::curl::CurlRequest::new(&delete_endpoint)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "files-api-2025-04-14")
            .header("x-api-key", &api_key)
            .delete()
            .or_fail()?
            .check_success()
            .or_fail()?;

        println!("  => Successfully deleted: {filename}");
    }

    println!("All files cleaned up!");
    Ok(())
}
