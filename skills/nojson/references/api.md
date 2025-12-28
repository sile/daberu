# nojson API Reference

## Core Types

### `nojson::Json<T>`
Wrapper for parsing JSON text to Rust types and generating JSON from Rust types.
- Implements `Display` for JSON generation when `T: DisplayJson`
- Implements `FromStr` for JSON parsing when `T: TryFrom<RawJsonValue<'_, '_>, Error = JsonParseError>`

### `nojson::RawJson<'text>`
Parsed JSON text (syntactically validated, not converted to Rust types).
- `RawJson::parse(text: &'text str) -> Result<Self, JsonParseError>` - Parse JSON
- `RawJson::parse_jsonc(text: &'text str) -> Result<(Self, Vec<Range<usize>>), JsonParseError>` - Parse JSONC (JSON with comments and trailing commas)
- `text(&self) -> &'text str` - Get original JSON text
- `value(&self) -> RawJsonValue<'text, '_>` - Get top-level value
- `get_value_by_position(&self, position: usize) -> Option<RawJsonValue<'text, '_>>` - Find value at byte position
- `into_owned(self) -> RawJsonOwned` - Convert to owned version

### `nojson::RawJsonOwned`
Owned version of RawJson (contains owned String).
- `RawJsonOwned::parse<T: Into<String>>(text: T) -> Result<Self, JsonParseError>`
- `RawJsonOwned::parse_jsonc<T: Into<String>>(text: T) -> Result<(Self, Vec<Range<usize>>), JsonParseError>`
- Same methods as RawJson but with owned lifetime

### `nojson::RawJsonValue<'text, 'raw>`
Individual JSON value within RawJson.

**Type checking:**
- `kind(&self) -> JsonValueKind` - Get value type
- `as_raw_str(self) -> &'text str` - Raw JSON text
- `as_boolean_str(self) -> Result<&'text str, JsonParseError>` - Verify boolean, return text
- `as_integer_str(self) -> Result<&'text str, JsonParseError>` - Verify integer, return text
- `as_float_str(self) -> Result<&'text str, JsonParseError>` - Verify float, return text
- `as_number_str(self) -> Result<&'text str, JsonParseError>` - Verify number, return text
- `to_unquoted_string_str(self) -> Result<Cow<'text, str>, JsonParseError>` - Verify string, return unquoted

**Navigation:**
- `position(self) -> usize` - Byte position in text
- `parent(self) -> Option<Self>` - Parent value (array/object)
- `root(self) -> Self` - Root value
- `extract(self) -> RawJson<'text>` - Extract value and children as RawJson

**Arrays:**
- `to_array(self) -> Result<impl Iterator<Item = Self>, JsonParseError>` - Iterate array elements

**Objects:**
- `to_object(self) -> Result<impl Iterator<Item = (Self, Self)>, JsonParseError>` - Iterate (key, value) pairs
- `to_member<'a>(self, name: &'a str) -> Result<RawJsonMember<'text, 'raw, 'a>, JsonParseError>` - Access member by name

**Utilities:**
- `map<F, T>(self, f: F) -> Result<T, JsonParseError>` - Transform value
- `invalid<E>(self, error: E) -> JsonParseError` - Create InvalidValue error

### `nojson::RawJsonMember<'text, 'raw, 'a>`
Result of member access on JSON object.
- `required(self) -> Result<RawJsonValue<'text, 'raw>, JsonParseError>` - Get value or error if missing
- `get(self) -> Option<RawJsonValue<'text, 'raw>>` - Get optional value
- `map<F, T>(self, f: F) -> Result<Option<T>, JsonParseError>` - Transform if present

### `nojson::JsonValueKind`
Enum for JSON value types: `Null`, `Boolean`, `Integer`, `Float`, `String`, `Array`, `Object`.

Methods: `is_null()`, `is_bool()`, `is_integer()`, `is_float()`, `is_number()`, `is_string()`, `is_array()`, `is_object()`

### `nojson::JsonParseError`
Error type for JSON parsing.

**Variants:**
- `UnexpectedEos { kind: Option<JsonValueKind>, position: usize }` - Unexpected end of string
- `UnexpectedTrailingChar { kind: JsonValueKind, position: usize }` - Extra characters after JSON
- `UnexpectedValueChar { kind: Option<JsonValueKind>, position: usize }` - Invalid character
- `InvalidValue { kind: JsonValueKind, position: usize, error: Box<dyn Error> }` - Valid syntax but invalid value

