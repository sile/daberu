# nojson Usage Examples

## Parsing with Type Conversion

### Simple Array
```rust
use nojson::Json;

let text = "[1, null, 2]";
let value: Json<[Option<u32>; 3]> = text.parse()?;
assert_eq!(value.0, [Some(1), None, Some(2)]);
```

### Object to Struct
```rust
use nojson::{Json, JsonParseError, RawJsonValue};
use std::convert::TryFrom;

struct Person {
    name: String,
    age: u32,
}

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for Person {
    type Error = JsonParseError;

    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let name = value.to_member("name")?.required()?;
        let age = value.to_member("age")?.required()?;
        Ok(Person {
            name: name.try_into()?,
            age: age.try_into()?,
        })
    }
}

let json_text = r#"{"name":"Alice","age":30}"#;
let person: Json<Person> = json_text.parse()?;
```

### Nested Objects
```rust
let json = nojson::RawJson::parse(r#"
{
  "user": {
    "name": "John",
    "age": 30,
    "address": {
      "city": "New York"
    }
  }
}
"#)?;

let city: String = json.value()
    .to_member("user")?.required()?
    .to_member("address")?.required()?
    .to_member("city")?.required()?
    .try_into()?;

assert_eq!(city, "New York");
```

### Arrays of Objects
```rust
let json = nojson::RawJson::parse(r#"
[
  {"id": 1, "name": "Alice"},
  {"id": 2, "name": "Bob"},
  {"id": 3, "name": "Carol"}
]
"#)?;

for item in json.value().to_array()? {
    let id: u32 = item.to_member("id")?.required()?.try_into()?;
    let name: String = item.to_member("name")?.required()?.try_into()?;
    println!("User {}: {}", id, name);
}
```

### Optional Fields
```rust
let json = nojson::RawJson::parse(r#"
{
  "name": "Alice",
  "email": "alice@example.com"
}
"#)?;

let obj = json.value();
let name: String = obj.to_member("name")?.required()?.try_into()?;
let phone: Option<String> = obj.to_member("phone")?.try_into()?;

assert_eq!(name, "Alice");
assert_eq!(phone, None);
```

### With Maps
```rust
use std::collections::BTreeMap;

let json = nojson::RawJson::parse(r#"
{
  "alice": 85,
  "bob": 92,
  "carol": 78
}
"#)?;

let scores: BTreeMap<String, i32> = json.value().try_into()?;
assert_eq!(scores.get("alice"), Some(&85));
```

## Generating JSON

### From Arrays and Tuples
```rust
use nojson::Json;

let arr = [1, 2, 3];
assert_eq!(Json(&arr).to_string(), "[1,2,3]");

let tuple = (42, "hello", true);
assert_eq!(Json(&tuple).to_string(), r#"[42,"hello",true]"#);
```

### From Objects
```rust
use std::collections::BTreeMap;
use nojson::Json;

let mut obj = BTreeMap::new();
obj.insert("name", "Alice");
obj.insert("status", "active");

assert_eq!(
    Json(&obj).to_string(),
    r#"{"name":"Alice","status":"active"}"#
);
```

### With Custom Formatting
```rust
use nojson::json;

// Compact
let compact = json(|f| f.value([1, 2, 3]));
assert_eq!(compact.to_string(), "[1,2,3]");

// Pretty-printed with 2-space indentation
let pretty = json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.value([1, 2, 3])
});
```

### Building Objects Incrementally
```rust
use nojson::json;

let user = json(|f| {
    f.object(|f| {
        f.member("id", 123)?;
        f.member("name", "Alice")?;
        f.member("email", "alice@example.com")?;
        f.member("verified", true)
    })
});

println!("{}", user);
// {"id":123,"name":"Alice","email":"alice@example.com","verified":true}
```

### Building Nested Structures
```rust
use nojson::json;

let config = json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.object(|f| {
        f.member("server", json(|inner| {
            inner.object(|inner| {
                inner.member("host", "0.0.0.0")?;
                inner.member("port", 8080)
            })
        }))?;
        f.member("features", &["auth", "logging", "caching"])
    })
});
```

### Custom Type Implementation
```rust
use nojson::{DisplayJson, JsonFormatter, Json};

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl DisplayJson for Point {
    fn fmt(&self, f: &mut JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("x", self.x)?;
            f.member("y", self.y)
        })
    }
}

let point = Point { x: 1.5, y: 2.5 };
assert_eq!(Json(&point).to_string(), r#"{"x":1.5,"y":2.5}"#);
```

## Working with Raw JSON

### Preserving Original Text
```rust
use nojson::RawJson;

let text = r#"{"value": 12.5}"#;
let json = RawJson::parse(text)?;
let num_str = json.value().to_member("value")?.required()?.as_raw_str();
assert_eq!(num_str, "12.5");
```

