daberu
======

[![daberu](https://img.shields.io/crates/v/daberu.svg)](https://crates.io/crates/daberu)
![License](https://img.shields.io/crates/l/daberu)

A simple command-line tool for conversing with ChatGPT.

"daberu" is a Japanese translation of "chatting".

```console
$ daberu -h
ChatGPT client tool that reads your message from stdin and writes the response to stdout

Usage: daberu [OPTIONS] --api-key <OPENAI_API_KEY>

Options:
      --api-key <OPENAI_API_KEY>  OpenAI API key [env: OPENAI_API_KEY]
      --log <LOG_FILE_PATH>       Log file path to save the conversation history. If the file already exists, the history will be considered in the next conversation
      --model <MODEL>             ChatGPT model name [env: CHATGPT_MODEL=] [default: gpt-4o] [possible values: gpt-4, gpt-4o, gpt-4-0314, gpt-4-32k, gpt-4-32k-0314, gpt-3.5-turbo, gpt-3.5-turbo-0301]
      --system <SYSTEM_MESSAGE>   If specified, the system role message will be added to the beginning of the conversation [env: CHATGPT_SYSTEM_MESSAGE=]
      --verbose                   If specified, HTTP request and response body JSONs are printed to stderr
  -h, --help                      Print help
  -V, --version                   Print version
```

Installation
------------

```cosnole
$ cargo install daberu
$ export OPENAI_API_KEY="YOUR API KEY"
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

- [API Reference - OpenAI API](https://platform.openai.com/docs/api-reference/chat)
- [Chat completion - OpenAI API](https://platform.openai.com/docs/guides/chat)
