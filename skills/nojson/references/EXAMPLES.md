# nojson Examples

Comprehensive examples demonstrating common patterns and use cases.

## Table of Contents

- [Basic Parsing](#basic-parsing)
- [Basic Generation](#basic-generation)
- [Custom Types](#custom-types)
- [Complex Structures](#complex-structures)
- [Error Handling](#error-handling)
- [Validation](#validation)
- [JSONC Support](#jsonc-support)
- [Advanced Patterns](#advanced-patterns)

## Basic Parsing

### Parse Simple Values

```rust
fn parse_primitives() -> Result<(), nojson::JsonParseError> {
    // Boolean
    let json = nojson::RawJson::parse("true")?;
    let value: bool = json.value().try_into()?;
    assert_eq!(value, true);
    
    // Integer
    let json = nojson::RawJson::parse("42")?;
    let value: i32 = json.value().try_into()?;
    assert_eq!(value, 42);
    
    // Float
    let json = nojson::RawJson::parse("3.14")?;
    let value: f64 = json.value().try_into()?;
    assert_eq!(value, 3.14);
    
    // String
    let json = nojson::RawJson::parse(r#""hello""#)?;
    let value: String = json.value().try_into()?;
    assert_eq!(value, "hello");
    
    // Null
    let json = nojson::RawJson::parse("null")?;
    let value: () = json.value().try_into()?;
    
    // Option
    let json = nojson::RawJson::parse("null")?;
    let value: Option<i32> = json.value().try_into()?;
    assert_eq!(value, None);
    
    Ok(())
}
```

### Parse Arrays

```rust
fn parse_arrays() -> Result<(), nojson::JsonParseError> {
    // Fixed-size array
    let json = nojson::RawJson::parse("[1, 2, 3]")?;
    let arr: [i32; 3] = json.value().try_into()?;
    assert_eq!(arr, [1, 2, 3]);
    
    // Dynamic Vec
    let json = nojson::RawJson::parse("[1, 2, 3, 4, 5]")?;
    let vec: Vec<i32> = json.value().try_into()?;
    assert_eq!(vec.len(), 5);
    
    // Array with Options
    let json = nojson::RawJson::parse("[1, null, 3]")?;
    let arr: [Option<i32>; 3] = json.value().try_into()?;
    assert_eq!(arr, [Some(1), None, Some(3)]);
    
    // Nested arrays
    let json = nojson::RawJson::parse("[[1, 2], [3, 4]]")?;
    let nested: Vec<Vec<i32>> = json.value().try_into()?;
    assert_eq!(nested, vec![vec![1, 2], vec![3, 4]]);
    
    Ok(())
}
```

### Parse Objects

```rust
fn parse_objects() -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(r#"{"name": "Alice", "age": 30, "active": true}"#)?;
    
    // Access individual members
    let name: String = json.value().to_member("name")?.required()?.try_into()?;
    let age: u32 = json.value().to_member("age")?.required()?.try_into()?;
    let active: bool = json.value().to_member("active")?.required()?.try_into()?;
    
    assert_eq!(name, "Alice");
    assert_eq!(age, 30);
    assert_eq!(active, true);
    
    // Optional member
    let email: Option<String> = json.value().to_member("email")?.try_into()?;
    assert_eq!(email, None);
    
    // Parse to HashMap
    use std::collections::HashMap;
    let json = nojson::RawJson::parse(r#"{"a": 1, "b": 2, "c": 3}"#)?;
    let map: HashMap<String, i32> = json.value().try_into()?;
    assert_eq!(map.get("a"), Some(&1));
    
    Ok(())
}
```

### Iterate Collections

```rust
fn iterate_collections() -> Result<(), nojson::JsonParseError> {
    // Iterate array
    let json = nojson::RawJson::parse("[1, 2, 3, 4, 5]")?;
    let mut sum = 0;
    for element in json.value().to_array()? {
        let num: i32 = element.try_into()?;
        sum += num;
    }
    assert_eq!(sum, 15);
    
    // Iterate object
    let json = nojson::RawJson::parse(r#"{"a": 1, "b": 2, "c": 3}"#)?;
    for (key, value) in json.value().to_object()? {
        let k = key.to_unquoted_string_str()?;
        let v: i32 = value.try_into()?;
        println!("{} = {}", k, v);
    }
    
    Ok(())
}
```

## Basic Generation

### Generate Simple Values

```rust
fn generate_primitives() {
    // Using Json wrapper
    assert_eq!(nojson::Json(true).to_string(), "true");
    assert_eq!(nojson::Json(42).to_string(), "42");
    assert_eq!(nojson::Json(3.14).to_string(), "3.14");
    assert_eq!(nojson::Json("hello").to_string(), r#""hello""#);
    assert_eq!(nojson::Json(()).to_string(), "null");
    assert_eq!(nojson::Json(None::<i32>).to_string(), "null");
    assert_eq!(nojson::Json(Some(42)).to_string(), "42");
}
```

### Generate Arrays

```rust
fn generate_arrays() {
    // Fixed-size array
    let arr = [1, 2, 3];
    assert_eq!(nojson::Json(arr).to_string(), "[1,2,3]");
    
    // Vec
    let vec = vec![1, 2, 3, 4, 5];
    assert_eq!(nojson::Json(&vec).to_string(), "[1,2,3,4,5]");
    
    // With Options
    let arr = [Some(1), None, Some(3)];
    assert_eq!(nojson::Json(arr).to_string(), "[1,null,3]");
    
    // Using array builder
    let arr = nojson::array(|f| {
        f.element(1)?;
        f.element(2)?;
        f.element(3)
    });
    assert_eq!(arr.to_string(), "[1,2,3]");
    
    // Pretty-printed
    let pretty = nojson::json(|f| {
        f.set_indent_size(2);
        f.set_spacing(true);
        f.array(|f| f.elements([1, 2, 3]))
    });
    println!("{}", pretty);
    // Output:
    // [
    //   1,
    //   2,
    //   3
    // ]
}
```

### Generate Objects

```rust
fn generate_objects() {
    use std::collections::BTreeMap;
    
    // Using BTreeMap
    let mut map = BTreeMap::new();
    map.insert("name", "Alice");
    map.insert("city", "NYC");
    assert_eq!(nojson::Json(&map).to_string(), r#"{"city":"NYC","name":"Alice"}"#);
    
    // Using object builder
    let obj = nojson::object(|f| {
        f.member("name", "Alice")?;
        f.member("age", 30)?;
        f.member("active", true)
    });
    assert_eq!(obj.to_string(), r#"{"name":"Alice","age":30,"active":true}"#);
    
    // Pretty-printed
    let pretty = nojson::json(|f| {
        f.set_indent_size(2);
        f.set_spacing(true);
        f.object(|f| {
            f.member("name", "Alice")?;
            f.member("age", 30)
        })
    });
    println!("{}", pretty);
    // Output:
    // {
    //   "name": "Alice",
    //   "age": 30
    // }
}
```

### Formatting Options

```rust
fn formatting_examples() {
    let data = vec![1, 2, 3];
    
    // Compact (default)
    let compact = nojson::json(|f| f.value(&data));
    assert_eq!(compact.to_string(), "[1,2,3]");
    
    // With spacing only
    let spaced = nojson::json(|f| {
        f.set_spacing(true);
        f.value(&data)
    });
    assert_eq!(spaced.to_string(), "[1, 2, 3]");
    
    // Pretty-printed
    let pretty = nojson::json(|f| {
        f.set_indent_size(2);
        f.set_spacing(true);
        f.value(&data)
    });
    // Output:
    // [
    //   1,
    //   2,
    //   3
    // ]
    
    // Mixed formatting (compact nested in pretty)
    let mixed = nojson::json(|f| {
        f.set_indent_size(2);
        f.set_spacing(true);
        f.array(|f| {
            f.element(&vec![1])?;
            f.element(nojson::json(|f| {
                f.set_indent_size(0);
                f.value(vec![2, 3])
            }))
        })
    });
}
```

## Custom Types

### Simple Struct

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

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Person {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let name = value.to_member("name")?.required()?;
        let age = value.to_member("age")?.required()?;
        Ok(Person {
            name: name.try_into()?,
            age: age.try_into()?,
        })
    }
}

fn use_person() -> Result<(), nojson::JsonParseError> {
    // Parse
    let json_text = r#"{"name":"Alice","age":30}"#;
    let person: nojson::Json<Person> = json_text.parse()?;
    
    // Generate
    let generated = nojson::Json(&person.0).to_string();
    assert_eq!(generated, json_text);
    
    Ok(())
}
```

### Struct with Optional Fields

```rust
struct User {
    id: u64,
    name: String,
    email: Option<String>,
    verified: bool,
}

impl nojson::DisplayJson for User {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("id", self.id)?;
            f.member("name", &self.name)?;
            f.member("email", &self.email)?;
            f.member("verified", self.verified)
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for User {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(User {
            id: value.to_member("id")?.required()?.try_into()?,
            name: value.to_member("name")?.required()?.try_into()?,
            email: value.to_member("email")?.try_into()?,
            verified: value.to_member("verified")?.required()?.try_into()?,
        })
    }
}
```

### Enum Representation

```rust
enum Status {
    Active,
    Inactive,
    Pending,
}

impl nojson::DisplayJson for Status {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        let s = match self {
            Status::Active => "active",
            Status::Inactive => "inactive",
            Status::Pending => "pending",
        };
        f.string(s)
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Status {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let s = value.to_unquoted_string_str()?;
        match s.as_ref() {
            "active" => Ok(Status::Active),
            "inactive" => Ok(Status::Inactive),
            "pending" => Ok(Status::Pending),
            _ => Err(value.invalid(format!("unknown status: {}", s))),
        }
    }
}
```

### Tagged Enum (Externally Tagged)

```rust
enum Event {
    Click { x: i32, y: i32 },
    Keypress { key: String },
    Scroll { delta: i32 },
}

impl nojson::DisplayJson for Event {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            match self {
                Event::Click { x, y } => {
                    f.member("type", "click")?;
                    f.member("x", *x)?;
                    f.member("y", *y)
                }
                Event::Keypress { key } => {
                    f.member("type", "keypress")?;
                    f.member("key", key)
                }
                Event::Scroll { delta } => {
                    f.member("type", "scroll")?;
                    f.member("delta", *delta)
                }
            }
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Event {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let event_type: String = value.to_member("type")?.required()?.try_into()?;
        
        match event_type.as_str() {
            "click" => {
                let x = value.to_member("x")?.required()?.try_into()?;
                let y = value.to_member("y")?.required()?.try_into()?;
                Ok(Event::Click { x, y })
            }
            "keypress" => {
                let key = value.to_member("key")?.required()?.try_into()?;
                Ok(Event::Keypress { key })
            }
            "scroll" => {
                let delta = value.to_member("delta")?.required()?.try_into()?;
                Ok(Event::Scroll { delta })
            }
            _ => Err(value.invalid(format!("unknown event type: {}", event_type))),
        }
    }
}
```

## Complex Structures

### Nested Structures

```rust
struct Address {
    street: String,
    city: String,
    country: String,
}

struct Company {
    name: String,
    address: Address,
}

impl nojson::DisplayJson for Address {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("street", &self.street)?;
            f.member("city", &self.city)?;
            f.member("country", &self.country)
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Address {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Address {
            street: value.to_member("street")?.required()?.try_into()?,
            city: value.to_member("city")?.required()?.try_into()?,
            country: value.to_member("country")?.required()?.try_into()?,
        })
    }
}

impl nojson::DisplayJson for Company {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("name", &self.name)?;
            f.member("address", &self.address)
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Company {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Company {
            name: value.to_member("name")?.required()?.try_into()?,
            address: value.to_member("address")?.required()?.try_into()?,
        })
    }
}
```

### Collections of Custom Types

```rust
struct Task {
    id: u32,
    title: String,
    completed: bool,
}

struct TodoList {
    name: String,
    tasks: Vec<Task>,
}

impl nojson::DisplayJson for Task {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("id", self.id)?;
            f.member("title", &self.title)?;
            f.member("completed", self.completed)
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Task {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Task {
            id: value.to_member("id")?.required()?.try_into()?,
            title: value.to_member("title")?.required()?.try_into()?,
            completed: value.to_member("completed")?.required()?.try_into()?,
        })
    }
}

impl nojson::DisplayJson for TodoList {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("name", &self.name)?;
            f.member("tasks", &self.tasks)
        })
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for TodoList {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(TodoList {
            name: value.to_member("name")?.required()?.try_into()?,
            tasks: value.to_member("tasks")?.required()?.try_into()?,
        })
    }
}
```

## Error Handling

### Basic Error Handling

```rust
fn handle_parse_errors(text: &str) {
    match nojson::RawJson::parse(text) {
        Ok(json) => {
            println!("Parsed successfully");
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            eprintln!("Error position: {}", e.position());
            
            if let Some((line, col)) = e.get_line_and_column_numbers(text) {
                eprintln!("At line {}, column {}", line, col);
            }
            
            if let Some(line_text) = e.get_line(text) {
                eprintln!("Line: {}", line_text);
            }
        }
    }
}
```

### Detailed Error Messages

```rust
fn detailed_error_report(text: &str) -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text).map_err(|e| {
        let mut msg = format!("JSON parse error: {}", e);
        
        if let Some((line, col)) = e.get_line_and_column_numbers(text) {
            msg.push_str(&format!("\n  at line {}, column {}", line, col));
        }
        
        if let Some(line_text) = e.get_line(text) {
            msg.push_str(&format!("\n  line: {}", line_text));
            
            if let Some((_, col)) = e.get_line_and_column_numbers(text) {
                let pointer = " ".repeat(col.get() + 7) + "^";
                msg.push_str(&format!("\n  {}", pointer));
            }
        }
        
        eprintln!("{}", msg);
        e
    })?;
    
    Ok(())
}
```

### Recovery Strategies

```rust
fn parse_with_fallback(text: &str) -> Result<i32, nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let value = json.value();
    
    // Try parsing as integer
    if let Ok(n) = value.try_into::<i32>() {
        return Ok(n);
    }
    
    // Fallback: try parsing string as integer
    if let Ok(s) = value.try_into::<String>() {
        if let Ok(n) = s.parse::<i32>() {
            return Ok(n);
        }
    }
    
    Err(value.invalid("expected integer or string containing integer"))
}
```

## Validation

### Range Validation

```rust
fn parse_percentage(text: &str) -> Result<f64, nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let value = json.value();
    
    let num: f64 = value.as_number_str()?
        .parse()
        .map_err(|e| value.invalid(e))?;
    
    if num < 0.0 || num > 100.0 {
        return Err(value.invalid(format!("percentage must be 0-100, got {}", num)));
    }
    
    Ok(num)
}
```

### Pattern Validation

```rust
fn parse_email(text: &str) -> Result<String, nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let value = json.value();
    
    let email: String = value.try_into()?;
    
    if !email.contains('@') {
        return Err(value.invalid("email must contain @"));
    }
    
    Ok(email)
}
```

### Custom Validation Logic

```rust
struct ValidatedUser {
    username: String,
    age: u32,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ValidatedUser {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let username_val = value.to_member("username")?.required()?;
        let username: String = username_val.try_into()?;
        
        // Validate username
        if username.len() < 3 {
            return Err(username_val.invalid("username must be at least 3 characters"));
        }
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(username_val.invalid("username must be alphanumeric"));
        }
        
