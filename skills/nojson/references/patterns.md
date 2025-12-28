# Common Patterns

## Parsing JSON

### Basic parsing with Json wrapper
```rust
use nojson::Json;

// Parse directly to Rust types
let value: Json<Vec<i32>> = "[1, 2, 3]".parse()?;
assert_eq!(value.0, vec![1, 2, 3]);

// With nulls
let value: Json<Vec<Option<i32>>> = "[1, null, 3]".parse()?;
assert_eq!(value.0, vec![Some(1), None, Some(3)]);
```

### Low-level parsing with RawJson
```rust
use nojson::RawJson;

let json = nojson::RawJson::parse(r#"{"name": "John", "age": 30}"#)?;
let root = json.value();

// Access object members
let name: String = root.to_member("name")?.required()?.try_into()?;
let age: i32 = root.to_member("age")?.required()?.try_into()?;
```

### Parsing JSONC (with comments)
```rust
let text = r#"{
    "name": "Alice", // User name
    "age": 30,       // Age in years
}"#;

let (json, comment_ranges) = nojson::RawJson::parse_jsonc(text)?;
// comment_ranges contains byte ranges of comments in original text
```

## Custom Type Conversion

### Implementing TryFrom for parsing
```rust
use nojson::{RawJsonValue, JsonParseError};

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

// Usage
let person: nojson::Json<Person> = r#"{"name":"Alice","age":30}"#.parse()?;
```

### Implementing DisplayJson for generation
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

// Usage
let json_text = nojson::Json(&person).to_string();
```

## Custom Validation

### Adding validation during parsing
```rust
use nojson::{RawJson, RawJsonValue, JsonParseError};

fn parse_positive_number(text: &str) -> Result<u32, JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let raw_value = json.value();
    
    let num: u32 = raw_value.as_number_str()?
        .parse()
        .map_err(|e| raw_value.invalid(e))?;
    
    if num == 0 {
        return Err(raw_value.invalid("Expected positive number, got 0"));
    }
    
    Ok(num)
}
```

### Validation in custom TryFrom
```rust
struct Email(String);

impl<'text, 'raw> TryFrom<RawJsonValue<'text, 'raw>> for Email {
    type Error = JsonParseError;
    
    fn try_from(value: RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let s: String = value.try_into()?;
        if !s.contains('@') {
            return Err(value.invalid("Invalid email format"));
        }
        Ok(Email(s))
    }
}
```

## Generating JSON

### Simple generation
```rust
use nojson::Json;

// From standard types
let json = nojson::Json(vec![1, 2, 3]).to_string();
assert_eq!(json, "[1,2,3]");

// With nulls
let json = nojson::Json(vec![Some(1), None, Some(3)]).to_string();
assert_eq!(json, "[1,null,3]");
```

### Pretty printing
```rust
use nojson::json;

let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.array(|f| {
        f.element(1)?;
        f.element(2)?;
        f.element(3)
    })
});

println!("{}", output);
// [
//   1,
//   2,
//   3
// ]
```

### In-place object construction
```rust
let output = nojson::object(|f| {
    f.member("name", "Alice")?;
    f.member("age", 30)?;
    f.member("active", true)
});

