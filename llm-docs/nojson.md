# Nojson Crate Documentation

## Crate Summary

### Overview

`nojson` is a flexible and ergonomic JSON library for Rust that provides a balance between type-safety and the dynamic nature of JSON. Unlike `serde`, which requires strict one-to-one mapping between Rust types and JSON structures, `nojson` offers a toolbox approach that allows developers to mix type-level programming with imperative code flexibility.

### Key Use Cases and Benefits

- **Flexible Type Mapping**: No requirement for strict one-to-one type mapping between Rust structs and JSON
- **Clean Error Messages**: Parsing errors include precise position information with line/column numbers
- **Custom Validation**: Easy addition of application-specific validation rules with proper error context
- **Formatting Control**: Flexible JSON formatting including pretty-printing with customizable indentation
- **Zero Dependencies**: No external crate dependencies and no procedural macros
- **Dual-Level Access**: Both low-level access to raw JSON structure and high-level conveniences for common operations

### Design Philosophy

The crate follows these core principles:

1. **Toolbox over Framework**: Provides building blocks rather than imposing a rigid structure
2. **Type Safety with Flexibility**: Leverages Rust's type system while allowing imperative code where needed
3. **Rich Error Context**: Error messages indicate exact positions in JSON text for easier debugging
4. **Ergonomic API**: Simple cases are simple, complex cases are possible

---

## API Reference

### Core Types

#### `nojson::Json<T>`

A wrapper type that enables JSON parsing and generation through standard Rust traits.

```rust
pub struct Json<T>(pub T)
```

**Trait Implementations:**
- `std::fmt::Display` for types implementing `nojson::DisplayJson`
- `std::str::FromStr` for types implementing `TryFrom<nojson::RawJsonValue<'_, '_>, Error = nojson::JsonParseError>`

**Methods:**
- Inherits `to_string()` from `Display`
- Inherits `parse()` from `FromStr`

---

#### `nojson::RawJson<'text>`

Represents parsed JSON text that is syntactically valid but not yet converted to Rust types.

```rust
pub struct RawJson<'text> { /* private fields */ }
```

**Methods:**

##### `nojson::RawJson::parse`
```rust
pub fn parse(text: &'text str) -> Result<Self, nojson::JsonParseError>
```
Parses a JSON string, validating syntax without converting values to Rust types.

##### `nojson::RawJson::parse_jsonc`
```rust
pub fn parse_jsonc(text: &'text str) -> Result<(Self, Vec<std::ops::Range<usize>>), nojson::JsonParseError>
```
Parses JSONC (JSON with Comments) format. Supports line comments (`//`), block comments (`/* */`), and trailing commas. Returns the parsed JSON and the byte ranges where comments were found.

##### `nojson::RawJson::text`
```rust
pub fn text(&self) -> &'text str
```
Returns the original JSON text.

##### `nojson::RawJson::value`
```rust
pub fn value(&self) -> nojson::RawJsonValue<'text, '_>
```
Returns the top-level JSON value as an entry point for traversal.

##### `nojson::RawJson::get_value_by_position`
```rust
pub fn get_value_by_position(&self, position: usize) -> Option<nojson::RawJsonValue<'text, '_>>
```
Finds the most specific JSON value at the given byte position.

##### `nojson::RawJson::into_owned`
```rust
pub fn into_owned(self) -> nojson::RawJsonOwned
```
Converts to an owned version that doesn't borrow from the original text.

---

#### `nojson::RawJsonOwned`

Owned version of `nojson::RawJson` containing its own copy of the JSON text.

```rust
pub struct RawJsonOwned { /* private fields */ }
```

**Methods:**

##### `nojson::RawJsonOwned::parse`
```rust
pub fn parse<T>(text: T) -> Result<Self, nojson::JsonParseError>
where T: Into<String>
```
Parses a JSON string into an owned instance.

##### `nojson::RawJsonOwned::parse_jsonc`
```rust
pub fn parse_jsonc<T>(text: T) -> Result<(Self, Vec<std::ops::Range<usize>>), nojson::JsonParseError>
where T: Into<String>
```
Parses JSONC into an owned instance.

##### `nojson::RawJsonOwned::text`
```rust
pub fn text(&self) -> &str
```
Returns the JSON text.

