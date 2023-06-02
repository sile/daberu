daberu
======

[![daberu](https://img.shields.io/crates/v/daberu.svg)](https://crates.io/crates/daberu)
[![Documentation](https://docs.rs/daberu/badge.svg)](https://docs.rs/daberu)
[![Actions Status](https://github.com/sile/daberu/workflows/CI/badge.svg)](https://github.com/sile/daberu/actions)
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
      --model <MODEL>             ChatGPT model name [env: CHATGPT_MODEL=] [default: gpt-3.5-turbo] [possible values: gpt-4, gpt-4-0314, gpt-4-32k, gpt-4-32k-0314, gpt-3.5-turbo, gpt-3.5-turbo-0301]
      --system <SYSTEM_MESSAGE>   If specified, the system role message will be added to the beginning of the conversation [env: CHATGPT_SYSTEM_MESSAGE=]
      --verbose                   If specified, HTTP request and response body JSONs are printed to stderr
  -h, --help                      Print help
  -V, --version                   Print version
```

Installation
------------

```cosnole
$ cargo install daberu
```

Usage Examle
------------

```console
$ export OPENAI_API_KEY="YOUR API KEY"

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

References
----------

- [API Reference - OpenAI API](https://platform.openai.com/docs/api-reference/chat)
- [Chat completion - OpenAI API](https://platform.openai.com/docs/guides/chat)
