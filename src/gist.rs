use std::{io::Write, process::Stdio};

use orfail::OrFail;

use crate::message::{Message, MessageLog, Role};

pub fn load(id: &str) -> orfail::Result<MessageLog> {
    let output = call(&["gist", "view", "--files", id]).or_fail()?;
    let mut filenames = output.lines().collect::<Vec<_>>();
    filenames.sort();

    let mut log = MessageLog::default();
    for (i, filename) in filenames.into_iter().enumerate() {
        let role = Role::from_gist_filename(filename, i).or_fail()?;
        let content = call(&["gist", "view", "--raw", "--filename", filename, id]).or_fail()?;
        log.messages.push(Message { role, content });
    }
    Ok(log)
}

pub fn create(log: &MessageLog) -> orfail::Result<()> {
    let message = log.messages.first().or_fail()?;
    let url = call_with_input(
        &[
            "gist",
            "create",
            "--desc",
            "daberu log",
            "--filename",
            &message.role.gist_filename(0),
            "-",
        ],
        &message.content,
    )
    .or_fail()?;
    eprintln!("{}", url.trim());

    update(url.trim(), log, 1).or_fail()?;
    Ok(())
}

pub fn update(id: &str, log: &MessageLog, offset: usize) -> orfail::Result<()> {
    for (i, message) in log.messages.iter().enumerate().skip(offset) {
        let filename = message.role.gist_filename(i);
        eprint!("Uploading gist {filename} ... ");
        call_with_input(
            &["gist", "edit", id, "-", "--add", &filename],
            &message.content,
        )
        .or_fail()?;
        eprintln!("done");
    }
    Ok(())
}

fn call(args: &[&str]) -> orfail::Result<String> {
    let output = std::process::Command::new("gh")
        .args(args)
        .stderr(Stdio::inherit())
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .or_fail_with(|()| format!("failed to execute `$ gh {}`", args.join(" ")))?;
    Ok(output)
}

fn call_with_input(args: &[&str], input: &str) -> orfail::Result<String> {
    let mut child = std::process::Command::new("gh")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .or_fail_with(|e| format!("Failed to execute `$ gh {}`: {e}", args.join(" ")))?;

    let mut stdin = child.stdin.take().or_fail()?;
    stdin.write_all(input.as_bytes()).or_fail()?;
    std::mem::drop(stdin);

    let output = child
        .wait_with_output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .or_fail_with(|()| format!("failed to execute `$ gh {}`", args.join(" ")))?;

    Ok(output)
}
