# noargs - Rust Command-Line Argument Parser

## Crate Summary

### Overview

`noargs` is an imperative command-line argument parser library for Rust that prioritizes simplicity and explicitness. It has zero dependencies, no procedural macros, and performs no implicit I/O operations. The library follows an imperative programming style where you explicitly take arguments in sequence, making the control flow transparent and predictable.

### Key Use Cases and Benefits

- **Simple CLI applications**: Build command-line tools without learning complex DSLs or macro syntax
- **Explicit control flow**: Arguments are consumed imperatively, making the parsing logic clear
- **Type-safe parsing**: Parse arguments into any type implementing `FromStr` with proper error handling
- **Automatic help generation**: Help text is automatically generated from specifications
- **Subcommand support**: Build multi-command CLIs with isolated argument handling per subcommand
- **Environment variable integration**: Options and flags can fall back to environment variables

### Design Philosophy

1. **No magic**: Everything is explicit - you call methods to take arguments in order
2. **No implicit I/O**: The library never reads from stdin, prints to stdout, or exits the process
3. **Imperative over declarative**: Rather than defining a struct and deriving traits, you write normal Rust code
4. **Progressive disclosure**: Simple use cases are simple; complex scenarios are supported without hidden complexity
5. **Composability**: Argument parsing is separated from help generation and error handling

---

## API Reference

### Core Types

#### `noargs::RawArgs`

Container for raw command-line arguments that will be parsed into structured argument types.

```rust
pub struct RawArgs { /* fields omitted */ }
```

**Methods:**

- `noargs::RawArgs::new<I>(args: I) -> noargs::RawArgs where I: Iterator<Item = String>` - Creates a new `RawArgs` instance from an iterator of strings
- `noargs::RawArgs::metadata(&self) -> noargs::Metadata` - Returns the current metadata
- `noargs::RawArgs::metadata_mut(&mut self) -> &mut noargs::Metadata` - Returns a mutable reference to metadata
- `noargs::RawArgs::remaining_args(&self) -> impl Iterator<Item = (usize, &str)>` - Returns unconsumed arguments with their indices
- `noargs::RawArgs::finish(self) -> Result<Option<String>, noargs::Error>` - Completes parsing and validates no unexpected arguments remain. Returns help text if in help mode

#### `noargs::Metadata`

Configuration and metadata for argument parsing and help generation.

```rust
pub struct Metadata {
    pub app_name: &'static str,
    pub app_description: &'static str,
    pub help_flag_name: Option<&'static str>,
    pub help_mode: bool,
    pub full_help: bool,
    pub is_valid_flag_chars: fn(&str) -> bool,
}
```

**Fields:**

- `app_name` - Application name (typically `env!("CARGO_PKG_NAME")`)
- `app_description` - Application description (typically `env!("CARGO_PKG_DESCRIPTION")`)
- `help_flag_name` - Name of the help flag (default: `Some("help")`)
- `help_mode` - When true, parsing returns example/default values and `finish()` returns help text
- `full_help` - When true, displays full multi-line documentation instead of summaries
- `is_valid_flag_chars` - Predicate to determine valid flag character sequences (default: ASCII alphabetic only)

---

### Positional Arguments

#### `noargs::ArgSpec`

Specification for positional arguments.

```rust
pub struct ArgSpec {
    pub name: &'static str,
    pub doc: &'static str,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
}
```

**Constructor Functions:**

- `noargs::arg(name: &'static str) -> noargs::ArgSpec` - Creates an `ArgSpec` with the given name
- `noargs::ArgSpec::new(name: &'static str) -> noargs::ArgSpec` - Equivalent to `noargs::arg(name)`

**Builder Methods:**

- `noargs::ArgSpec::doc(self, doc: &'static str) -> noargs::ArgSpec` - Sets documentation string
- `noargs::ArgSpec::default(self, default: &'static str) -> noargs::ArgSpec` - Sets default value (makes argument optional)
- `noargs::ArgSpec::example(self, example: &'static str) -> noargs::ArgSpec` - Sets example value for help text (marks as required in help)

**Parsing Method:**

- `noargs::ArgSpec::take(self, args: &mut noargs::RawArgs) -> noargs::Arg` - Consumes and returns the first matching positional argument

