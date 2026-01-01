# The Q Programming Language

# Quentin Compilor

```
# Build an entire project
quentin build

# Build one file only
quentin build file.q

# Build with logs
quentin build --log

# Run the app
quentin run latest

# Clear build cache
quentin clear cache
```

## Idea

```q
system.include{
    from filename.q import{
        "name1": filename::objekt // Zum Beispiel eine Funkion
        "name2": filename::objekt // Zum Beispiel eine Variable
    },
    from filename2.q import{
        "name3": filename::objekt // Zum Beispiel eine Funkion
        "name4": filename::objekt // Zum Beispiel eine Variable
    }
}

// variable

system.init{
    "type": variable/array,
    "name": var1,
    "datatype": string/number/bool,

    // value dann brauchen wenn man die Variable beim erstellen befüllen möchte

    "value": "Hello World!"/ 99/ true
};

// Variable befüllen / ändern
system.set{
    "name": var1,
    "value": "Hallo Welt!"
};

// logging

system.log{
    "type": info/warn/error,
    arguments{
        var1.value/type
    },
    "message": "Wert:" & var1.value
};

function name(param1 in number, param2 in number){
    system.log{
        "type": info,
        arguments{
            param1.value, 
            param2.value
        },
        "message": "Ergebniss: " & (param1.value + param2.value)
    };
    return null;
};

system.exec{
    "type": function,
    "name": name,
    parameters{
        param1 => 2,
        param2 => 3
    }
};
```

---

## Progress Update

I have made significant progress on the Q language compiler. Here is a summary of the work done so far:

*   **Language Specification:** Analyzed the `README.md` file to understand the syntax and features of the Q language.
*   **Test File:** Created a `test.q` file containing a sample Q program to be used for testing the compiler.
*   **Project Setup:** Configured the Rust project with the necessary dependencies (`clap` for CLI and `pest` for parsing). The package has been named `quentin`.
*   **Parser:** Implemented a `pest` grammar (`src/q.pest`) that defines the syntax of the Q language.
*   **Abstract Syntax Tree (AST):** Defined the AST structures in `src/ast.rs` to represent the Q language constructs in a typed manner.
*   **AST Builder:** Implemented the logic in `src/main.rs` to parse the Q code and transform it into an AST.
*   **Compiler Errors Fixed:** Addressed and fixed several batches of compilation errors.
*   **Interpreter:** Implemented a basic interpreter in `src/interpreter.rs` that can walk the AST and execute the Q program. The interpreter supports variables, assignments, logging, and function calls.
*   **Bug Fixes:** Fixed an issue where the interpreter would not produce any output due to an error in the AST building process. Fixed a panic that occurred when parsing comments. Fixed a bug that caused string literals to be parsed as empty strings.

## How to Run

1.  Build the project:
    ```
    cargo build
    ```
2.  Run the interpreter on the test file:
    ```
    ./target/debug/quentin build test.q
    ```

This will parse the `test.q` file, build the AST, and execute it with the interpreter.

## Next Steps

*   **Testing:** Thoroughly test the parser, AST builder, and interpreter to ensure they work correctly for all language features.
*   **Error Handling:** Implement robust error handling for both parsing and interpretation.
*   **Feature Expansion:** Add support for more complex features like arrays, modules (`system.include`), and more built-in functions.
*   **Code Generation:** As an alternative to interpretation, implement a code generator to compile Q code to another target, such as Rust or LLVM IR.