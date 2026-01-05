# Comprehensive Documentation for `orfail` Crate

## 1. Crate Summary

### Overview

`orfail` is a Rust error handling library designed for portable unrecoverable errors. It provides a lightweight, `no_std` compatible error type that can be easily serialized and transmitted across process and language boundaries. The crate emphasizes simplicity and portability by using only basic types (`u32`, `String`, and `Vec`) for its error representation.

### Key Use Cases and Benefits

- **Portable Error Handling**: Errors consist only of primitive types, making them serializable and transferable across different contexts
- **User-Level Backtraces**: Automatically builds a backtrace of error propagation points without relying on platform-specific stack unwinding
- **Simplified Error Conversion**: The `OrFail` trait provides ergonomic conversion from common types (`bool`, `Option`, `Result`) to the `Failure` type
- **No_std Support**: Works in embedded and resource-constrained environments
- **Explicit Error Propagation**: Each call to `or_fail()` adds a location to the backtrace, making error paths explicit and traceable

### Design Philosophy

The crate deliberately does **not** implement `std::error::Error` for its `Failure` type. This design decision emphasizes that `Failure` represents truly unrecoverable errors that should be handled at application boundaries rather than being part of recoverable error chains. The focus is on simplicity, portability, and explicit error propagation with minimal dependencies.

---

## 2. API Reference

### Type Aliases

#### `orfail::Result<T>`

```rust
pub type Result<T> = core::result::Result<T, orfail::Failure>
```

A convenience type alias for `Result<T, Failure>`. This is the primary result type used throughout code that leverages this crate.

---

### Core Types

#### `orfail::Failure`

```rust
pub struct Failure {
    pub message: String,
    pub backtrace: Vec<orfail::Location>,
}
```

Represents an unrecoverable error with an error message and a user-level backtrace.

**Fields:**
- `message: String` - The error message describing what went wrong
- `backtrace: Vec<Location>` - A vector of source code locations showing the error propagation path

**Methods:**

##### `orfail::Failure::new`

```rust
#[track_caller]
pub fn new<T: core::fmt::Display>(message: T) -> Self
```

Creates a new `Failure` instance with the given message. The backtrace is automatically initialized with the caller's location.

**Parameters:**
- `message: T` - Any type that implements `Display`, used as the error message

**Returns:** A new `Failure` instance with a single-entry backtrace

**Example:**

```rust
fn example() -> orfail::Result<()> {
    Err(orfail::Failure::new("something went wrong"))
}
```

##### `orfail::Failure::default`

```rust
#[track_caller]
fn default() -> Self
```

Creates a `Failure` with the default message "a failure occurred" and the caller's location.

**Trait Implementations:**

- `Clone`: Full clone of message and backtrace
- `PartialEq`, `Eq`, `Hash`: Equality based on message and backtrace
- `Debug`: Delegates to `Display` implementation
- `Display`: Formats the error message followed by backtrace locations, each prefixed with "  at "

---

#### `orfail::Location`

```rust
pub struct Location {
    pub file: String,
    pub line: u32,
}
```

Represents a single location in a source file, used to build the backtrace in `Failure`.

**Fields:**
- `file: String` - The source file path
- `line: u32` - The line number in the source file

**Methods:**

##### `orfail::Location::new`

```rust
#[track_caller]
pub fn new() -> Self
```

Creates a new `Location` instance capturing the caller's file and line number.

**Returns:** A `Location` with the current caller's position

##### `orfail::Location::default`

```rust
#[track_caller]
fn default() -> Self
```

Same as `new()`, creates a location at the caller's position.

**Trait Implementations:**

- `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`: Standard implementations for comparison and hashing

---

### Traits

#### `orfail::OrFail`

```rust
pub trait OrFail: Sized {
    type Value;
    type Error;
    
    fn or_fail(self) -> orfail::Result<Self::Value>;
    fn or_fail_with<F>(self, f: F) -> orfail::Result<Self::Value>
    where
        F: FnOnce(Self::Error) -> String;
}
```

A trait for converting various types into `Result<T, Failure>` with automatic backtrace tracking.

**Associated Types:**
- `Value` - The success type when conversion succeeds
- `Error` - The error information used to construct failure messages

**Methods:**

##### `orfail::OrFail::or_fail`

```rust
#[track_caller]
fn or_fail(self) -> orfail::Result<Self::Value>
```

Converts the value into a `Result<Self::Value, Failure>`. On failure, adds the caller's location to the backtrace.

**Returns:** `Ok(value)` if successful, `Err(Failure)` with an updated backtrace otherwise

##### `orfail::OrFail::or_fail_with`

```rust
#[track_caller]
fn or_fail_with<F>(self, f: F) -> orfail::Result<Self::Value>
where
    F: FnOnce(Self::Error) -> String
```