        let age_val = value.to_member("age")?.required()?;
        let age: u32 = age_val.try_into()?;
        
        // Validate age
        if age < 13 {
            return Err(age_val.invalid("user must be at least 13 years old"));
        }
        if age > 120 {
            return Err(age_val.invalid("age seems unrealistic"));
        }
        
        Ok(ValidatedUser { username, age })
    }
}
```

### Cross-Field Validation

```rust
struct DateRange {
    start: i64,
    end: i64,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for DateRange {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let start: i64 = value.to_member("start")?.required()?.try_into()?;
        let end_val = value.to_member("end")?.required()?;
        let end: i64 = end_val.try_into()?;
        
        if end < start {
            return Err(end_val.invalid("end date must be after start date"));
        }
        
        Ok(DateRange { start, end })
    }
}
```

## JSONC Support

### Parse JSONC with Comments

```rust
fn parse_config_with_comments() -> Result<(), nojson::JsonParseError> {
    let config_text = r#"{
        // Server configuration
        "host": "localhost",
        "port": 8080,
        
        /* Database settings
           Multiple lines supported */
        "database": {
            "url": "postgres://localhost/mydb",
            "pool_size": 10, // Max connections
        },
        
        // Feature flags
        "features": [
            "auth",      // Authentication
            "logging",   // Request logging
            "metrics",   // Performance metrics
        ]
    }"#;
    
    let (json, comment_ranges) = nojson::RawJson::parse_jsonc(config_text)?;
    
    // Parse configuration
    let host: String = json.value().to_member("host")?.required()?.try_into()?;
    let port: u16 = json.value().to_member("port")?.required()?.try_into()?;
    
    println!("Server: {}:{}", host, port);
    
    // Process comments if needed
    for range in comment_ranges {
        let comment = &config_text[range];
        if comment.contains("TODO") || comment.contains("FIXME") {
            println!("Found annotation: {}", comment);
        }
    }
    
    Ok(())
}
```

### Handle Trailing Commas

```rust
fn parse_with_trailing_commas() -> Result<(), nojson::JsonParseError> {
    // Trailing commas are allowed in JSONC
    let text = r#"{
        "items": [
            "first",
            "second",
            "third",
        ],
        "count": 3,
    }"#;
    
    let (json, _) = nojson::RawJson::parse_jsonc(text)?;
    let items: Vec<String> = json.value().to_member("items")?.required()?.try_into()?;
    
    assert_eq!(items.len(), 3);
    
    Ok(())
}
```

## Advanced Patterns

### Dynamic Field Access

```rust
fn get_nested_field(
    json: &nojson::RawJson,
    path: &[&str],
) -> Result<nojson::RawJsonValue, nojson::JsonParseError> {
    let mut current = json.value();
    
    for field in path {
        current = current.to_member(field)?.required()?;
    }
    
    Ok(current)
}

