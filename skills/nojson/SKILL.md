---
name: nojson
description: |
  Flexible JSON library for Rust with no dependencies. Supports strong typing, custom validation, JSONC with comments, and flexible formatting. Use when working with JSON parsing, validation, generation, or transformation. Use this skill when Claude needs to: parse JSON text into Rust types, generate JSON from Rust values, work with raw JSON values preserving original text, implement custom JSON validation, handle JSONC format with comments, format JSON with custom indentation/spacing, or provide rich error context for JSON parsing failures.
---

# nojson Skill

nojson is a no-dependency, no-macro JSON library for Rust that balances type-safety with the dynamic nature of JSON. Unlike serde, which requires strict one-to-one type mapping, nojson provides a flexible toolbox approach.

## Quick Start

### Parse JSON to Rust Types

Use qualified names like `nojson::Json`, `nojson::RawJson`, `nojson::RawJsonValue`:

```rust
use nojson::Json;

fn main() -> Result<(), nojson::JsonParseError> {
    let text = "[1, null, 2]";
    let value: Json<[Option<u32>; 3]> = text.parse()?;
    assert_eq!(value.0, [Some(1), None, Some(2)]);
    Ok(())
}
```

### Generate JSON from Rust Types

```rust
use nojson::Json;

let array = [Some(1), None, Some(2)];
assert_eq!(Json(array).to_string(), "[1,null,2]");
```

### Work with Raw JSON Values

Parse JSON while preserving original text:

```rust
use nojson::RawJson;

let json = nojson::RawJson::parse(r#"{"name":"Alice","age":30}"#)?;
let name: String = json.value()
    .to_member("name")?.required()?
    .try_into()?;
```

### Pretty-Print JSON

```rust
use nojson::json;

let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.object(|f| {
        f.member("name", "Alice")?;
        f.member("age", 30)
    })
});
```

## Core Patterns

### Parse with Type Conversion

Use `nojson::RawJson::parse()` to parse, then `TryFrom<nojson::RawJsonValue<'_, '_>>` to convert:

```rust
let json = nojson::RawJson::parse(text)?;

// Scalar types
let num: u32 = json.value().try_into()?;
let text: String = json.value().try_into()?;

// Collections
let array: Vec<String> = json.value().try_into()?;
let map: std::collections::HashMap<String, i32> = json.value().try_into()?;

// Custom types
impl<'text, 'raw> std::convert::TryFrom<nojson::RawJsonValue<'text, 'raw>> for MyType {
    type Error = nojson::JsonParseError;
    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        // Implementation
    }
}
let custom: MyType = json.value().try_into()?;
```

### Access Object Members

```rust
let member = value.to_member("field_name")?;

// Required member
let required_value = member.required()?;

// Optional member
let optional: Option<String> = member.try_into()?;

// With transformation
let transformed: Option<u32> = member.map(|v| v.try_into())?;
```

### Iterate Collections

**Arrays:**
```rust
for item in json.value().to_array()? {
    let parsed: u32 = item.try_into()?;
}
```

**Objects:**
```rust
for (key, value) in json.value().to_object()? {
    let name = key.to_unquoted_string_str()?;
    let val: String = value.try_into()?;
}
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

### Generate JSON from Custom Types

Implement `nojson::DisplayJson` for your types:

```rust
use nojson::{DisplayJson, JsonFormatter};

impl DisplayJson for MyType {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("field1", &self.field1)?;
            f.member("field2", &self.field2)
        })
    }
}

