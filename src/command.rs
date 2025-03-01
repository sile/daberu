use std::path::PathBuf;

#[derive(Debug, clap::Args)]
pub struct Command {
    /// OpenAI API key.
    #[arg(
        long,
        value_name = "OPENAI_API_KEY",
        env = "OPENAI_API_KEY",
        hide_env_values = true
    )]
    openai_api_key: Option<String>,

    /// Anthropic API key.
    #[arg(
        long,
        value_name = "ANTHROPIC_API_KEY",
        env = "ANTHROPIC_API_KEY",
        hide_env_values = true
    )]
    anthropic_api_key: Option<String>,

    /// Log file path to save the conversation history. If the file already exists, the history will be considered in the next conversation.
    // TODO: Add env (DABERU_LOG_PATH)
    #[arg(long, value_name = "LOG_FILE_PATH")]
    log: Option<PathBuf>,

    /// ChatGPT model name.
    #[arg(long, env = "CHATGPT_MODEL", default_value = "gpt-4o")]
    model: String,

    /// If specified, the system role message will be added to the beginning of the conversation.
    #[arg(long, value_name = "SYSTEM_MESSAGE", env = "CHATGPT_SYSTEM_MESSAGE")]
    system: Option<String>,

    /// If specified, HTTP request and response body JSONs are printed to stderr.
    // TODO: remove?
    #[arg(long)]
    verbose: bool,

    #[arg(short, long)]
    echo_input: bool,
    // TODO: --truncate_log
    // TODO: --model-params
}
