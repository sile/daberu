#! /bin/sh

set -eux

cd ~/rust/mp4-rust/

echo 'Create a comprehensive markdown documentation file for this Rust crate, optimized for LLM consumption.

## Structure Requirements

Include the following sections in this order:

### 1. Crate Summary
- Brief overview of what the crate does
- Key use cases and benefits
- Design philosophy

### 2. API Reference
- Complete and organized documentation of all public types, traits, and functions
- Document parameters, return types, and error conditions
- Group related items logically

### 3. Examples and Common Patterns
- Practical usage examples demonstrating core functionality
- Common workflows and patterns
- Edge cases and error handling

## Code Examples Guidelines

- Keep examples concise and focused
- Include both basic and advanced patterns
- Assume that users can use `std` features (i.e., use std types and macros instead of `core::*` and `alloc::*` in examples)

## Additional Notes

- Assume readers are language models with broad programming knowledge
- Prioritize clarity and completeness over brevity
- Use clear section headers and consistent formatting
- Include code blocks with Rust syntax highlighting' | \
  daberu \
    -m claude-sonnet-4-5 \
    -r README.md \
    -r Cargo.toml \
    -g 'src/*.rs' \
    -g 'tests/*.rs' \
    -g 'examples/*.rs' 