Like `or_fail()`, but allows customization of the error message using a closure.

**Parameters:**
- `f: F` - A closure that takes the error information and returns a custom error message

**Returns:** `Ok(value)` if successful, `Err(Failure)` with custom message and updated backtrace otherwise

---

### Trait Implementations

#### `OrFail` for `bool`

```rust
impl orfail::OrFail for bool {
    type Value = ();
    type Error = ();
}
```

Treats `false` as a failure condition.

**Behavior:**
- `true.or_fail()` returns `Ok(())`
- `false.or_fail()` returns `Err(Failure)` with message "expected `true` but got `false`"

**Example:**

```rust
fn check_positive(n: i32) -> orfail::Result<()> {
    (n > 0).or_fail()?;
    Ok(())
}
```

---

#### `OrFail` for `Option<T>`

```rust
impl<T> orfail::OrFail for Option<T> {
    type Value = T;
    type Error = ();
}
```

Converts `Option<T>` to `Result<T, Failure>`.

**Behavior:**
- `Some(value).or_fail()` returns `Ok(value)`
- `None.or_fail()` returns `Err(Failure)` with message "expected `Some(_)` but got `None`"

**Example:**

```rust
fn get_config(key: &str) -> orfail::Result<String> {
    std::env::var(key).ok().or_fail()
}
```

---

#### `OrFail` for `core::result::Result<T, E>` where `E: core::error::Error`

```rust
impl<T, E: core::error::Error> orfail::OrFail for core::result::Result<T, E> {
    type Value = T;
    type Error = E;
}
```

Converts standard `Result` types with `Error` trait errors to `Failure`.

**Behavior:**
- `Ok(value).or_fail()` returns `Ok(value)`
- `Err(error).or_fail()` converts the error to a `Failure` using its `Display` implementation

**Example:**

```rust
fn read_file(path: &str) -> orfail::Result<String> {
    std::fs::read_to_string(path).or_fail()
}
```

---

#### `OrFail` for `orfail::Result<T>`

```rust
impl<T> orfail::OrFail for orfail::Result<T> {
    type Value = T;
    type Error = String;
}
```

Allows chaining `or_fail()` calls to build up a backtrace.

**Behavior:**
- `Ok(value).or_fail()` returns `Ok(value)`
- `Err(failure).or_fail()` appends the current caller location to the existing backtrace

**Example:**

```rust
fn validate(s: &str) -> orfail::Result<()> {
    check_length(s).or_fail()?;  // Adds another backtrace entry
    Ok(())
}

fn check_length(s: &str) -> orfail::Result<()> {
    (s.len() > 0).or_fail()
}
```

---

## 3. Examples and Common Patterns

### Basic Usage

#### Simple Boolean Checks

```rust
fn validate_input(value: i32) -> orfail::Result<()> {
    (value >= 0).or_fail()?;
    (value <= 100).or_fail()?;
    Ok(())
}

fn example() {
    match validate_input(-5) {
        Ok(_) => println!("Valid"),
        Err(e) => println!("Error: {}", e),
    }
}
```

#### Option to Result Conversion

```rust
fn find_user(id: u32) -> orfail::Result<String> {
    let users = std::collections::HashMap::from([
        (1, "Alice"),
        (2, "Bob"),
    ]);
    
    users.get(&id)
        .map(|s| s.to_string())
        .or_fail()
}
```

#### Converting Standard Library Errors

```rust
fn parse_config(content: &str) -> orfail::Result<i32> {
    content.trim().parse::<i32>().or_fail()
}

fn load_config(path: &str) -> orfail::Result<i32> {
    let content = std::fs::read_to_string(path).or_fail()?;
    parse_config(&content).or_fail()
}
```

---

### Custom Error Messages

#### Using `or_fail_with` for Context

```rust
fn open_database(path: &str) -> orfail::Result<std::fs::File> {
    std::fs::File::open(path)
        .or_fail_with(|e| format!("Failed to open database at '{}': {}", path, e))
}
```

#### Custom Boolean Failure Messages

```rust
fn check_permissions(user: &str, required: &str) -> orfail::Result<()> {
    has_permission(user, required)
        .or_fail_with(|_| format!("User '{}' lacks '{}' permission", user, required))
}

fn has_permission(user: &str, perm: &str) -> bool {
    // Check logic here
    false
}
```

#### Enriching Option Failures

```rust
fn get_env_var(key: &str) -> orfail::Result<String> {
    std::env::var(key)
        .ok()
        .or_fail_with(|_| format!("Environment variable '{}' not found", key))
}
```

---

### Backtrace Building

#### Multi-Level Error Propagation