#### `noargs::Arg`

A positional argument value.

```rust
pub enum Arg {
    Positional { spec: ArgSpec, metadata: Metadata, index: usize, value: String },
    Default { spec: ArgSpec, metadata: Metadata },
    Example { spec: ArgSpec, metadata: Metadata },
    None { spec: ArgSpec },
}
```

**Methods:**

- `noargs::Arg::spec(&self) -> noargs::ArgSpec` - Returns the specification
- `noargs::Arg::is_present(&self) -> bool` - Returns true if argument has a value
- `noargs::Arg::present(self) -> Option<noargs::Arg>` - Returns `Some(self)` if present
- `noargs::Arg::value(&self) -> &str` - Returns the raw value or empty string if not present
- `noargs::Arg::index(&self) -> Option<usize>` - Returns the position in the raw arguments
- `noargs::Arg::then<F, T, E>(self, f: F) -> Result<T, noargs::Error> where F: FnOnce(noargs::Arg) -> Result<T, E>, E: std::fmt::Display` - Applies conversion/validation; returns `Error::MissingArg` if not present, `Error::InvalidArg` if conversion fails
- `noargs::Arg::present_and_then<F, T, E>(self, f: F) -> Result<Option<T>, noargs::Error>` - Shorthand for `self.present().map(|arg| arg.then(f)).transpose()`

---

### Named Options (with values)

#### `noargs::OptSpec`

Specification for named options that take values.

```rust
pub struct OptSpec {
    pub name: &'static str,
    pub short: Option<char>,
    pub ty: &'static str,
    pub doc: &'static str,
    pub env: Option<&'static str>,
    pub default: Option<&'static str>,
    pub example: Option<&'static str>,
}
```

**Constructor Functions:**

- `noargs::opt(name: &'static str) -> noargs::OptSpec` - Creates an `OptSpec` with the given name
- `noargs::OptSpec::new(name: &'static str) -> noargs::OptSpec` - Equivalent to `noargs::opt(name)`

**Builder Methods:**

- `noargs::OptSpec::short(self, name: char) -> noargs::OptSpec` - Sets short name (single character)
- `noargs::OptSpec::ty(self, value_type: &'static str) -> noargs::OptSpec` - Sets value type name for help (default: "VALUE")
- `noargs::OptSpec::doc(self, doc: &'static str) -> noargs::OptSpec` - Sets documentation string
- `noargs::OptSpec::env(self, variable_name: &'static str) -> noargs::OptSpec` - Sets environment variable fallback
- `noargs::OptSpec::default(self, default: &'static str) -> noargs::OptSpec` - Sets default value (makes option optional)
- `noargs::OptSpec::example(self, example: &'static str) -> noargs::OptSpec` - Sets example value for help text (marks as required in help)

**Parsing Method:**

- `noargs::OptSpec::take(self, args: &mut noargs::RawArgs) -> noargs::Opt` - Consumes and returns the first matching option

**Supported Formats:**

Long options support: `--name=value` and `--name value`

Short options support: `-k value` and `-kvalue` (concatenated)

#### `noargs::Opt`

A named option value.

```rust
pub enum Opt {
    Long { spec: OptSpec, metadata: Metadata, index: usize, value: String },
    Short { spec: OptSpec, metadata: Metadata, index: usize, value: String },
    Env { spec: OptSpec, metadata: Metadata, value: String },
    Default { spec: OptSpec, metadata: Metadata },
    Example { spec: OptSpec, metadata: Metadata },
    MissingValue { spec: OptSpec, long: bool },
    None { spec: OptSpec },
}
```

**Methods:**

- `noargs::Opt::spec(&self) -> noargs::OptSpec` - Returns the specification
- `noargs::Opt::is_present(&self) -> bool` - Returns true if option is present
- `noargs::Opt::is_value_present(&self) -> bool` - Returns true if option has a value (not `None` or `MissingValue`)
- `noargs::Opt::present(self) -> Option<noargs::Opt>` - Returns `Some(self)` if present
- `noargs::Opt::value(&self) -> &str` - Returns the raw value or empty string if not present
- `noargs::Opt::index(&self) -> Option<usize>` - Returns the position in the raw arguments
- `noargs::Opt::then<F, T, E>(self, f: F) -> Result<T, noargs::Error> where F: FnOnce(noargs::Opt) -> Result<T, E>, E: std::fmt::Display` - Applies conversion/validation; returns `Error::MissingOpt` if not present, `Error::InvalidOpt` if conversion fails
- `noargs::Opt::present_and_then<F, T, E>(self, f: F) -> Result<Option<T>, noargs::Error>` - Shorthand for `self.present().map(|opt| opt.then(f)).transpose()`