**Methods:**
- `invalid_value<E>(value: RawJsonValue, error: E) -> JsonParseError` - Create InvalidValue error
- `kind(&self) -> Option<JsonValueKind>` - Get JSON value kind
- `position(&self) -> usize` - Get byte position
- `get_line_and_column_numbers(&self, text: &str) -> Option<(NonZeroUsize, NonZeroUsize)>` - Get line/column
- `get_line<'a>(&self, text: &'a str) -> Option<&'a str>` - Get error line text

## JSON Generation

### `nojson::DisplayJson` trait
Trait for formatting Rust types as JSON (alternative to std::fmt::Display).

```rust
trait DisplayJson {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result;
}
```

### `nojson::JsonFormatter<'a, 'b>`
Formatter for JSON output with customizable indentation and spacing.

**Value formatting:**
- `value<T: DisplayJson>(&mut self, value: T) -> std::fmt::Result` - Format any value
- `string<T: Display>(&mut self, content: T) -> std::fmt::Result` - Format string with escaping

**Structured formatting:**
- `array<F>(&mut self, f: F) -> std::fmt::Result` - Create array with callback
- `object<F>(&mut self, f: F) -> std::fmt::Result` - Create object with callback

**Configuration:**
- `get_level(&self) -> usize` / `set_indent_size(&mut self, size: usize)` - Indentation spaces per level
- `get_spacing(&self) -> bool` / `set_spacing(&mut self, enable: bool)` - Space after `:`, `,`, `{`
- `inner_mut(&mut self) -> &mut std::fmt::Formatter<'b>` - Direct access to inner formatter

### `nojson::JsonArrayFormatter<'a, 'b, 'c>`
Formatter for array contents (from `JsonFormatter::array()`).
- `element<T: DisplayJson>(&mut self, element: T) -> std::fmt::Result` - Add single element
- `elements<I>(&mut self, elements: I) -> std::fmt::Result` - Add multiple elements from iterator

### `nojson::JsonObjectFormatter<'a, 'b, 'c>`
Formatter for object contents (from `JsonFormatter::object()`).
- `member<N: Display, V: DisplayJson>(&mut self, name: N, value: V) -> std::fmt::Result` - Add name-value pair
- `members<I, N, V>(&mut self, members: I) -> std::fmt::Result` - Add multiple members from iterator

### Convenience functions

**`nojson::json<F>(f: F) -> impl DisplayJson + Display`**
Create JSON with custom formatting.
```rust
let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.value([1, 2, 3])
});
```

**`nojson::object<F>(f: F) -> impl DisplayJson + Display`**
Shorthand for `json(|f| f.object(|f| fmt(f)))`.

**`nojson::array<F>(f: F) -> impl DisplayJson + Display`**
Shorthand for `json(|f| f.array(|f| fmt(f)))`.

## Built-in Implementations

### TryFrom<RawJsonValue> implementations
- Primitives: `bool`, `i8`-`i128`, `u8`-`u128`, `isize`, `usize`, `f32`, `f64`, `char`
- NonZero types: `NonZeroI8`, `NonZeroU8`, etc.
- Strings: `String`, `Cow<'text, str>`
- Containers: `Vec<T>`, `[T; N]`, `VecDeque<T>`, `BTreeSet<T>`, `HashSet<T>`
- Maps: `BTreeMap<K, V>`, `HashMap<K, V>` (K must implement FromStr)
- Smart pointers: `Box<T>`, `Rc<T>`, `Arc<T>`, `Option<T>`
- Network: `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6`
- Path: `PathBuf`
- Special: `()` for null, `RawJsonOwned`, `RawJson`

### DisplayJson implementations
- Primitives: `bool`, integers, floats (non-finite as null), `char`
- Strings: `str`, `String`, `Cow<'_, str>`
- Containers: `[T]`, `[T; N]`, `Vec<T>`, `VecDeque<T>`, `BTreeSet<T>`, `HashSet<T>`
- Maps: `BTreeMap<K, V>`, `HashMap<K, V>` (K must implement Display)
- Smart pointers: `&T`, `&mut T`, `Box<T>`, `Rc<T>`, `Arc<T>`, `Option<T>`
- Network: `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6`
- Path: `Path`, `PathBuf`
- Special: `()` as null, `RawJsonOwned`, `RawJson`, `RawJsonValue`