// Now you can use it
let obj = MyType { /* ... */ };
assert_eq!(nojson::Json(&obj).to_string(), r#"{"field1":"...","field2":"..."}"#);
```

### Format JSON Output

Use `nojson::json()` to control formatting:

```rust
let output = nojson::json(|f| {
    f.set_indent_size(4);      // 4 spaces per level
    f.set_spacing(true);        // Add spaces after colons/commas
    f.object(|f| {
        f.member("key", "value")?;
        f.member("array", &[1, 2, 3])
    })
});
```

### Handle JSONC (with Comments)

Parse JSON that includes comments and trailing commas:

```rust
let (json, comment_ranges) = nojson::RawJson::parse_jsonc(text)?;

// comment_ranges: Vec<Range<usize>> - byte positions of comments

for range in comment_ranges {
    let comment_text = &text[range];
    println!("Found comment: {}", comment_text);
}
```

### Error Context and Debugging

```rust
match nojson::RawJson::parse(text) {
    Err(error) => {
        // Get error position
        let pos = error.position();
        
        // Get line and column (1-indexed)
        if let Some((line, col)) = error.get_line_and_column_numbers(text) {
            println!("Error at line {}, column {}", line.get(), col.get());
        }
        
        // Get the full line containing the error
        if let Some(line) = error.get_line(text) {
            println!("Line: {}", line);
        }
        
        // Get the specific value where error occurred
        if let Some(value) = RawJson::parse(text)
            .ok()
            .and_then(|j| j.get_value_by_position(error.position())) {
            println!("Value: {}", value.as_raw_str());
        }
    }
    Ok(_) => { /* ... */ }
}
```

### Navigate JSON Structure

```rust
// Get parent (containing array/object)
let parent = value.parent();

// Get root (top-level value) - O(1) operation
let root = value.root();

// Find value at byte position
if let Some(value) = json.get_value_by_position(position) {
    // ...
}

// Extract substructure as separate RawJson
let subtree: nojson::RawJson = value.extract();
```

## Key Types Reference

See `references/api-guide.md` for complete type documentation:
- `nojson::Json<T>` - Wrapper for parsing/displaying types
- `nojson::RawJson<'text>` - Parsed JSON preserving original text
- `nojson::RawJsonOwned` - Owned version of RawJson
- `nojson::RawJsonValue<'text, 'raw>` - A JSON value with structural info
- `nojson::RawJsonMember<'text, 'raw, 'a>` - Object member access result
- `nojson::JsonValueKind` - Enum: Null, Boolean, Integer, Float, String, Array, Object
- `nojson::JsonParseError` - Comprehensive error type with position info
- `nojson::DisplayJson` - Trait for JSON generation
- `nojson::JsonFormatter<'a, 'b>` - Formatting control
- `nojson::JsonArrayFormatter<'a, 'b, 'c>` - Array builder
- `nojson::JsonObjectFormatter<'a, 'b, 'c>` - Object builder

## Common Usage Examples

See `references/examples.md` for detailed examples including:
- Parsing arrays, objects, nested structures
- Type conversions and custom validations
- Generating JSON with formatting
- Working with raw values and JSONC
- Stream processing and polymorphic JSON
- Error handling with context

## Type Conversion Support

Automatic `TryFrom<nojson::RawJsonValue>` implementations available for:

**Scalars:** bool, i8-i128, u8-u128, isize, usize, f32, f64, char, String

**NonZero types:** NonZeroI8, NonZeroU8, NonZeroI16, NonZeroU16, NonZeroI32, NonZeroU32, NonZeroI64, NonZeroU64, NonZeroI128, NonZeroU128, NonZeroIsize, NonZeroUsize

**Collections:** Vec<T>, VecDeque<T>, BTreeSet<T>, HashSet<T>, BTreeMap<K, V>, HashMap<K, V>, [T; N] (fixed-size arrays)

**References:** Box<T>, Rc<T>, Arc<T>, Option<T>

**Standard types:** Path, PathBuf, IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6

**Unit type:** () (null)

## Important Notes

1. **Use fully qualified names** when writing code - include crate name like `nojson::RawJson`, not just `RawJson`

2. **Lifetime parameters** - `RawJsonValue<'text, 'raw>` has two lifetimes: `'text` for input text, `'raw` for value indices

3. **No macros** - nojson uses no procedural macros, only standard trait implementations

4. **Zero dependencies** - nojson depends only on Rust's standard library

5. **JSONC support** - Use `parse_jsonc()` for JSON with comments and trailing commas

6. **Custom validation** - Use `value.invalid(error)` to create validation errors with proper error context

7. **Performance** - `value.root()` is O(1), but `value.parent()` requires traversal. `to_member()` is O(n) in object size.

8. **Memory** - `RawJson` borrows from input text. Use `RawJsonOwned` or `into_owned()` for owned versions.