---

### Flags (boolean switches)

#### `noargs::FlagSpec`

Specification for boolean flags (options without values).

```rust
pub struct FlagSpec {
    pub name: &'static str,
    pub short: Option<char>,
    pub doc: &'static str,
    pub env: Option<&'static str>,
}
```

**Constructor Functions:**

- `noargs::flag(name: &'static str) -> noargs::FlagSpec` - Creates a `FlagSpec` with the given name
- `noargs::FlagSpec::new(name: &'static str) -> noargs::FlagSpec` - Equivalent to `noargs::flag(name)`

**Builder Methods:**

- `noargs::FlagSpec::short(self, name: char) -> noargs::FlagSpec` - Sets short name (single character)
- `noargs::FlagSpec::doc(self, doc: &'static str) -> noargs::FlagSpec` - Sets documentation string
- `noargs::FlagSpec::env(self, variable_name: &'static str) -> noargs::FlagSpec` - Sets environment variable (flag is set if variable is non-empty)

**Parsing Methods:**

- `noargs::FlagSpec::take(self, args: &mut noargs::RawArgs) -> noargs::Flag` - Consumes and returns the first matching flag
- `noargs::FlagSpec::take_help(self, args: &mut noargs::RawArgs) -> noargs::Flag` - Like `take()`, but also updates metadata for help mode (sets `help_mode=true`, `full_help=true` for long form)

**Note:** Multiple short flags can be combined (e.g., `-abc` is equivalent to `-a -b -c`)

#### `noargs::Flag`

A boolean flag value.

```rust
pub enum Flag {
    Long { spec: FlagSpec, index: usize },
    Short { spec: FlagSpec, index: usize },
    Env { spec: FlagSpec },
    None { spec: FlagSpec },
}
```

**Methods:**

- `noargs::Flag::spec(self) -> noargs::FlagSpec` - Returns the specification
- `noargs::Flag::is_present(self) -> bool` - Returns true if flag is set
- `noargs::Flag::present(self) -> Option<noargs::Flag>` - Returns `Some(self)` if present
- `noargs::Flag::index(self) -> Option<usize>` - Returns the position in the raw arguments

---

### Subcommands

#### `noargs::CmdSpec`

Specification for subcommands.

```rust
pub struct CmdSpec {
    pub name: &'static str,
    pub doc: &'static str,
}
```

**Constructor Functions:**

- `noargs::cmd(name: &'static str) -> noargs::CmdSpec` - Creates a `CmdSpec` with the given name
- `noargs::CmdSpec::new(name: &'static str) -> noargs::CmdSpec` - Equivalent to `noargs::cmd(name)`

**Builder Methods:**

- `noargs::CmdSpec::doc(self, doc: &'static str) -> noargs::CmdSpec` - Sets documentation string

**Parsing Method:**

- `noargs::CmdSpec::take(self, args: &mut noargs::RawArgs) -> noargs::Cmd` - Consumes and returns the subcommand if it's the next unconsumed positional argument

#### `noargs::Cmd`

A subcommand value.

```rust
pub enum Cmd {
    Some { spec: CmdSpec, index: usize },
    None { spec: CmdSpec },
}
```

**Methods:**

- `noargs::Cmd::spec(self) -> noargs::CmdSpec` - Returns the specification
- `noargs::Cmd::is_present(self) -> bool` - Returns true if subcommand is present
- `noargs::Cmd::present(self) -> Option<noargs::Cmd>` - Returns `Some(self)` if present
- `noargs::Cmd::index(self) -> Option<usize>` - Returns the position in the raw arguments

---

### Error Handling

#### `noargs::Error`

Error type for argument parsing failures.

