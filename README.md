daberu
======

[![daberu](https://img.shields.io/crates/v/daberu.svg)](https://crates.io/crates/daberu)
[![Actions Status](https://github.com/sile/daberu/workflows/CI/badge.svg)](https://github.com/sile/daberu/actions)
![License](https://img.shields.io/crates/l/daberu)

A simple command-line tool for conversing with Claude.

"daberu" is a Japanese translation of "chatting".

```console
$ daberu -h
Claude client tool that reads your message from stdin and writes the response to stdout

Usage: daberu [OPTIONS]

Options:
      --version                    Print version
  -h, --help                       Print help ('--help' for full help, '-h' for summary)
  -x, --ext                        Enable extended subcommands
      --config-file <PATH>         Path to configuration JSONC file [env: DABERU_CONFIG_FILE]
      --anthropic-api-key <STRING> Anthropic API key [env: ANTHROPIC_API_KEY]
  -l, --log <PATH>                 Path to log file for saving the last conversation [env: DABERU_LOG_PATH]
  -c, --continue                   Continue conversation from the existing log file instead of truncating it
  -a, --enable-agents-md           Automatically load AGENTS.md or CLAUDE.md as a resource [env: DABERU_ENABLE_AGENTS_MD]
  -m, --model <MODEL_NAME>         Model name [env: DABERU_MODEL] [default: claude-sonnet-4-5]
  -s, --system <STRING>            System message
  -r, --resource <PATH>            File path to be used as a resource for the conversion
  -e, --shell-command <COMMAND>    Shell command to be used as a resource for the conversion
```

Installation
------------

```console
$ cargo install daberu

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

References
----------

- Claude:
  - [API](https://www.anthropic.com/api)
