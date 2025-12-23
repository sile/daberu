use std::io::Read;

use orfail::OrFail;

pub fn pretty_print_reader<R: Read>(mut json_reader: R) -> orfail::Result<()> {
    let mut text = String::new();
    json_reader.read_to_string(&mut text).or_fail()?;
    pretty_print_text(&text).or_fail()?;
    Ok(())
}

pub fn pretty_print_text(json_text: &str) -> orfail::Result<()> {
    let json = nojson::RawJson::parse(&json_text).or_fail()?;
    let pretty = nojson::json(|f| {
        f.set_indent_size(2);
        f.set_spacing(true);
        f.value(json.value())
    });
    println!("{pretty}");
    Ok(())
}