```rust
pub enum Error {
    UnexpectedArg { metadata: Metadata, raw_arg: String },
    UndefinedCommand { metadata: Metadata, raw_arg: String },
    MissingCommand { metadata: Metadata },
    InvalidArg { arg: Box<Arg>, reason: String },
    MissingArg { arg: Box<Arg> },
    InvalidOpt { opt: Box<Opt>, reason: String },
    MissingOpt { opt: Box<Opt> },
    Other { metadata: Option<Metadata>, error: Box<dyn std::fmt::Display> },
}
```

**Methods:**

- `noargs::Error::other<E>(args: &noargs::RawArgs, error: E) -> noargs::Error where E: 'static + std::fmt::Display` - Creates an application-specific error

**Conversions:**

`noargs::Error` implements `From<T>` for any `T: 'static + std::fmt::Display`, enabling `?` operator usage with custom error types.

**Note:** This error type intentionally does not implement `std::error::Error` or `std::fmt::Display` as it's designed for top-level error handling. It implements `Debug` which formats errors with ANSI colors when outputting to a terminal.

#### `noargs::Result<T>`

Type alias for `std::result::Result<T, noargs::Error>`.

---

### Top-Level Functions

- `noargs::raw_args() -> noargs::RawArgs` - Creates `RawArgs` from `std::env::args()` (shorthand for `RawArgs::new(std::env::args())`)
- `noargs::arg(name: &'static str) -> noargs::ArgSpec` - Creates a positional argument specification
- `noargs::opt(name: &'static str) -> noargs::OptSpec` - Creates a named option specification
- `noargs::flag(name: &'static str) -> noargs::FlagSpec` - Creates a flag specification
- `noargs::cmd(name: &'static str) -> noargs::CmdSpec` - Creates a subcommand specification

### Well-Known Constants

- `noargs::HELP_FLAG: noargs::FlagSpec` - Pre-configured help flag (`--help, -h`)
- `noargs::VERSION_FLAG: noargs::FlagSpec` - Pre-configured version flag (`--version`)

---

## Examples and Common Patterns

### Basic Application with Options and Flags

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    
    // Set metadata
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");
    
    // Handle version flag
    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    
    // Handle help flag
    noargs::HELP_FLAG.take_help(&mut args);
    
    // Parse application arguments
    let verbose: bool = noargs::flag("verbose")
        .short('v')
        .doc("Enable verbose output")
        .take(&mut args)
        .is_present();
    
    let count: usize = noargs::opt("count")
        .short('c')
        .doc("Number of iterations")
        .default("10")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    
    let input: String = noargs::arg("<INPUT>")
        .doc("Input file path")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    
    // Finish parsing
    if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    // Application logic
    println!("verbose={}, count={}, input={}", verbose, count, input);
    
    Ok(())
}
```

### Optional Arguments

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "myapp";
    
    noargs::HELP_FLAG.take_help(&mut args);
    
    // Optional argument with default
    let output: String = noargs::arg("[OUTPUT]")
        .doc("Output file (defaults to stdout)")
        .default("-")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    
    // Optional argument without default (returns Option)
    let config: Option<String> = noargs::arg("[CONFIG]")
        .doc("Configuration file path")
        .take(&mut args)
        .present_and_then(|a| a.value().parse())?;
    
    if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    println!("output={}, config={:?}", output, config);
    Ok(())
}
```

### Subcommands

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "git-like";
    
    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("git-like 1.0.0");
        return Ok(());
    }
    
    noargs::HELP_FLAG.take_help(&mut args);
    
    // Define subcommands
    if noargs::cmd("clone")
        .doc("Clone a repository")
        .take(&mut args)
        .is_present()
    {
        let url: String = noargs::arg("<URL>")
            .doc("Repository URL")
            .take(&mut args)
            .then(|a| a.value().parse())?;
        
        let depth: Option<usize> = noargs::opt("depth")
            .doc("Clone depth")
            .take(&mut args)
            .present_and_then(|o| o.value().parse())?;
        
        if let Some(help) = args.finish()? {
            print!("{}", help);
            return Ok(());
        }
        
        println!("Cloning {} with depth {:?}", url, depth);
    } else if noargs::cmd("push")
        .doc("Push changes")
        .take(&mut args)
        .is_present()
    {
        let force: bool = noargs::flag("force")
            .short('f')
            .doc("Force push")
            .take(&mut args)
            .is_present();
        
        if let Some(help) = args.finish()? {
            print!("{}", help);
            return Ok(());
        }
        
        println!("Pushing with force={}", force);
    } else if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    Ok(())
}
```

### Environment Variable Fallback

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "myapp";
    
    noargs::HELP_FLAG.take_help(&mut args);
    
    // Option that falls back to environment variable
    let api_key: String = noargs::opt("api-key")
        .doc("API key for authentication")
        .env("MYAPP_API_KEY")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    
    // Flag controlled by environment variable
    let debug: bool = noargs::flag("debug")
        .doc("Enable debug mode")
        .env("MYAPP_DEBUG")
        .take(&mut args)
        .is_present();
    
    if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    println!("api_key={}, debug={}", api_key, debug);
    Ok(())
}
```

