# Vernacular Programming Language

Vernacular is a modern programming language designed to bridge the gap between natural language and code. It emphasizes readability and expressiveness while maintaining strong type safety.

## Design Philosophy

Vernacular is built on several core principles:

1. **Natural Readability**: Code should read like natural English while maintaining programming precision
2. **Strong Type Safety**: Static typing with clear type annotations using the `as` keyword
3. **Flexible Type System**: Support for both strict typing and dynamic `Any` types
4. **Clear Intent**: Keywords like `is`, `as`, `when`, and `requires` make code intent obvious

## Key Features

### Type System
```nair8
# Explicit type declarations
count as Whole        # Integer type
measure as Decimal    # Float type
message as Text       # String type
flag as Truth         # Boolean type
empty as Void         # Null type

# Dynamic typing
flexible is "Hello"   # Type inferred as Any
flexible is 42        # Valid - Any type can change
```

### Functions (Jobs)
```nair8
Job calculate requires x, y as Whole returning Whole:
    output x + y

# With multiple parameters
Job greet requires name as Text, age as Whole:
    show "Hello {name}, you are {age} years old"
```

### Control Flow
```nair8
when count > 10:
    show "Greater than 10"
or:
    show "Less than or equal to 10"

loop while condition:
    # Loop body
```

### Object-Oriented Programming
```nair8
Object Person inherits BaseEntity:
    build defaults name as Text, age as Whole:
        my name is name
        my age is age
    
    Job greet returns Text:
        output "Hello, I'm {my name}"
```

### Error Handling
```nair8
do:
    result is riskyOperation()
fail error as Error:
    show "Error: {error.message}"
always:
    cleanup()
```

## Running Vernacular

1. Install Rust (required to build Vernacular)
2. Clone the repository:
```bash
git clone https://github.com/yourusername/Vernacular.git
cd Vernacular
```

3. Build and run:
```bash
cargo build
cargo run
```

4. Use the REPL:
```bash
> message as Text is "Hello, World!"
> show message
Hello, World!
```

## Goals

- Create a programming language that feels natural to write and read
- Maintain strong type safety while allowing flexibility when needed
- Bridge the gap between domain experts and developers
- Support modern programming paradigms in an intuitive way

## Status

Vernacular is currently in early development. Core features are being implemented and the language design is being refined.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for:
- Language design suggestions
- Bug reports
- Documentation improvements
- New features

## License

Ancillary License - See LICENSE file for details