fn use_dynamic_access() -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(r#"{
        "user": {
            "profile": {
                "name": "Alice"
            }
        }
    }"#)?;
    
    let name_value = get_nested_field(&json, &["user", "profile", "name"])?;
    let name: String = name_value.try_into()?;
    
    assert_eq!(name, "Alice");
    
    Ok(())
}
```

### Partial Parsing

```rust
fn parse_large_json_partially(text: &str) -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    
    // Only parse what we need
    let id: u64 = json.value().to_member("id")?.required()?.try_into()?;
    
    // Skip processing other fields
    println!("ID: {}", id);
    
    Ok(())
}
```

### Streaming Array Processing

```rust
fn process_large_array(text: &str) -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    
    // Process array elements one at a time
    for (index, element) in json.value().to_array()?.enumerate() {
        let item: i32 = element.try_into()?;
        
        // Process item without loading entire array
        println!("Item {}: {}", index, item);
        
        // Can break early if needed
        if index >= 100 {
            break;
        }
    }
    
    Ok(())
}
```

### Conditional Parsing

```rust
fn parse_polymorphic_data(text: &str) -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    let value = json.value();
    
    // Check type discriminator
    let type_field: String = value.to_member("type")?.required()?.try_into()?;
    
    match type_field.as_str() {
        "user" => {
            let username: String = value.to_member("username")?.required()?.try_into()?;
            println!("User: {}", username);
        }
        "group" => {
            let group_name: String = value.to_member("name")?.required()?.try_into()?;
            let member_count: usize = value.to_member("members")?
                .required()?
                .to_array()?
                .count();
            println!("Group: {} ({} members)", group_name, member_count);
        }
        _ => return Err(value.invalid(format!("unknown type: {}", type_field))),
    }
    
    Ok(())
}
```

### Value Extraction and Manipulation

```rust
fn extract_and_modify() -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(r#"{
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]
    }"#)?;
    
    // Extract users array
    let users_value = json.value().to_member("users")?.required()?;
    let users_json = users_value.extract();
    
    // Now users_json is a standalone RawJson containing just the array
    println!("Users JSON: {}", users_json.text());
    
    // Can convert to owned if needed
    let owned = users_json.into_owned();
    
    Ok(())
}
```

### Building JSON from Mixed Sources

```rust
fn build_complex_json() -> String {
    // Mix different sources
    let header = vec![("version", "1.0"), ("timestamp", "2024-01-01")];
    let data = vec![1, 2, 3, 4, 5];
    
    nojson::json(|f| {
        f.set_indent_size(2);
        f.set_spacing(true);
        f.object(|f| {
            // Add header fields
            f.member("header", nojson::object(|f| {
                f.members(header)
            }))?;
            
            // Add data array
            f.member("data", &data)?;
            
            // Add computed field
            f.member("count", data.len())
        })
    }).to_string()
}
```

### Error Context Preservation

```rust
fn parse_with_context(text: &str) -> Result<Vec<String>, nojson::JsonParseError> {
    let json = nojson::RawJson::parse(text)?;
    
    json.value()
        .to_array()?
        .enumerate()
        .map(|(i, element)| {
            element.try_into::<String>().map_err(|e| {
                // Add context to error
                eprintln!("Error parsing array element {}: {}", i, e);
                e
            })
        })
        .collect()
}
```

### Using map for Transformations

```rust
fn transform_values() -> Result<(), nojson::JsonParseError> {
    let json = nojson::RawJson::parse(r#"{"name": "alice"}"#)?;
    
    // Transform with map
    let uppercase_name: String = json.value()
        .to_member("name")?
        .required()?
        .map(|v| {
            let s: String = v.try_into()?;
            Ok(s.to_uppercase())
        })?;
    
    assert_eq!(uppercase_name, "ALICE");
    
    // Optional transformation
    let opt_name: Option<String> = json.value()
        .to_member("nickname")?
        .map(|v| {
            let s: String = v.try_into()?;
            Ok(s.to_uppercase())
        })?;
    
    assert_eq!(opt_name, None);
    
    Ok(())
}
```
