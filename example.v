# Natural Language Syntax Exploration

# Variable Declarations with Type Annotations (similar to TypeScript/Java)
count as Whole                  # int/Integer type
measure as Decimal is 3.14      # float/double type
message as Text                 # string type
flag as Logic                   # boolean type
empty1 as Nothing               # Nothing type with a null value
empty2 is null                  # null value as an Any type

count is 0
message as Text is "Hello, World!"

# Strongly-Typed Collections (analogous to Java Generics/TypeScript Arrays)
number_collection is [1, 2, 3, 4, 5] as List[Whole]  # Generic List<int>

# Dynamic Mapping with Static Type Annotation
person_info as Mapping \
    of Text to Any includes \             # Dictionary<string, object> or Map<String, Object>
    name as Text is "John",
    age as Whole is 30,
    is_student as Logic is false

# Function Declaration with Pattern Matching (similar to Scala/F# match expressions)
Task process requires first, second, action as Number, Number, Text returning Whole:
    when action is "add":           # Case/Switch statement equivalent
        output first + second
    when action is "multiply":
        output first * second
    or:                             # default case
        raise "Unknown action" as Error

# Class Definition with Constructor and Inheritance (OOP style like Java/C#)
Object Person inherits BaseEntity:
    instance as secret Person
    name as Text
    age as Whole

    build defaults name is "Unknown", age is 0:
        my name is name                                 # 'my' is equivalent to 'this' or 'self'
        my age is age
    
    Task greetings returns Text:
        output "I am {my name}, {my age} years old"     # String interpolation
    
    Task singleton returns Person:
        when my instance is Nothing:
            my instance is new Person()
    
        output my instance

# Exception Handling (try-catch-finally block)
Task possible_risk:
    do:                                     # try block
        result is 10 / 0
    fail problem as DivideByZeroError:      # catch block
        show "Encountered: {problem.message}"
    always:                                 # finally block
        show "Examination finished"

# Functional Programming Operations (similar to LINQ/Stream operations)
numbers is [1, 2, 3, 4, 5] as List[Whole]
squared is numbers and each number becomes number * number  # map() operation
filtered is numbers when each number > 2                    # filter() operation

# Asynchronous Function (similar to async/await in JS/C#)
Task gather_data requires url as Text, returns Promise[Text]:
    response as Text is awaiting http.fetch at url    # await keyword for async operations
    output response.content

# Type Pattern Matching (similar to Rust/Scala match expressions)
Task describe_value requires value as Any returns Text:
    match value:                # Pattern matching on types
        when Whole:             # Type case branches
            output "This is a whole number"
        when Decimal: 
            output "This is a measured number"
        when Text: 
            output "This is a message"
        or: 
            output "Unknown type"

# Generator Function (similar to Python generators) # TODO: Implement as iterator/generator
Stream fibonacci requires max as Whole, emitting Whole:
    first as Whole is 0
    second as Whole is 1
    
    loop while true:            # Infinite loop with Emit
        emit first            # Not yet implemented
        next as Whole is first + second
        first is second
        second is next
        show "first: {first}, second: {second}"
        
        when first > max:
            output first

# Another stream example with a different syntax
Stream primes emits Whole requiring max as Whole:
    Task isPrime returns Logic requires number as Whole:
        when number < 2:
            output false
        
        #Iterate .. 

# TODO: Create a Lambda type
lambda_example as Auto returns Whole a + Whole b

# Program Entry Point (similar to main() in C/Java)
main:
    alice as Person is new Person using "Alice", 25  # Object instantiation
    show alice's greetings
    
    result as Whole is process using 5, 3, "multiply"
    show "Calculation outcome: {result}"
    
    show "Fibonacci sequence up to 100:" as Text
    fibonacci using 100

    show "Describe value:"
    show describe_value using 10
    show describe_value using 3.14
    show describe_value using "Hello, World!"
    show describe_value using true

    show "Lambda:"
    show lambda_example using 1, 2

    show "Gather data:"
    data as Text is awaiting gather_data using "https://example.com"
    show data

    show "Possible risk:"
    show possible_risk