##### `nojson::RawJsonOwned::value`
```rust
pub fn value(&self) -> nojson::RawJsonValue<'_, '_>
```
Returns the top-level JSON value.

##### `nojson::RawJsonOwned::get_value_by_position`
```rust
pub fn get_value_by_position(&self, position: usize) -> Option<nojson::RawJsonValue<'_, '_>>
```
Finds the JSON value at the specified position.

---

#### `nojson::RawJsonValue<'text, 'raw>`

Represents a single JSON value within parsed JSON data.

```rust
pub struct RawJsonValue<'text, 'raw> { /* private fields */ }
```

**Methods:**

##### `nojson::RawJsonValue::kind`
```rust
pub fn kind(self) -> nojson::JsonValueKind
```
Returns the kind (type) of this JSON value.

##### `nojson::RawJsonValue::position`
```rust
pub fn position(self) -> usize
```
Returns the byte position where this value begins in the JSON text.

##### `nojson::RawJsonValue::parent`
```rust
pub fn parent(self) -> Option<Self>
```
Returns the parent array or object containing this value.

##### `nojson::RawJsonValue::root`
```rust
pub fn root(self) -> Self
```
Returns the root (top-level) value of the JSON data.

##### `nojson::RawJsonValue::as_raw_str`
```rust
pub fn as_raw_str(self) -> &'text str
```
Returns the raw JSON text of this value as-is.

##### `nojson::RawJsonValue::extract`
```rust
pub fn extract(self) -> nojson::RawJson<'text>
```
Converts this value to a borrowed `nojson::RawJson` containing just this value and its children.

##### `nojson::RawJsonValue::as_boolean_str`
```rust
pub fn as_boolean_str(self) -> Result<&'text str, nojson::JsonParseError>
```
Returns the raw string if this is a boolean value.

##### `nojson::RawJsonValue::as_integer_str`
```rust
pub fn as_integer_str(self) -> Result<&'text str, nojson::JsonParseError>
```
Returns the raw string if this is an integer number.

##### `nojson::RawJsonValue::as_float_str`
```rust
pub fn as_float_str(self) -> Result<&'text str, nojson::JsonParseError>
```
Returns the raw string if this is a floating-point number.

##### `nojson::RawJsonValue::as_number_str`
```rust
pub fn as_number_str(self) -> Result<&'text str, nojson::JsonParseError>
```
Returns the raw string if this is any kind of number.

##### `nojson::RawJsonValue::to_unquoted_string_str`
```rust
pub fn to_unquoted_string_str(self) -> Result<std::borrow::Cow<'text, str>, nojson::JsonParseError>
```
Returns the unquoted string content if this is a string value.

##### `nojson::RawJsonValue::to_array`
```rust
pub fn to_array(self) -> Result<impl Iterator<Item = Self>, nojson::JsonParseError>
```
Returns an iterator over array elements if this is an array.

##### `nojson::RawJsonValue::to_object`
```rust
pub fn to_object(self) -> Result<impl Iterator<Item = (Self, Self)>, nojson::JsonParseError>
```
Returns an iterator over key-value pairs if this is an object.

##### `nojson::RawJsonValue::to_member`
```rust
pub fn to_member<'a>(self, name: &'a str) -> Result<nojson::RawJsonMember<'text, 'raw, 'a>, nojson::JsonParseError>
```
Attempts to access an object member by name. Has O(n) complexity.

##### `nojson::RawJsonValue::map`
```rust
pub fn map<F, T>(self, f: F) -> Result<T, nojson::JsonParseError>
where F: FnOnce(nojson::RawJsonValue<'text, 'raw>) -> Result<T, nojson::JsonParseError>
```
Applies a transformation function to this JSON value.

##### `nojson::RawJsonValue::invalid`
```rust
pub fn invalid<E>(self, error: E) -> nojson::JsonParseError
where E: Into<Box<dyn Send + Sync + std::error::Error>>
```
Creates a `JsonParseError::InvalidValue` error for this value.

---

#### `nojson::RawJsonMember<'text, 'raw, 'a>`

Represents a member access result for a JSON object.

```rust
pub struct RawJsonMember<'text, 'raw, 'a> { /* private fields */ }
```

**Methods:**

##### `nojson::RawJsonMember::required`
```rust
pub fn required(self) -> Result<nojson::RawJsonValue<'text, 'raw>, nojson::JsonParseError>
```
Returns the member value or an error if missing.

