---
name: nojson-rust
description: Flexible Rust JSON library (nojson crate) for parsing and generating JSON without dependencies or macros. Use when working with JSON in Rust and need (1) parsing JSON text into Rust types with TryFrom, (2) generating JSON from Rust types with DisplayJson trait, (3) custom validation during parsing, (4) low-level JSON access without strict type mapping, (5) pretty-printing with custom formatting, (6) JSONC support (JSON with comments and trailing commas), (7) precise error messages with position context, or (8) flexible mix of type-safe and imperative JSON handling.
---

# nojson Rust Crate

## Overview

The `nojson` crate provides flexible JSON parsing and generation for Rust without dependencies or macros. Unlike serde's strict one-to-one type mapping, nojson offers a toolbox approach mixing type-level programming with imperative flexibility.

**Key capabilities:**
- Parse JSON to Rust types via `TryFrom<nojson::RawJsonValue>`
- Generate JSON via `DisplayJson` trait
- Low-level JSON access without full type conversion
- Custom validation with rich error context
- Pretty-printing with configurable indentation
- JSONC support (comments + trailing commas)

## Quick Start Patterns

### Parse JSON to Rust types
```rust
use nojson::Json;

// Direct parsing
let value: Json<Vec<i32>> = "[1, 2, 3]".parse()?;

// With nulls
let value: Json<Vec<Option<i32>>> = "[1, null, 3]".parse()?;
```

### Generate JSON from Rust types
```rust
use nojson::Json;

let json = Json(vec![1, 2, 3]).to_string();
// Output: [1,2,3]
```

### Low-level JSON access
```rust
let json = nojson::RawJson::parse(r#"{"name": "Alice", "age": 30}"#)?;
let name: String = json.value().to_member("name")?.required()?.try_into()?;
```

### Pretty printing
```rust
let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.value([1, 2, 3])
});
```

## Custom Types

### Implement parsing (TryFrom)
```rust
use nojson::{RawJsonValue, JsonParseError};

struct Person { name: String, age: u32 }

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for Person {
    type Error = JsonParseError;
    
    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Person {
            name: value.to_member("name")?.required()?.try_into()?,
            age: value.to_member("age")?.required()?.try_into()?,
        })
    }
}
```

### Implement generation (DisplayJson)
```rust
use nojson::{DisplayJson, JsonFormatter};

impl DisplayJson for Person {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("name", &self.name)?;
            f.member("age", self.age)
        })
    }
}
```

## Custom Validation

Add application-specific validation during parsing:

```rust
fn parse_positive_number(text: &str) -> Result<u32, nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let value = json.value();
    
    let num: u32 = value.as_number_str()?.parse()
        .map_err(|e| value.invalid(e))?;
    
    if num == 0 {
        return Err(value.invalid("Expected positive number, got 0"));
    }
    
    Ok(num)
}
```

## Key Type Usage

### nojson::RawJsonValue - Core parsing type
```rust
// Type checking
value.kind() // Get JsonValueKind
value.as_integer_str()? // Verify integer, return text
value.as_number_str()? // Verify number (int or float)
value.to_unquoted_string_str()? // Verify string, return unquoted

// Arrays
for element in value.to_array()? {
    let item: i32 = element.try_into()?;
}

// Objects
let name: String = value.to_member("name")?.required()?.try_into()?;
let city: Option<String> = value.to_member("city")?.try_into()?;

// Navigation
value.parent() // Parent array/object
value.root() // Root value
value.extract() // Extract as separate RawJson
```

### nojson::JsonFormatter - Core generation type
```rust
nojson::json(|f| {
    f.set_indent_size(2); // Spaces per level
    f.set_spacing(true); // Space after :,{
    
    f.object(|f| {
        f.member("name", "Alice")?;
        f.member("age", 30)?;
        f.member("items", nojson::array(|f| {
            f.element(1)?;
            f.element(2)?;
            f.element(3)
        }))
    })
})
```

## Error Handling

```rust
match nojson::RawJson::parse(text) {
    Err(error) => {
        // Get position context
        if let Some((line, col)) = error.get_line_and_column_numbers(text) {
            eprintln!("Error at line {}, column {}", line, col);
        }
        
        // Get error line
        if let Some(line_text) = error.get_line(text) {
            eprintln!("Line: {}", line_text);
        }
        
        // Find value at error position
        if let Ok(json) = nojson::RawJson::parse(valid_prefix) {
            if let Some(value) = json.get_value_by_position(error.position()) {
                eprintln!("Problem near: {}", value.as_raw_str());
            }
        }
    }
    Ok(json) => { /* use json */ }
}
```

## JSONC Support

Parse JSON with comments and trailing commas:

```rust
let text = r#"{
    "name": "Alice", // User name
    "age": 30,       // Trailing comma allowed
}"#;

let (json, comment_ranges) = nojson::RawJson::parse_jsonc(text)?;
// comment_ranges: Vec<Range<usize>> of comment positions
```

## Reference Files

- **[api.md](references/api.md)** - Complete API reference for all types and methods
- **[patterns.md](references/patterns.md)** - Common usage patterns and examples

## Important Notes

### Always use fully qualified names
Do NOT use imports. Write `nojson::RawJsonValue` instead of importing and using `RawJsonValue`.

**Correct:**
```rust
use nojson::{RawJsonValue, JsonParseError};

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for MyType {
    type Error = JsonParseError;
    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        // implementation
    }
}
```

**Incorrect:**
```rust
use nojson::*;
// Don't do this
```

### Lifetime parameters for TryFrom
When implementing `TryFrom<RawJsonValue>`, use `<'text, 'raw>` lifetime parameters:

```rust
impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for MyType {
    type Error = nojson::JsonParseError;
    // ...
}
```

### Custom validation pattern
Always use `value.invalid(error)` to create validation errors:

```rust
let num: u32 = value.as_number_str()?.parse()
    .map_err(|e| value.invalid(e))?;

if num == 0 {
    return Err(value.invalid("Must be positive"));
}
```

### Member access performance
`to_member()` is O(n). For multiple members, iterate once:

```rust
// Efficient for many members
let mut name = None;
let mut age = None;
for (key, value) in obj.to_object()? {
    match key.to_unquoted_string_str()?.as_ref() {
        "name" => name = Some(value),
        "age" => age = Some(value),
        _ => {}
    }
}
```

### Built-in type conversions
The crate provides `TryFrom<RawJsonValue>` for: primitives, NonZero types, String, Cow<str>, Vec, arrays, HashMap, BTreeMap, HashSet, BTreeSet, Option, Box, Rc, Arc, network types (IpAddr, SocketAddr), PathBuf, and () for null.

It provides `DisplayJson` for all the same types plus references (&T, &mut T).