assert_eq!(output.to_string(), r#"{"name":"Alice","age":30,"active":true}"#);
```

### Nested structures
```rust
let output = nojson::json(|f| {
    f.set_indent_size(2);
    f.set_spacing(true);
    f.object(|f| {
        f.member("user", nojson::object(|f| {
            f.member("name", "Bob")?;
            f.member("age", 25)
        }))?;
        f.member("tags", &["rust", "json"])
    })
});
```

## Working with Arrays

### Iterating array elements
```rust
let json = nojson::RawJson::parse("[1, 2, 3, 4, 5]")?;
let array = json.value();

for element in array.to_array()? {
    let num: i32 = element.try_into()?;
    println!("{}", num);
}
```

### Converting to fixed-size array
```rust
let json = nojson::RawJson::parse("[1, 2, 3]")?;
let fixed: [i32; 3] = json.value().try_into()?;
assert_eq!(fixed, [1, 2, 3]);
```

### Converting to Vec
```rust
let json = nojson::RawJson::parse("[1, 2, 3, 4, 5]")?;
let vec: Vec<i32> = json.value().try_into()?;
```

## Working with Objects

### Iterating object members
```rust
let json = nojson::RawJson::parse(r#"{"a": 1, "b": 2, "c": 3}"#)?;
let object = json.value();

for (key, value) in object.to_object()? {
    let k = key.to_unquoted_string_str()?;
    let v: i32 = value.try_into()?;
    println!("{}: {}", k, v);
}
```

### Accessing specific members
```rust
let json = nojson::RawJson::parse(r#"{"name": "Alice", "age": 30}"#)?;
let obj = json.value();

// Required member
let name: String = obj.to_member("name")?.required()?.try_into()?;

// Optional member
let city: Option<String> = obj.to_member("city")?.try_into()?;
```

### Converting to HashMap
```rust
use std::collections::HashMap;

let json = nojson::RawJson::parse(r#"{"a": 1, "b": 2}"#)?;
let map: HashMap<String, i32> = json.value().try_into()?;
```

## Error Handling

### Getting error context
```rust
let text = r#"{"invalid": 123e++}"#;
match nojson::RawJson::parse(text) {
    Err(error) => {
        println!("Error: {}", error);
        
        // Get line and column
        if let Some((line, col)) = error.get_line_and_column_numbers(text) {
            println!("At line {}, column {}", line, col);
        }
        
        // Get error line text
        if let Some(line_text) = error.get_line(text) {
            println!("Line: {}", line_text);
        }
        
        // Get position
        println!("Position: {}", error.position());
    }
    Ok(_) => {}
}
```

### Finding value by position
```rust
let json = nojson::RawJson::parse(r#"{"name": "John", "age": 30}"#)?;

// Find value at byte position 2 (the "name" key)
if let Some(value) = json.get_value_by_position(2) {
    println!("Value at position 2: {}", value.as_raw_str());
}
```

## Navigation

### Traversing JSON structure
```rust
let json = nojson::RawJson::parse(r#"{"users": [{"name": "Alice"}]}"#)?;
let root = json.value();

// Navigate down
let users = root.to_member("users")?.required()?;
let first_user = users.to_array()?.next().unwrap();
let name = first_user.to_member("name")?.required()?;

// Navigate up
assert_eq!(name.parent().unwrap().position(), first_user.position());
assert_eq!(name.root().position(), root.position());
```

### Extracting substructures
```rust
let json = nojson::RawJson::parse(r#"{"user": {"name": "Bob", "age": 25}}"#)?;
let user_value = json.value().to_member("user")?.required()?;

// Extract user object as separate RawJson
let user_json = user_value.extract();
assert_eq!(user_json.text(), r#"{"name": "Bob", "age": 25}"#);

// Can be converted to owned
let owned = user_json.into_owned();
```

## Advanced Patterns

### Conditional parsing
```rust
use nojson::{RawJsonValue, JsonParseError, JsonValueKind};

fn parse_number_or_string(value: RawJsonValue) -> Result<f64, JsonParseError> {
    match value.kind() {
        JsonValueKind::Integer | JsonValueKind::Float => {
            value.as_number_str()?.parse().map_err(|e| value.invalid(e))
        }
        JsonValueKind::String => {
            value.to_unquoted_string_str()?.parse().map_err(|e| value.invalid(e))
        }
        _ => Err(value.invalid("Expected number or string"))
    }
}
```

### Using map for transformations
```rust
let json = nojson::RawJson::parse(r#""42""#)?;

let number: i32 = json.value().map(|v| {
    v.to_unquoted_string_str()?.parse().map_err(|e| v.invalid(e))
})?;

assert_eq!(number, 42);
```

### Optional member transformation
```rust
let json = nojson::RawJson::parse(r#"{"age": "30"}"#)?;
let obj = json.value();

// Parse optional string to optional number
let age: Option<i32> = obj.to_member("age")?.map(|v| {
    v.to_unquoted_string_str()?.parse().map_err(|e| v.invalid(e))
})?;

assert_eq!(age, Some(30));
```

### Efficient multi-member access
```rust
// When accessing many members, iterate once instead of multiple O(n) lookups
let json = nojson::RawJson::parse(r#"{"a": 1, "b": 2, "c": 3, "d": 4}"#)?;

let mut a = None;
let mut b = None;
let mut d = None;

for (key, value) in json.value().to_object()? {
    match key.to_unquoted_string_str()?.as_ref() {
        "a" => a = Some(value),
        "b" => b = Some(value),
        "d" => d = Some(value),
        _ => {}
    }
}
```