##### `nojson::RawJsonMember::get`
```rust
pub fn get(self) -> Option<nojson::RawJsonValue<'text, 'raw>>
```
Returns the inner raw JSON value as an `Option`.

##### `nojson::RawJsonMember::map`
```rust
pub fn map<F, T>(self, f: F) -> Result<Option<T>, nojson::JsonParseError>
where F: FnOnce(nojson::RawJsonValue<'text, 'raw>) -> Result<T, nojson::JsonParseError>
```
Applies a transformation function if the member exists.

---

### Error Types

#### `nojson::JsonParseError`

Represents errors that can occur during JSON parsing.

```rust
pub enum JsonParseError {
    UnexpectedEos { kind: Option<nojson::JsonValueKind>, position: usize },
    UnexpectedTrailingChar { kind: nojson::JsonValueKind, position: usize },
    UnexpectedValueChar { kind: Option<nojson::JsonValueKind>, position: usize },
    InvalidValue { kind: nojson::JsonValueKind, position: usize, error: Box<dyn Send + Sync + std::error::Error> },
}
```

**Variants:**
- `UnexpectedEos`: End of string reached unexpectedly
- `UnexpectedTrailingChar`: Extra non-whitespace characters after valid JSON
- `UnexpectedValueChar`: Unexpected character in JSON value
- `InvalidValue`: Syntactically valid but semantically invalid value

**Methods:**

##### `nojson::JsonParseError::invalid_value`
```rust
pub fn invalid_value<E>(value: nojson::RawJsonValue<'_, '_>, error: E) -> nojson::JsonParseError
where E: Into<Box<dyn Send + Sync + std::error::Error>>
```
Creates an `InvalidValue` error.

##### `nojson::JsonParseError::kind`
```rust
pub fn kind(&self) -> Option<nojson::JsonValueKind>
```
Returns the kind of JSON value associated with the error.

##### `nojson::JsonParseError::position`
```rust
pub fn position(&self) -> usize
```
Returns the byte position where the error occurred.

##### `nojson::JsonParseError::get_line_and_column_numbers`
```rust
pub fn get_line_and_column_numbers(&self, text: &str) -> Option<(std::num::NonZeroUsize, std::num::NonZeroUsize)>
```
Returns the line and column numbers for the error position.

##### `nojson::JsonParseError::get_line`
```rust
pub fn get_line<'a>(&self, text: &'a str) -> Option<&'a str>
```
Returns the line of text containing the error.

---

### Value Kind Enum

#### `nojson::JsonValueKind`

Represents the type of a JSON value.

```rust
pub enum JsonValueKind {
    Null,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Object,
}
```

**Methods:**
- `is_null(self) -> bool`
- `is_bool(self) -> bool`
- `is_integer(self) -> bool`
- `is_float(self) -> bool`
- `is_number(self) -> bool`
- `is_string(self) -> bool`
- `is_array(self) -> bool`
- `is_object(self) -> bool`

---

### Formatting Types and Traits

#### `nojson::DisplayJson`

A trait for formatting Rust types as JSON.

```rust
pub trait DisplayJson {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result;
}
```

Implemented for:
- Primitives: `bool`, integers (`i8`-`i128`, `u8`-`u128`, `isize`, `usize`), floats (`f32`, `f64`)
- Non-zero integers: `std::num::NonZero*` types
- Characters and strings: `char`, `str`, `String`, `std::borrow::Cow<str>`
- Collections: arrays, `Vec`, `VecDeque`, `BTreeSet`, `HashSet`
- Maps: `BTreeMap`, `HashMap`
- Smart pointers: `Box`, `Rc`, `Arc`, references
- Optional values: `Option<T>`
- Unit type: `()`
- Network types: `IpAddr`, `SocketAddr`, and variants
- Paths: `Path`, `PathBuf`

---

#### `nojson::JsonFormatter<'a, 'b>`

Controls layout and formatting of JSON output.

```rust
pub struct JsonFormatter<'a, 'b> { /* private fields */ }
```

**Methods:**

##### `nojson::JsonFormatter::value`
```rust
pub fn value<T: nojson::DisplayJson>(&mut self, value: T) -> std::fmt::Result
```
Formats a value implementing `DisplayJson`.