### JSONC (JSON with Comments)
```rust
let jsonc_text = r#"{
  "name": "Config", // Application config
  "debug": true,    // Enable debug mode
  "port": 8080,     // Server port
  /* End config */
}"#;

let (json, comment_ranges) = nojson::RawJson::parse_jsonc(jsonc_text)?;
let name: String = json.value().to_member("name")?.required()?.try_into()?;

println!("Comments found at {} locations", comment_ranges.len());
for range in comment_ranges {
    println!("Comment: {}", &jsonc_text[range]);
}
```

### Custom Validation
```rust
use nojson::RawJson;

fn parse_positive_number(text: &str) -> Result<u32, nojson::JsonParseError> {
    let json = RawJson::parse(text)?;
    let raw_value = json.value();
    
    let num: u32 = raw_value.as_number_str()?
        .parse()
        .map_err(|e| raw_value.invalid(e))?;
    
    if num == 0 {
        return Err(raw_value.invalid("Expected a positive number, got 0"));
    }
    
    Ok(num)
}

assert_eq!(parse_positive_number("42")?, 42);
assert!(parse_positive_number("0").is_err());
assert!(parse_positive_number("-5").is_err());
```

### Error Context
```rust
use nojson::RawJson;

let text = r#"{"invalid": 123e++}"#;
match RawJson::parse(text) {
    Err(error) => {
        println!("Error: {}", error);
        
        if let Some((line, col)) = error.get_line_and_column_numbers(text) {
            println!("At line {}, column {}", line.get(), col.get());
        }
        
        if let Some(line) = error.get_line(text) {
            println!("Line content: {}", line);
        }
        
        if let Some(value) = RawJson::parse(text)
            .ok()
            .and_then(|j| j.get_value_by_position(error.position())) {
            println!("Value context: {}", value.as_raw_str());
        }
    }
    Ok(_) => unreachable!(),
}
```

### Value Navigation
```rust
let json = nojson::RawJson::parse(r#"
{
  "users": [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
  ]
}
"#)?;

// Navigate to nested value
let age_value = json.value()
    .to_member("users")?.required()?
    .to_array()?.next().unwrap()
    .to_member("age")?.required()?;

// Go back to root from nested value
let root = age_value.root();
let users_count = root
    .to_member("users")?.required()?
    .to_array()?.count();

assert_eq!(users_count, 2);
```

### Extracting Substructures
```rust
let json = nojson::RawJson::parse(r#"
{
  "metadata": {
    "created": "2024-01-01",
    "version": "1.0"
  },
  "data": [1, 2, 3]
}
"#)?;

// Extract just the metadata object
let metadata = json.value()
    .to_member("metadata")?.required()?
    .extract();

// Now metadata is a RawJson with just that subtree
let version: String = metadata.value()
    .to_member("version")?.required()?
    .try_into()?;

assert_eq!(version, "1.0");
```

## Advanced Patterns

### Stream Processing
```rust
use nojson::RawJson;

let json_array = nojson::RawJson::parse(r#"
[
  {"id": 1, "score": 85},
  {"id": 2, "score": 92},
  {"id": 3, "score": 78}
]
"#)?;

let total: u32 = json_array.value()
    .to_array()?
    .map(|item| item.to_member("score").and_then(|m| m.required()).and_then(|v| v.try_into()))
    .collect::<Result<Vec<u32>, _>>()?
    .iter()
    .sum();

assert_eq!(total, 255);
```

### Polymorphic JSON
```rust
use nojson::RawJsonValue;

fn process_value(value: RawJsonValue) -> Result<String, nojson::JsonParseError> {
    match value.kind() {
        nojson::JsonValueKind::Null => Ok("null".to_string()),
        nojson::JsonValueKind::Boolean => Ok(value.as_boolean_str()?.to_string()),
        nojson::JsonValueKind::Integer => Ok(value.as_integer_str()?.to_string()),
        nojson::JsonValueKind::Float => Ok(value.as_float_str()?.to_string()),
        nojson::JsonValueKind::String => {
            let s = value.to_unquoted_string_str()?;
            Ok(format!("String: {}", s))
        }
        nojson::JsonValueKind::Array => {
            Ok(format!("Array with {} elements", value.to_array()?.count()))
        }
        nojson::JsonValueKind::Object => {
            Ok(format!("Object with {} members", value.to_object()?.count()))
        }
    }
}
```

### Conditional Parsing
```rust
use nojson::{RawJson, JsonValueKind};

let json = RawJson::parse(r#"
{
  "type": "user",
  "data": {"name": "Alice", "age": 30}
}
"#)?;

let type_str: String = json.value()
    .to_member("type")?.required()?.try_into()?;

let data = json.value().to_member("data")?.required()?;

match type_str.as_str() {
    "user" => {
        let name = data.to_member("name")?.required()?;
        println!("User: {}", name.to_unquoted_string_str()?);
    }
    _ => println!("Unknown type"),
}
```
