use std::{io::Write, process::Stdio};

use orfail::OrFail;

use crate::message::MessageLog;

pub fn load(id: &str) -> orfail::Result<MessageLog> {
    //     > gh gist view --files ID
    // bar.md
    // gistfile0.txt
    // new.md
    todo!()
}

pub fn create(log: &MessageLog) -> orfail::Result<()> {
    println!();

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
    println!("{}", url.trim());

    update(url.trim(), log, 1).or_fail()?;
    Ok(())
}

pub fn update(id: &str, log: &MessageLog, offset: usize) -> orfail::Result<()> {
    for (i, message) in log.messages.iter().enumerate().skip(offset) {
        call_with_input(
            &[
                "gist",
                "edit",
                id,
                "-",
                "--add",
                &message.role.gist_filename(i),
            ],
            &message.content,
        )
        .or_fail()?;
    }
    Ok(())
}

fn call(args: &[&str]) -> orfail::Result<()> {
    std::process::Command::new("gh")
        .args(args)
        .status()
        .is_ok_and(|s| s.success())
        .or_fail_with(|()| format!("failed to execute `$ gh gist {}`", args.join(" ")))?;
    Ok(())
}

fn call_with_input(args: &[&str], input: &str) -> orfail::Result<String> {
    let mut child = std::process::Command::new("gh")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .or_fail_with(|e| format!("Failed to execute `$ gh gist {}`: {e}", args.join(" ")))?;

    let mut stdin = child.stdin.take().or_fail()?;
    stdin.write_all(input.as_bytes()).or_fail()?;
    std::mem::drop(stdin);

    let output = child
        .wait_with_output()
        .map_err(|_| ())
        .and_then(|output| String::from_utf8(output.stdout).map_err(|_| ()))
        .ok()
        .or_fail_with(|()| format!("failed to execute `$ gh gist {}`", args.join(" ")))?;

    Ok(output)
}