##### `nojson::JsonFormatter::string`
```rust
pub fn string<T: std::fmt::Display>(&mut self, content: T) -> std::fmt::Result
```
Formats a value as a JSON string with proper escaping.

##### `nojson::JsonFormatter::array`
```rust
pub fn array<F>(&mut self, f: F) -> std::fmt::Result
where F: FnOnce(&mut nojson::JsonArrayFormatter<'a, 'b, '_>) -> std::fmt::Result
```
Creates a JSON array.

##### `nojson::JsonFormatter::object`
```rust
pub fn object<F>(&mut self, f: F) -> std::fmt::Result
where F: FnOnce(&mut nojson::JsonObjectFormatter<'a, 'b, '_>) -> std::fmt::Result
```
Creates a JSON object.

##### `nojson::JsonFormatter::inner_mut`
```rust
pub fn inner_mut(&mut self) -> &mut std::fmt::Formatter<'b>
```
Returns mutable reference to the inner formatter.

##### `nojson::JsonFormatter::get_level`
```rust
pub fn get_level(&self) -> usize
```
Returns the current indentation level.

##### `nojson::JsonFormatter::get_indent_size`
```rust
pub fn get_indent_size(&self) -> usize
```
Returns the number of spaces per indentation level.

##### `nojson::JsonFormatter::set_indent_size`
```rust
pub fn set_indent_size(&mut self, size: usize)
```
Sets the number of spaces per indentation level.

##### `nojson::JsonFormatter::get_spacing`
```rust
pub fn get_spacing(&self) -> bool
```
Returns whether spacing after `:`, `,`, and `{` is enabled.

##### `nojson::JsonFormatter::set_spacing`
```rust
pub fn set_spacing(&mut self, enable: bool)
```
Sets whether to insert spaces after `:`, `,`, and `{`.

---

#### `nojson::JsonArrayFormatter<'a, 'b, 'c>`

Formatter for JSON arrays.

```rust
pub struct JsonArrayFormatter<'a, 'b, 'c> { /* private fields */ }
```

**Methods:**

##### `nojson::JsonArrayFormatter::element`
```rust
pub fn element<T: nojson::DisplayJson>(&mut self, element: T) -> std::fmt::Result
```
Adds a single element to the array.

##### `nojson::JsonArrayFormatter::elements`
```rust
pub fn elements<I>(&mut self, elements: I) -> std::fmt::Result
where
    I: IntoIterator,
    I::Item: nojson::DisplayJson
```
Adds multiple elements from an iterator.

---

#### `nojson::JsonObjectFormatter<'a, 'b, 'c>`

Formatter for JSON objects.

```rust
pub struct JsonObjectFormatter<'a, 'b, 'c> { /* private fields */ }
```

**Methods:**

##### `nojson::JsonObjectFormatter::member`
```rust
pub fn member<N, V>(&mut self, name: N, value: V) -> std::fmt::Result
where
    N: std::fmt::Display,
    V: nojson::DisplayJson
```
Adds a single name-value pair.

##### `nojson::JsonObjectFormatter::members`
```rust
pub fn members<I, N, V>(&mut self, members: I) -> std::fmt::Result
where
    I: IntoIterator<Item = (N, V)>,
    N: std::fmt::Display,
    V: nojson::DisplayJson
```
Adds multiple members from an iterator.

---

### Convenience Functions

#### `nojson::json`

```rust
pub fn json<F>(f: F) -> impl nojson::DisplayJson + std::fmt::Display
where F: Fn(&mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result
```

Creates a displayable JSON value with custom formatting.

#### `nojson::object`

```rust
pub fn object<F>(fmt: F) -> impl nojson::DisplayJson + std::fmt::Display
where F: Fn(&mut nojson::JsonObjectFormatter<'_, '_, '_>) -> std::fmt::Result
```

Shorthand for creating JSON objects.

#### `nojson::array`

```rust
pub fn array<F>(fmt: F) -> impl nojson::DisplayJson + std::fmt::Display
where F: Fn(&mut nojson::JsonArrayFormatter<'_, '_, '_>) -> std::fmt::Result
```

Shorthand for creating JSON arrays.

---

### Type Conversions

Many standard types implement `TryFrom<nojson::RawJsonValue<'_, '_>>`:

