# for-else
[![](https://badgers.space/badge/crates.io/for-else)](https://crates.io/crates/for-else)
[![](https://badgers.space/github/checks/sanchopanca/for-else)](https://github.com/sanchopanca/for-else/actions)
[![](https://badgers.space/badge/%E2%80%8B/docs.rs/orange?icon=eva-book-open-outline)](https://docs.rs/for-else/latest/for_else/index.html)

Python-esque for-else construct for Rust

## Overview

The for-else library introduces two procedural macros, `for_!` and `while_!`, that extend the capabilities of the standard loops in Rust.
This enhancement allows for an else block to be added directly after the loop,
which is executed only if the loop completes without hitting a break statement, closely mirroring Python's for-else and while-else constructs.

## Usage

First, add the dependency to your Cargo.toml:

```
cargo add for-else
```

Then, use the macros in your code:

```rust
use for_else::for_;

// not the best way to test primality, just for demonstration
fn is_prime(n: u32) -> bool {
    if n <= 1 {
        return false;
    }
    for i in 2..n {
        if n % i == 0 {
            return false;
        }
    }
    true
}

for_! { n in 2100..=2110 {
    if is_prime(n) {
        println!("Found a prime number: {}", n);
        break;
    }
} else {
    println!("No prime numbers found in the range.");
}}

```

In this example, the program searches for the first prime number in the range [2100, 2110].
If a prime is found, it prints the number.
If no prime is found, the else block within the for_! macro executes, notifying the user.

## Documentation

For detailed information on each macro and its behavior, please refer to the [documentation](https://docs.rs/for-else/latest)

## Contributing

Contributions are always welcome! Please open an issue or submit a pull request.

## License

[MIT](LICENSE)
