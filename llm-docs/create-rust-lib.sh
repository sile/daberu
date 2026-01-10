#! /bin/sh

set -eux

cd $1

echo 'Create a comprehensive markdown documentation file for this Rust crate, optimized for LLM consumption.

## Structure Requirements

Include the following sections in this order:

### 1. Crate Summary
- Brief overview of what the crate does
- Key use cases and benefits
- Design philosophy

### 2. API Reference
- Complete and organized documentation of all public types, traits, and functions
- Include signatures with full qualified names (e.g., `CRATE_NAME::STRUCT_NAME`)
- Document parameters, return types, and error conditions
- Group related items logically

### 3. Examples and Common Patterns
- Practical usage examples demonstrating core functionality
- Common workflows and patterns
- Edge cases and error handling

## Code Examples Guidelines

- **DO NOT** use `use` statements or imports
- Always use fully qualified names (e.g., `CRATE_NAME::STRUCT_NAME`)
- Keep examples concise and focused
- Include both basic and advanced patterns

## Additional Notes

- Assume readers are language models with broad programming knowledge
- Prioritize clarity and completeness over brevity
- Use clear section headers and consistent formatting
- Include code blocks with Rust syntax highlighting' | \
  daberu \
    -m claude-sonnet-4-5 \
    -r README.md \
    -r Cargo.toml \
    -g '**.rs'