- Primitives: `bool`, integers, floats, `char`, `String`
- Non-zero integers: all `NonZero*` types
- Collections: `Vec`, `VecDeque`, `BTreeSet`, `HashSet`, fixed-size arrays
- Maps: `BTreeMap`, `HashMap`
- Smart pointers: `Rc`, `Arc`
- Optional: `Option<T>` (maps JSON null to None)
- Network types: `IpAddr`, `SocketAddr` and variants
- Paths: `PathBuf`
- Unit: `()` (from JSON null)
- String borrowing: `std::borrow::Cow<'text, str>`

---

## Examples and Common Patterns

### Basic Parsing

```rust
// Parse primitive types
let text = "42";
let json = nojson::RawJson::parse(text).expect("valid JSON");
let num: i32 = json.value().try_into().expect("valid integer");
assert_eq!(num, 42);

// Parse using Json wrapper
let value: nojson::Json<i32> = "42".parse().expect("valid");
assert_eq!(value.0, 42);
```

### Parsing Arrays

```rust
// Fixed-size array
let text = "[1, 2, 3]";
let json = nojson::RawJson::parse(text).expect("valid");
let arr: [i32; 3] = json.value().try_into().expect("valid array");
assert_eq!(arr, [1, 2, 3]);

// Dynamic vector
let vec: Vec<i32> = json.value().try_into().expect("valid");
assert_eq!(vec, vec![1, 2, 3]);

// Iterate manually
for (i, element) in json.value().to_array().expect("array").enumerate() {
    let num: i32 = element.try_into().expect("number");
    assert_eq!(num as usize, i + 1);
}
```

### Parsing Objects

```rust
let text = r#"{"name": "Alice", "age": 30, "active": true}"#;
let json = nojson::RawJson::parse(text).expect("valid");

// Access specific members
let name: String = json.value()
    .to_member("name").expect("object")
    .required().expect("name exists")
    .try_into().expect("string");
assert_eq!(name, "Alice");

// Handle optional members
let email_member = json.value().to_member("email").expect("object");
let email: Option<String> = email_member.try_into().expect("valid");
assert_eq!(email, None);

// Iterate all members
for (key, value) in json.value().to_object().expect("object") {
    let key_str = key.to_unquoted_string_str().expect("string");
    match key_str.as_ref() {
        "name" => {
            let val: String = value.try_into().expect("string");
            assert_eq!(val, "Alice");
        }
        "age" => {
            let val: i32 = value.try_into().expect("number");
            assert_eq!(val, 30);
        }
        _ => {}
    }
}
```

### Custom Type Conversion

```rust
struct Person {
    name: String,
    age: u32,
    email: Option<String>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Person {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let name = value.to_member("name")?.required()?;
        let age = value.to_member("age")?.required()?;
        let email = value.to_member("email")?;
        
        Ok(Person {
            name: name.try_into()?,
            age: age.try_into()?,
            email: email.try_into()?,
        })
    }
}

// Usage
let text = r#"{"name": "Bob", "age": 25}"#;
let person: nojson::Json<Person> = text.parse().expect("valid");
assert_eq!(person.0.name, "Bob");
assert_eq!(person.0.email, None);
```

### Generating JSON

```rust
// Using Json wrapper with DisplayJson trait
let data = vec![1, 2, 3];
let json_str = nojson::Json(&data).to_string();
assert_eq!(json_str, "[1,2,3]");

// Using json() function for custom formatting
let pretty = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.array(|f| {
        f.element(1)?;
        f.element(2)?;
        f.element(3)
    })
});
// Output: [\n  1,\n  2,\n  3\n]

// Building objects
let obj = nojson::object(|f| {
    f.member("name", "Charlie")?;
    f.member("age", 35)?;
    f.member("scores", &[95, 87, 92])
});
```

### Custom DisplayJson Implementation

```rust
struct Person {
    name: String,
    age: u32,
}

impl nojson::DisplayJson for Person {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("name", &self.name)?;
            f.member("age", self.age)
        })
    }
}

let person = Person { name: "Diana".to_string(), age: 28 };
let json_str = nojson::Json(&person).to_string();
assert_eq!(json_str, r#"{"name":"Diana","age":28}"#);
```

### Custom Validation

```rust
fn parse_positive_number(text: &str) -> Result<u32, nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let value = json.value();
    
    let num: u32 = value.as_number_str()?
        .parse()
        .map_err(|e| value.invalid(e))?;
    
    if num == 0 {
        return Err(value.invalid("Expected a positive number, got 0"));
    }
    
    Ok(num)
}

// Usage
assert!(parse_positive_number("0").is_err());
assert_eq!(parse_positive_number("42").unwrap(), 42);
```

