daberu
======

[![daberu](https://img.shields.io/crates/v/daberu.svg)](https://crates.io/crates/daberu)
[![Actions Status](https://github.com/sile/daberu/workflows/CI/badge.svg)](https://github.com/sile/daberu/actions)
![License](https://img.shields.io/crates/l/daberu)

A simple command-line tool for conversing with ChatGPT / Claude.

"daberu" is a Japanese translation of "chatting".

```console
$ daberu -h
ChatGPT / Claude client tool that reads your message from stdin and writes the response to stdout

Usage: daberu [OPTIONS]

Options:
      --openai-api-key <OPENAI_API_KEY>
          OpenAI API key [env: OPENAI_API_KEY]
      --anthropic-api-key <ANTHROPIC_API_KEY>
          Anthropic API key [env: ANTHROPIC_API_KEY]
      --log <LOG>
          Path to log file for saving conversation history. If the file already exists, its contents are included in subsequent conversations
      --oneshot-log <DABERU_ONESHOT_LOG_PATH>
          Specifies a path for a "one-shot" conversation log file which will overwrite any existing file content upon opening
      --model <MODEL>
          Model name [env: DABERU_MODEL=] [default: gpt-4o]
      --system <SYSTEM_MESSAGE>
          System message [env: DABERU_SYSTEM_MESSAGE=]
      --gist <new | EXISTING_GIST_ID>
          Save the output to GitHub Gist
  -h, --help
          Print help (see more with '--help')
  -V, --version
          Print version
```

Installation
------------

```cosnole
$ cargo install daberu

$ export OPENAI_API_KEY="YOUR API KEY"
// or
$ export ANTHROPIC_API_KEY="YOUR API KEY"
```

Usage Examle
------------

```console
// Conversation without history.
$ echo hi | daberu
Hello! How can I assist you today?

// Conversation with history.
$ echo "Translate to Japanese: 'hi'" | daberu --log chat.log
こんにちは

$ echo "Next, translate to Spanish" | daberu --log chat.log
hola

$ jq . chat.log
[
  {
    "role": "user",
    "content": "Translate to Japanese: 'hi'\n"
  },
  {
    "role": "assistant",
    "content": "こんにちは"
  },
  {
    "role": "user",
    "content": "Next, translate to Spanish\n"
  },
  {
    "role": "assistant",
    "content": "hola"
  }
]
```

The following example utilizes shell pipeline to get advice from ChatGPT about an existing file:
```console
$ git clone https://github.com/sile/daberu
$ cd daberu/

$ echo 'How to improve the following code?:' | cat - src/main.rs | daberu
The code looks concise and well-structured. A few suggestions to further improve it could be:

1. Add more documentation to the code, especially to the `Args` struct and its fields. This will help users understand the purpose and usage of each field.

2. Handle errors and display meaningful error messages to the user. Right now, the code uses the `orfail` crate to return an error message, but it doesn't provide any additional context or information about the error. Consider adding more descriptive error messages or logging errors to a file.

3. Add unit tests to verify the correctness of the code. This is especially important since the code relies on external libraries and APIs.

4. Consider adding more features to the ChatGPT client tool, such as allowing users to specify the input and output files, or enabling interactive mode where the user can have multiple conversations with the model.
```

In the next example, a system role message is used to instruct ChatGPT:
```console
$ git diff d8be79e..cb34a00 | daberu --system 'Please write consice changelog entries for the following diff.'
- Added author, license, description, homepage, repository, and readme fields to Cargo.toml
- Removed documentation badge and Actions Status badge from README.md
- Added an example utilizing shell pipeline to get advice from ChatGPT about an existing file to README.md
```

References
----------

- ChatGPT:
  - [API Reference - OpenAI API](https://platform.openai.com/docs/api-reference/chat)
  - [Chat completion - OpenAI API](https://platform.openai.com/docs/guides/chat)
- Claude:
  - [API](https://www.anthropic.com/api)