### Variadic Arguments

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "myapp";
    
    noargs::HELP_FLAG.take_help(&mut args);
    
    // Collect all remaining positional arguments
    let mut files: Vec<String> = Vec::new();
    loop {
        let arg_spec = noargs::arg("[FILE]...")
            .doc("Input files");
        
        let result = arg_spec.take(&mut args)
            .present_and_then(|a| a.value().parse())?;
        
        match result {
            Some(file) => files.push(file),
            None => break,
        }
    }
    
    if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    println!("Processing files: {:?}", files);
    Ok(())
}
```

### Custom Validation

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = "myapp";
    
    noargs::HELP_FLAG.take_help(&mut args);
    
    // Argument with custom validation
    let port: u16 = noargs::opt("port")
        .short('p')
        .doc("Server port")
        .default("8080")
        .take(&mut args)
        .then(|o| -> Result<u16, String> {
            let port: u16 = o.value().parse()
                .map_err(|e| format!("invalid port number: {}", e))?;
            
            if port < 1024 {
                return Err("port must be >= 1024".to_string());
            }
            
            Ok(port)
        })?;
    
    if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    println!("Starting server on port {}", port);
    Ok(())
}
```

### Error Handling Patterns

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    
    // Using ? operator with parse errors
    let timeout: std::time::Duration = noargs::opt("timeout")
        .doc("Request timeout in seconds")
        .default("30")
        .take(&mut args)
        .then(|o| {
            let secs: u64 = o.value().parse()?;
            Ok(std::time::Duration::from_secs(secs))
        })?;
    
    // Custom application error
    if timeout.as_secs() > 300 {
        return Err(noargs::Error::other(&args, "timeout cannot exceed 5 minutes"));
    }
    
    // Check for unexpected arguments
    if let Some(help) = args.finish()? {
        print!("{}", help);
        return Ok(());
    }
    
    Ok(())
}
```

### Dynamic Default Values

```rust
use std::sync::LazyLock;

fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    
    // Use LazyLock for computed default values
    const DEFAULT_THREADS: usize = 4;
    static DEFAULT_THREADS_STR: LazyLock<String> = 
        LazyLock::new(|| DEFAULT_THREADS.to_string());
    
    let threads: usize = noargs::opt("threads")
        .doc("Number of worker threads")
        .default(&*DEFAULT_THREADS_STR)
        .take(&mut args)
        .then(|o| o.value().parse())?;
    
    println!("Using {} threads", threads);
    Ok(())
}
```

### Conditional Argument Parsing

```rust
fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    
    let format: String = noargs::opt("format")
        .doc("Output format: json or yaml")
        .default("json")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    
    // Parse different options based on format
    match format.as_str() {
        "json" => {
            let pretty: bool = noargs::flag("pretty")
                .doc("Pretty-print JSON output")
                .take(&mut args)
                .is_present();
            
            println!("JSON format, pretty={}", pretty);
        }
        "yaml" => {
            let indent: usize = noargs::opt("indent")
                .doc("YAML indentation level")
                .default("2")
                .take(&mut args)
                .then(|o| o.value().parse())?;
            
            println!("YAML format, indent={}", indent);
        }
        _ => {
            return Err(noargs::Error::other(&args, 
                format!("unknown format: {}", format)));
        }
    }
    
    args.finish()?;
    Ok(())
}
```