### Error Handling with Context

```rust
let text = r#"{"value": 123invalid}"#;
let result = nojson::RawJson::parse(text);

if let Err(error) = result {
    // Get position information
    let position = error.position();
    
    // Get line and column
    if let Some((line, column)) = error.get_line_and_column_numbers(text) {
        println!("Error at line {}, column {}", line, column);
    }
    
    // Get the problematic line
    if let Some(line_text) = error.get_line(text) {
        println!("Line content: {}", line_text);
    }
    
    // Get the specific value that caused the error
    let json_partial = nojson::RawJson::parse(r#"{"value": 123}"#).unwrap();
    if let Some(problem_value) = json_partial.get_value_by_position(position) {
        println!("Problem value kind: {:?}", problem_value.kind());
    }
}
```

### Working with Nested Structures

```rust
let text = r#"{
    "user": {
        "name": "Eve",
        "address": {
            "city": "Boston",
            "zip": "02101"
        }
    },
    "timestamp": 1234567890
}"#;

let json = nojson::RawJson::parse(text).expect("valid");

// Navigate nested structure
let city: String = json.value()
    .to_member("user").expect("object")
    .required().expect("user exists")
    .to_member("address").expect("object")
    .required().expect("address exists")
    .to_member("city").expect("object")
    .required().expect("city exists")
    .try_into().expect("string");
    
assert_eq!(city, "Boston");

// Navigate back to root from nested value
let zip_value = json.value()
    .to_member("user").expect("object")
    .required().expect("exists")
    .to_member("address").expect("object")
    .required().expect("exists")
    .to_member("zip").expect("object")
    .required().expect("exists");

let root = zip_value.root();
let timestamp: i64 = root
    .to_member("timestamp").expect("object")
    .required().expect("exists")
    .try_into().expect("number");
assert_eq!(timestamp, 1234567890);
```

### Extracting Subtrees

```rust
let text = r#"{"config": {"debug": true, "level": 5}, "version": 2}"#;
let json = nojson::RawJson::parse(text).expect("valid");

// Extract just the config subtree
let config_value = json.value()
    .to_member("config").expect("object")
    .required().expect("exists");

let config_json = config_value.extract();
assert_eq!(config_json.text(), r#"{"debug": true, "level": 5}"#);

// The extracted JSON can be used independently
let debug: bool = config_json.value()
    .to_member("debug").expect("object")
    .required().expect("exists")
    .try_into().expect("bool");
assert_eq!(debug, true);
```

### JSONC (JSON with Comments)

```rust
let text = r#"{
    "name": "Frank", // User's name
    "age": 30,
    /* 
     * Multi-line comment
     * explaining configuration
     */
    "active": true,
}"#; // Trailing comma allowed

let (json, comment_ranges) = nojson::RawJson::parse_jsonc(text).expect("valid JSONC");

// Parse normally
let name: String = json.value()
    .to_member("name").expect("object")
    .required().expect("exists")
    .try_into().expect("string");
assert_eq!(name, "Frank");

// Access comment positions
assert_eq!(comment_ranges.len(), 2);
let first_comment = &text[comment_ranges[0].clone()];
assert!(first_comment.contains("User's name"));
```

### Handling Optional vs Required Fields

```rust
struct Config {
    host: String,
    port: u16,
    debug: bool, // Default to false if missing
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Config {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let host = value.to_member("host")?.required()?;
        let port = value.to_member("port")?.required()?;
        
        // Optional field with default
        let debug = value.to_member("debug")?
            .map(|v| v.try_into())
            .transpose()?
            .unwrap_or(false);
        
        Ok(Config {
            host: host.try_into()?,
            port: port.try_into()?,
            debug,
        })
    }
}

// Without debug field
let config: Config = nojson::RawJson::parse(r#"{"host":"localhost","port":8080}"#)
    .expect("valid").value().try_into().expect("valid config");
assert_eq!(config.debug, false);

// With debug field
let config: Config = nojson::RawJson::parse(r#"{"host":"localhost","port":8080,"debug":true}"#)
    .expect("valid").value().try_into().expect("valid config");
assert_eq!(config.debug, true);
```
