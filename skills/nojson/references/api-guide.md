# nojson API Guide

## Table of Contents
1. [Core Types](#core-types)
2. [Parsing JSON](#parsing-json)
3. [Generating JSON](#generating-json)
4. [Working with Raw Values](#working-with-raw-values)
5. [Error Handling](#error-handling)

## Core Types

### Json<T>
Wrapper type that enables JSON parsing and generation through `FromStr` and `Display` traits.

```rust
pub struct Json<T>(pub T);
```

**Usage:**
- Parsing: `let value: Json<[Option<u32>; 3]> = "[1, null, 2]".parse()?;`
- Generating: `Json(value).to_string()`

### RawJson<'text>
Parsed JSON text maintaining original form without type conversion. Holds index information about each JSON value.

**Key Methods:**
- `parse(text: &'text str) -> Result<Self, JsonParseError>` - Parse standard JSON
- `parse_jsonc(text: &'text str) -> Result<(Self, Vec<Range<usize>>), JsonParseError>` - Parse JSONC with comments
- `text(&self) -> &'text str` - Get original JSON text
- `value(&self) -> RawJsonValue<'text, '_>` - Get top-level value
- `get_value_by_position(position: usize) -> Option<RawJsonValue<'text, '_>>` - Find value at position
- `into_owned(self) -> RawJsonOwned` - Convert to owned version

### RawJsonOwned
Owned version of RawJson that doesn't borrow from input text.

**Key Methods:**
- `parse<T: Into<String>>(text: T) -> Result<Self, JsonParseError>`
- `parse_jsonc<T: Into<String>>(text: T) -> Result<(Self, Vec<Range<usize>>), JsonParseError>`
- `text(&self) -> &str` - Get owned JSON text
- `value(&self) -> RawJsonValue` - Get top-level value
- `get_value_by_position(position: usize) -> Option<RawJsonValue>`

### RawJsonValue<'text, 'raw>
A JSON value with structural information (kind, position, children).

**Key Methods:**
- `kind(self) -> JsonValueKind` - Get value type (Null, Boolean, Integer, Float, String, Array, Object)
- `position(self) -> usize` - Get byte position in text
- `as_raw_str(self) -> &'text str` - Get raw text as-is
- `as_boolean_str(self) -> Result<&'text str, JsonParseError>`
- `as_integer_str(self) -> Result<&'text str, JsonParseError>`
- `as_float_str(self) -> Result<&'text str, JsonParseError>`
- `as_number_str(self) -> Result<&'text str, JsonParseError>` - Integer or Float
- `to_unquoted_string_str(self) -> Result<Cow<'text, str>, JsonParseError>`
- `to_array(self) -> Result<impl Iterator<Item = RawJsonValue>, JsonParseError>`
- `to_object(self) -> Result<impl Iterator<Item = (RawJsonValue, RawJsonValue)>, JsonParseError>`
- `to_member<'a>(self, name: &'a str) -> Result<RawJsonMember, JsonParseError>`
- `parent(self) -> Option<Self>` - Get containing array/object
- `root(self) -> Self` - Get root value (O(1) operation)
- `extract(self) -> RawJson<'text>` - Create RawJson for this value and its children
- `map<F, T>(self, f: F) -> Result<T, JsonParseError>` where F is a transformation function
- `invalid<E>(self, error: E) -> JsonParseError` where E: Into<Box<dyn Send + Sync + std::error::Error>>

### RawJsonMember<'text, 'raw, 'a>
Result of accessing an object member (may or may not exist).

**Key Methods:**
- `required(self) -> Result<RawJsonValue, JsonParseError>` - Get value or error if missing
- `get(self) -> Option<RawJsonValue>` - Get inner Option
- `map<F, T>(self, f: F) -> Result<Option<T>, JsonParseError>` where F is a transformation

### JsonValueKind
Enum of JSON value types:
- `Null`, `Boolean`, `Integer`, `Float`, `String`, `Array`, `Object`

**Predicates:**
- `is_null()`, `is_bool()`, `is_integer()`, `is_float()`, `is_number()`, `is_string()`, `is_array()`, `is_object()`

### JsonParseError
Enum of parsing errors:
- `UnexpectedEos { kind: Option<JsonValueKind>, position: usize }` - Unexpected end of string
- `UnexpectedTrailingChar { kind: JsonValueKind, position: usize }` - Extra chars after valid JSON
- `UnexpectedValueChar { kind: Option<JsonValueKind>, position: usize }` - Invalid character in value
- `InvalidValue { kind: JsonValueKind, position: usize, error: Box<dyn Error> }` - Custom validation failure

**Key Methods:**
- `kind(&self) -> Option<JsonValueKind>` - Get associated JSON type
- `position(&self) -> usize` - Get error byte position
- `get_line_and_column_numbers(&self, text: &str) -> Option<(NonZeroUsize, NonZeroUsize)>` - Get 1-indexed line:column
- `get_line<'a>(&self, text: &'a str) -> Option<&'a str>` - Get full line containing error

### DisplayJson Trait
Implemented by types that can be formatted as JSON.

```rust
pub trait DisplayJson {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result;
}
```

**Built-in implementations:** All primitives, Option, collections (Vec, HashMap, BTreeMap, sets), references, Box, Rc, Arc, Path, SocketAddr, IpAddr, etc.

### JsonFormatter<'a, 'b>
Controls JSON formatting (indentation, spacing).

**Key Methods:**
- `value<T: DisplayJson>(&mut self, value: T) -> std::fmt::Result` - Format a value
- `string<T: Display>(&mut self, content: T) -> std::fmt::Result` - Format a string
- `array<F>(&mut self, f: F) -> std::fmt::Result` where F creates array content
- `object<F>(&mut self, f: F) -> std::fmt::Result` where F creates object content
- `inner_mut(&mut self) -> &mut std::fmt::Formatter<'b>` - Access wrapped formatter
- `get_level(&self) -> usize` - Current indentation level
- `get_indent_size(&self) -> usize` - Spaces per level
- `set_indent_size(&mut self, size: usize)` - Set spaces per level
- `get_spacing(&self) -> bool` - Whether to add spaces after `:`, `,`, `{`
- `set_spacing(&mut self, enable: bool)` - Toggle spacing

### JsonArrayFormatter<'a, 'b, 'c>
Helper for adding elements to JSON arrays.

**Key Methods:**
- `element<T: DisplayJson>(&mut self, element: T) -> std::fmt::Result`
- `elements<I>(&mut self, elements: I) -> std::fmt::Result` where I: IntoIterator, I::Item: DisplayJson

### JsonObjectFormatter<'a, 'b, 'c>
Helper for adding members to JSON objects.

**Key Methods:**
- `member<N, V>(&mut self, name: N, value: V) -> std::fmt::Result` where N: Display, V: DisplayJson
- `members<I, N, V>(&mut self, members: I) -> std::fmt::Result` where I: IntoIterator<Item = (N, V)>, N: Display, V: DisplayJson

## Parsing JSON

### With Type Conversion
Use `RawJson` to parse, then convert values using `TryFrom`:

```rust
let json = nojson::RawJson::parse(text)?;
let number: u32 = json.value().try_into()?;
let array: Vec<String> = json.value().try_into()?;
let map: HashMap<String, i32> = json.value().try_into()?;
```

**TryFrom implementations** available for all primitives, Option, Vec, arrays, HashMap, BTreeMap, Rc, Arc, Path, IpAddr, SocketAddr, etc.

### Parsing JSONC (with comments and trailing commas)
```rust
let (json, comment_ranges) = nojson::RawJson::parse_jsonc(text)?;
// comment_ranges: Vec<Range<usize>> - byte ranges of comments in original text
```

### Custom Validation
```rust
let json = nojson::RawJson::parse(text)?;
let num: u32 = json.value()
    .as_number_str()?
    .parse()
    .map_err(|e| json.value().invalid(e))?;
    
if num == 0 {
    return Err(json.value().invalid("Expected positive number"));
}
```

## Generating JSON

### Using DisplayJson Trait
Implement for custom types:

```rust
impl DisplayJson for MyType {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("field1", &self.field1)?;
            f.member("field2", &self.field2)
        })
    }
}
```

### Using json() Function
Generate JSON with formatting:

```rust
let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.object(|f| {
        f.member("name", "Alice")?;
        f.member("age", 30)?;
        f.member("items", &["a", "b", "c"])
    })
});
```

### Convenience Functions
```rust
nojson::array(|f| {
    f.element(1)?;
    f.element(2)?;
    f.element(3)
})

nojson::object(|f| {
    f.member("key", "value")
})
```

## Working with Raw Values

### Member Access
```rust
let obj = json.value();
let name_member = obj.to_member("name")?; // RawJsonMember

// Required member
let name: String = name_member.required()?.try_into()?;

// Optional member
let city: Option<String> = obj.to_member("city")?.try_into()?;

// With transformation
let age: Option<u32> = obj.to_member("age")?.map(|v| v.try_into())?;
```

### Array Iteration
```rust
for (i, elem) in json.value().to_array()?.enumerate() {
    let value: u32 = elem.try_into()?;
}
```

### Object Iteration
```rust
for (key, value) in json.value().to_object()? {
    let name = key.to_unquoted_string_str()?;
    let val = value.as_number_str()?;
}
```

### Navigation
```rust
let value = json.value().to_member("user")?.required()?;
let parent = value.parent(); // Contains "user" value
let root = value.root();      // Top-level value
```

### Value Extraction
```rust
// Create RawJson for this value and descendants
let extracted: RawJson = value.extract();
let owned: RawJsonOwned = extracted.into_owned();
```

## Error Handling

### Error Context
```rust
let error: JsonParseError = /* ... */;

// Position information
let pos = error.position();

// Kind of value being parsed
let kind = error.kind();

// Line and column (1-indexed)
if let Some((line, col)) = error.get_line_and_column_numbers(text) {
    println!("Error at line {}, column {}", line.get(), col.get());
}

// Full line content
if let Some(line) = error.get_line(text) {
    println!("Line: {}", line);
}

// Find value at error position
if let Some(value) = json.get_value_by_position(error.position()) {
    println!("Value: {}", value.as_raw_str());
}
```

### Custom Errors
```rust
// From validation failure
return Err(value.invalid("reason"));

// From conversion failure
value.invalid(format_error)
```
