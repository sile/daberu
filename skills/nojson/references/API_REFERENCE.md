# nojson API Reference

Complete reference for all public types and methods in the nojson crate.

## Table of Contents

- [Core Types](#core-types)
- [Parsing Types](#parsing-types)
- [Formatting Types](#formatting-types)
- [Error Types](#error-types)
- [Traits](#traits)
- [Helper Functions](#helper-functions)

## Core Types

### `Json<T>`

Marker struct for JSON parsing and generation through `FromStr` and `Display` traits.

```rust
pub struct Json<T>(pub T);
```

**Methods:**
- Implements `Display` when `T: DisplayJson`
- Implements `FromStr` when `T: TryFrom<RawJsonValue<'_, '_>, Error = JsonParseError>`

**Usage:**
```rust
// Parsing
let value: nojson::Json<Vec<u32>> = "[1,2,3]".parse()?;

// Generation
let json_str = nojson::Json(vec![1, 2, 3]).to_string();
```

### `RawJson<'text>`

Parsed JSON maintaining references to original text with structural information.

```rust
pub struct RawJson<'text> { /* private fields */ }
```

**Methods:**

#### `parse(text: &'text str) -> Result<Self, JsonParseError>`

Parse JSON text into `RawJson`.

```rust
let json = nojson::RawJson::parse(r#"{"key": "value"}"#)?;
```

#### `parse_jsonc(text: &'text str) -> Result<(Self, Vec<Range<usize>>), JsonParseError>`

Parse JSONC (JSON with comments) and return comment locations.

```rust
let (json, comments) = nojson::RawJson::parse_jsonc(r#"{
    "key": "value" // comment
}"#)?;
```

#### `text(&self) -> &'text str`

Return the original JSON text.

```rust
let text = json.text();
```

#### `value(&self) -> RawJsonValue<'text, '_>`

Get the root JSON value.

```rust
let root = json.value();
```

#### `get_value_by_position(&self, position: usize) -> Option<RawJsonValue<'text, '_>>`

Find the most specific value containing the given byte position.

```rust
let value = json.get_value_by_position(42);
```

#### `into_owned(self) -> RawJsonOwned`

Convert to owned version that doesn't borrow from original text.

```rust
let owned = json.into_owned();
```

**Traits:**
- `Display`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Clone`
- `DisplayJson` - Can be used with `Json` wrapper for generation

### `RawJsonOwned`

Owned version of `RawJson` that owns its text.

```rust
pub struct RawJsonOwned { /* private fields */ }
```

**Methods:**

#### `parse<T: Into<String>>(text: T) -> Result<Self, JsonParseError>`

Parse JSON text into owned structure.

```rust
let json = nojson::RawJsonOwned::parse("{\"key\": \"value\"}".to_string())?;
```

#### `parse_jsonc<T: Into<String>>(text: T) -> Result<(Self, Vec<Range<usize>>), JsonParseError>`

Parse JSONC into owned structure.

```rust
let (json, comments) = nojson::RawJsonOwned::parse_jsonc(text.to_string())?;
```

#### `text(&self) -> &str`

Return the JSON text.

#### `value(&self) -> RawJsonValue<'_, '_>`

Get the root value.

#### `get_value_by_position(&self, position: usize) -> Option<RawJsonValue<'_, '_>>`

Find value at byte position.

**Traits:**
- Same as `RawJson` plus `FromStr`

### `RawJsonValue<'text, 'raw>`

A JSON value within a `RawJson` structure.

```rust
pub struct RawJsonValue<'text, 'raw> { /* private fields */ }
```

**Inspection Methods:**

#### `kind(self) -> JsonValueKind`

Return the kind of this value.

```rust
match value.kind() {
    nojson::JsonValueKind::String => println!("It's a string"),
    nojson::JsonValueKind::Array => println!("It's an array"),
    // ...
}
```

#### `position(self) -> usize`

Return byte position in original text.

```rust
let pos = value.position();
```

#### `as_raw_str(self) -> &'text str`

Get raw JSON text for this value.

```rust
let text = value.as_raw_str(); // e.g., "[1,2,3]"
```

**Navigation Methods:**

#### `parent(self) -> Option<Self>`

Get parent value (array or object containing this value).

```rust
if let Some(parent) = value.parent() {
    println!("Parent: {}", parent.as_raw_str());
}
```

#### `root(self) -> Self`

Get root value of the JSON structure.

```rust
let root = nested_value.root();
```

#### `extract(self) -> RawJson<'text>`

Extract this value and its children as standalone `RawJson`.

```rust
let extracted = value.extract();
```

**Type-Specific Accessors:**

#### `as_boolean_str(self) -> Result<&'text str, JsonParseError>`

Get boolean as string, verifying type.

```rust
let bool_str = value.as_boolean_str()?; // "true" or "false"
let bool_val: bool = bool_str.parse()?;
```

#### `as_integer_str(self) -> Result<&'text str, JsonParseError>`

Get integer as string, verifying type.

```rust
let int_str = value.as_integer_str()?;
```

#### `as_float_str(self) -> Result<&'text str, JsonParseError>`

Get float as string, verifying type.

```rust
let float_str = value.as_float_str()?;
```

#### `as_number_str(self) -> Result<&'text str, JsonParseError>`

Get number (integer or float) as string.

```rust
let num_str = value.as_number_str()?;
let num: f64 = num_str.parse().map_err(|e| value.invalid(e))?;
```

#### `to_unquoted_string_str(self) -> Result<Cow<'text, str>, JsonParseError>`

Get unquoted string content, handling escape sequences.

```rust
let s = value.to_unquoted_string_str()?; // Removes quotes, unescapes
```

**Collection Accessors:**

#### `to_array(self) -> Result<impl Iterator<Item = Self>, JsonParseError>`

Get iterator over array elements.

```rust
for element in value.to_array()? {
    println!("{}", element.as_raw_str());
}
```

#### `to_object(self) -> Result<impl Iterator<Item = (Self, Self)>, JsonParseError>`

Get iterator over object key-value pairs.

```rust
for (key, val) in value.to_object()? {
    let k = key.to_unquoted_string_str()?;
    println!("{} = {}", k, val.as_raw_str());
}
```

#### `to_member<'a>(self, name: &'a str) -> Result<RawJsonMember<'text, 'raw, 'a>, JsonParseError>`

Access object member by name.

```rust
let member = value.to_member("age")?;
let age: u32 = member.required()?.try_into()?;
```

**Functional Methods:**

#### `map<F, T>(self, f: F) -> Result<T, JsonParseError>`

Apply transformation function to this value.

```rust
let result = value.map(|v| {
    let s: String = v.try_into()?;
    Ok(s.to_uppercase())
})?;
```

**Error Creation:**

#### `invalid<E>(self, error: E) -> JsonParseError`

Create `InvalidValue` error for this value.

```rust
return Err(value.invalid("Expected positive number"));
```

**Traits:**
- `Display`, `DisplayJson`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Clone`, `Copy`
- `TryFrom<RawJsonValue>` implemented for many standard types

### `RawJsonMember<'text, 'raw, 'a>`

Represents result of object member lookup.

```rust
pub struct RawJsonMember<'text, 'raw, 'a> { /* private fields */ }
```

**Methods:**

#### `required(self) -> Result<RawJsonValue<'text, 'raw>, JsonParseError>`

Get value if member exists, error otherwise.

```rust
let value = obj.to_member("required_field")?.required()?;
```

#### `get(self) -> Option<RawJsonValue<'text, 'raw>>`

Get value as `Option`.

```rust
if let Some(value) = obj.to_member("optional_field")?.get() {
    // Process value
}
```

#### `map<F, T>(self, f: F) -> Result<Option<T>, JsonParseError>`

Transform value if present.

```rust
let opt_value: Option<u32> = obj.to_member("count")?.map(|v| v.try_into())?;
```

**Traits:**
- `TryFrom<RawJsonMember> for Option<T>` where `T: TryFrom<RawJsonValue>`

## Formatting Types

### `JsonFormatter<'a, 'b>`

Formatter for JSON output with customizable formatting.

```rust
pub struct JsonFormatter<'a, 'b> { /* private fields */ }
```

**Value Methods:**

#### `value<T: DisplayJson>(&mut self, value: T) -> std::fmt::Result`

Format a value implementing `DisplayJson`.

```rust
f.value(&my_struct)?;
```

#### `string<T: Display>(&mut self, content: T) -> std::fmt::Result`

Format as JSON string with proper escaping.

```rust
f.string("Hello\nWorld")?; // Outputs: "Hello\nWorld"
```

**Container Methods:**

#### `array<F>(&mut self, f: F) -> std::fmt::Result`

Create JSON array.

```rust
f.array(|f| {
    f.element(1)?;
    f.element(2)?;
    f.element(3)
})?;
```

#### `object<F>(&mut self, f: F) -> std::fmt::Result`

Create JSON object.

```rust
f.object(|f| {
    f.member("name", "Alice")?;
    f.member("age", 30)
})?;
```

**Configuration Methods:**

#### `get_level(&self) -> usize`

Get current nesting level.

```rust
let level = f.get_level();
```

#### `get_indent_size(&self) -> usize`

Get spaces per indentation level.

```rust
let indent = f.get_indent_size();
```

#### `set_indent_size(&mut self, size: usize)`

Set indentation size (affects current and deeper levels).

```rust
f.set_indent_size(2); // 2 spaces per level
```

#### `get_spacing(&self) -> bool`

Check if spacing is enabled.

```rust
let spacing = f.get_spacing();
```

#### `set_spacing(&mut self, enable: bool)`

Enable/disable spacing after `:`, `,`, `{`.

```rust
f.set_spacing(true);
```

#### `inner_mut(&mut self) -> &mut std::fmt::Formatter<'b>`

Access underlying formatter for custom output.

```rust
write!(f.inner_mut(), "custom")?;
```

### `JsonArrayFormatter<'a, 'b, 'c>`

Formatter for building JSON arrays.

**Methods:**

#### `element<T: DisplayJson>(&mut self, element: T) -> std::fmt::Result`

Add single element to array.

```rust
f.element(42)?;
```

#### `elements<I>(&mut self, elements: I) -> std::fmt::Result`

Add multiple elements from iterator.

```rust
f.elements([1, 2, 3])?;
f.elements(vec.iter())?;
```

### `JsonObjectFormatter<'a, 'b, 'c>`

Formatter for building JSON objects.

**Methods:**

#### `member<N, V>(&mut self, name: N, value: V) -> std::fmt::Result`

Add name-value pair.

```rust
f.member("key", "value")?;
```

#### `members<I, N, V>(&mut self, members: I) -> std::fmt::Result`

Add multiple members from iterator.

```rust
f.members(&map)?;
f.members(vec![("a", 1), ("b", 2)])?;
```

## Error Types

### `JsonParseError`

JSON parsing error with position information.

```rust
pub enum JsonParseError {
    UnexpectedEos { kind: Option<JsonValueKind>, position: usize },
    UnexpectedTrailingChar { kind: JsonValueKind, position: usize },
    UnexpectedValueChar { kind: Option<JsonValueKind>, position: usize },
    InvalidValue { kind: JsonValueKind, position: usize, error: Box<dyn Error> },
}
```

**Methods:**

#### `invalid_value<E>(value: RawJsonValue, error: E) -> JsonParseError`

Create `InvalidValue` error.

```rust
let err = nojson::JsonParseError::invalid_value(value, "custom error");
```

#### `kind(&self) -> Option<JsonValueKind>`

Get JSON value kind associated with error.

```rust
if let Some(kind) = err.kind() {
    println!("Error in {:?}", kind);
}
```

#### `position(&self) -> usize`

Get byte position where error occurred.

```rust
let pos = err.position();
```

#### `get_line_and_column_numbers(&self, text: &str) -> Option<(NonZeroUsize, NonZeroUsize)>`

Get line and column numbers for error position.

```rust
if let Some((line, col)) = err.get_line_and_column_numbers(text) {
    eprintln!("Error at line {}, column {}", line, col);
}
```

#### `get_line<'a>(&self, text: &'a str) -> Option<&'a str>`

Get line of text containing error.

```rust
if let Some(line) = err.get_line(text) {
    eprintln!("Error in line: {}", line);
}
```

**Traits:**
- `Display`, `Error`, `Debug`

### `JsonValueKind`

Enum representing JSON value types.

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

All methods are `const`:

- `is_null(self) -> bool`
- `is_bool(self) -> bool`
- `is_integer(self) -> bool`
- `is_float(self) -> bool`
- `is_number(self) -> bool` - Integer or Float
- `is_string(self) -> bool`
- `is_array(self) -> bool`
- `is_object(self) -> bool`

**Traits:**
- `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`

## Traits

### `DisplayJson`

Trait for formatting values as JSON.

```rust
pub trait DisplayJson {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result;
}
```

**Implementation Pattern:**

```rust
impl nojson::DisplayJson for MyType {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("field1", &self.field1)?;
            f.member("field2", self.field2)
        })
    }
}
```

**Blanket Implementations:**

Implemented for:
- References: `&T`, `&mut T`, `Box<T>`, `Rc<T>`, `Arc<T>`
- Primitives: All numeric types, `bool`, `char`, `str`, `String`
- Collections: Arrays, slices, `Vec`, `VecDeque`, `BTreeSet`, `HashSet`, `BTreeMap`, `HashMap`
- Network types: `IpAddr`, `SocketAddr`, etc.
- Path types: `Path`, `PathBuf`
- `Option<T>`, `Cow<'a, T>`, `()`

### `TryFrom<RawJsonValue<'text, 'raw>>`

Standard conversion trait for parsing JSON.

**Implementation Pattern:**

```rust
impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for MyType {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        // Implementation
        Ok(MyType { /* ... */ })
    }
}
```

**Standard Implementations:**

Implemented for all types that have `DisplayJson` implementations, plus:
- Collections can be converted to other collections
- Fixed-size arrays: `[T; N]`
- Maps with parseable keys

## Helper Functions

### `json<F>(f: F) -> impl DisplayJson + Display`

Create JSON with custom formatting.

```rust
let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.value(&data)
});
```

### `object<F>(fmt: F) -> impl DisplayJson + Display`

Shorthand for creating JSON objects.

```rust
let obj = nojson::object(|f| {
    f.member("key", "value")
});
```

### `array<F>(fmt: F) -> impl DisplayJson + Display`

Shorthand for creating JSON arrays.

```rust
let arr = nojson::array(|f| {
    f.element(1)?;
    f.element(2)
});
```

## Type Conversion Reference

### Parsing (TryFrom<RawJsonValue>)

| Rust Type | JSON Type | Notes |
|-----------|-----------|-------|
| `bool` | Boolean | |
| `i8..i128`, `u8..u128`, `isize`, `usize` | Integer | |
| `NonZeroI8..NonZeroI128`, etc. | Integer | Non-zero validation |
| `f32`, `f64` | Integer or Float | |
| `char` | String | Must be single character |
| `String` | String | |
| `Cow<'text, str>` | String | Borrows when possible |
| `PathBuf` | String | |
| `IpAddr`, `Ipv4Addr`, `Ipv6Addr` | String | Parsed from string |
| `SocketAddr`, `SocketAddrV4`, `SocketAddrV6` | String | Parsed from string |
| `Option<T>` | `null` or T's type | `null` → `None` |
| `()` | `null` | Only accepts null |
| `Vec<T>` | Array | |
| `VecDeque<T>` | Array | |
| `[T; N]` | Array | Length validated |
| `BTreeSet<T>`, `HashSet<T>` | Array | |
| `BTreeMap<K, V>`, `HashMap<K, V>` | Object | Keys parsed from strings |
| `Rc<T>`, `Arc<T>` | T's type | Wraps parsed value |
| `RawJson<'text>` | Any | Extracts value as JSON |
| `RawJsonOwned` | Any | Extracts as owned |

### Generation (DisplayJson)

Same types supported for generation as parsing, with JSON output matching expected format.

## Best Practices

### Always Use Fully Qualified Names

```rust
// Correct
fn process(value: nojson::RawJsonValue<'_, '_>) -> Result<(), nojson::JsonParseError> {
    let num: u32 = value.as_integer_str()?.parse().map_err(|e| value.invalid(e))?;
    Ok(())
}

// Incorrect - don't use 'use' statements
use nojson::RawJsonValue;
use nojson::JsonParseError;
```

### Use `.invalid()` for Error Context

```rust
// Correct
let num: u32 = text.parse().map_err(|e| value.invalid(e))?;

// Correct (with custom message)
if num == 0 {
    return Err(value.invalid("number must be positive"));
}
```

### Chain Member Access

```rust
// Access nested fields efficiently
let city: String = json.value()
    .to_member("user")?
    .required()?
    .to_member("address")?
    .required()?
    .to_member("city")?
    .required()?
    .try_into()?;
```

### Handle Optional Members

```rust
// As Option<T>
let email: Option<String> = json.value().to_member("email")?.try_into()?;

// With map
let age: Option<u32> = json.value().to_member("age")?.map(|v| v.try_into())?;

// With default
let port: u16 = json.value()
    .to_member("port")?
    .map(|v| v.try_into())
    .transpose()?
    .unwrap_or(8080);
```

### Format JSON Consistently

```rust
// For debugging/logging - pretty print
let pretty = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.value(&data)
});

// For APIs - compact
let compact = nojson::json(|f| f.value(&data));

// For human-readable config - spaced
let config = nojson::json(|f| {
    f.set_spacing(true);
    f.value(&data)
});
```