```rust
fn level_3() -> orfail::Result<()> {
    false.or_fail()
}

fn level_2() -> orfail::Result<()> {
    level_3().or_fail()
}

fn level_1() -> orfail::Result<()> {
    level_2().or_fail()
}

fn example() {
    match level_1() {
        Err(failure) => {
            // Failure will have 3 locations in backtrace
            assert_eq!(failure.backtrace.len(), 3);
            println!("{}", failure);
        }
        Ok(_) => {}
    }
}
```

#### Conditional Error Path Tracking

```rust
fn process_data(data: &[i32]) -> orfail::Result<i32> {
    let first = data.first().or_fail()?;
    
    if *first < 0 {
        validate_negative(*first).or_fail()?;
    } else {
        validate_positive(*first).or_fail()?;
    }
    
    Ok(*first)
}

fn validate_negative(n: i32) -> orfail::Result<()> {
    (n >= -100).or_fail()
}

fn validate_positive(n: i32) -> orfail::Result<()> {
    (n <= 100).or_fail()
}
```

---

### Error Handling Patterns

#### Early Return Pattern

```rust
fn validate_and_process(input: Option<&str>) -> orfail::Result<String> {
    let s = input.or_fail()?;
    (s.len() > 0).or_fail()?;
    (s.len() < 100).or_fail()?;
    
    Ok(s.to_uppercase())
}
```

#### Result Aggregation

```rust
fn validate_all(values: &[i32]) -> orfail::Result<()> {
    for (i, &value) in values.iter().enumerate() {
        (value >= 0)
            .or_fail_with(|_| format!("Value at index {} is negative: {}", i, value))?;
    }
    Ok(())
}
```

#### Combining with Standard Result Types

```rust
fn read_and_parse(path: &str) -> orfail::Result<Vec<i32>> {
    let content = std::fs::read_to_string(path).or_fail()?;
    
    let numbers: orfail::Result<Vec<i32>> = content
        .lines()
        .map(|line| line.trim().parse::<i32>().or_fail())
        .collect();
    
    numbers.or_fail()
}
```

---

### Advanced Patterns

#### Creating Failures Directly

```rust
fn complex_validation(data: &str) -> orfail::Result<()> {
    if data.is_empty() {
        return Err(orfail::Failure::new("Data cannot be empty"));
    }
    
    if data.len() > 1000 {
        return Err(orfail::Failure::new(format!("Data too large: {} bytes", data.len())));
    }
    
    Ok(())
}
```

#### Inspecting Failure Details

```rust
fn handle_error(result: orfail::Result<()>) {
    match result {
        Ok(_) => println!("Success"),
        Err(failure) => {
            println!("Error: {}", failure.message);
            println!("Occurred at {} location(s):", failure.backtrace.len());
            for (i, loc) in failure.backtrace.iter().enumerate() {
                println!("  {}: {}:{}", i + 1, loc.file, loc.line);
            }
        }
    }
}
```

#### Working with Generic Functions

```rust
fn process_optional<T, F>(opt: Option<T>, processor: F) -> orfail::Result<String>
where
    T: std::fmt::Display,
    F: FnOnce(T) -> String,
{
    let value = opt.or_fail()?;
    Ok(processor(value))
}

fn example() {
    let result = process_optional(Some(42), |n| format!("Number: {}", n));
    assert!(result.is_ok());
}
```

---

### Edge Cases and Error Handling

#### Nested Result Types

```rust
fn nested_operations() -> orfail::Result<i32> {
    let outer: core::result::Result<core::result::Result<i32, std::io::Error>, std::fmt::Error> 
        = Ok(Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found")));
    
    let inner = outer.or_fail()?;
    let value = inner.or_fail()?;
    Ok(value)
}
```

#### Handling Empty Backtraces

```rust
fn direct_failure() -> orfail::Result<()> {
    let failure = orfail::Failure::new("Direct error");
    // Backtrace will have exactly one entry (creation point)
    assert_eq!(failure.backtrace.len(), 1);
    Err(failure)
}
```

#### Converting Between Error Types

```rust
fn convert_error_types(use_std: bool) -> orfail::Result<String> {
    if use_std {
        std::fs::read_to_string("config.txt").or_fail()
    } else {
        Some("default".to_string()).or_fail()
    }
}
```

#### Preserving Context Across Boundaries

```rust
fn boundary_function(data: orfail::Result<String>) -> orfail::Result<usize> {
    let s = data.or_fail()?;  // Adds this location to backtrace
    Ok(s.len())
}

fn caller() -> orfail::Result<usize> {
    let initial: orfail::Result<String> = Err(orfail::Failure::new("initial error"));
    boundary_function(initial)  // Backtrace will have 2 locations
}
```

---

### Working in no_std Contexts

```rust
// This crate works in no_std environments
#![no_std]

extern crate alloc;

fn no_std_example() -> orfail::Result<i32> {
    let value: Option<i32> = None;
    value.or_fail()
}
```
