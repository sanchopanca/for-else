//! `for-else` - Enhanced loop control in Rust
//!
//! This crate provides a procedural macro, `for_!`, that enhances
//! the behavior of the standard `for` loop in Rust. It allows for an additional `else` block
//! that gets executed if the loop completes without encountering a `break` statement.
//!
//! # Usage
//!
//! Add the crate to your `Cargo.toml` dependencies and import the macros:
//!
//! ```bash
//! cargo add for-else
//! ```
//!
//! In your Rust code:
//!
//! ```rust
//! use for_else::for_;
//!
//! // not the best way to test primality, just for demonstration
//! fn is_prime(n: u32) -> bool {
//!     if n <= 1 {
//!         return false;
//!     }
//!     for i in 2..n {
//!         if n % i == 0 {
//!             return false;
//!         }
//!     }
//!     true
//! }
//!
//! for_! { n in 2100..=2110 {
//!     if is_prime(n) {
//!         println!("Found a prime number: {}", n);
//!         break;
//!     }
//! } else {
//!     println!("No prime numbers found in the range.");
//! }}
//! ```
//!
//! In this example, the program searches for the first prime number in the range [2100, 2110]. If a prime is found, it prints out the number. If no prime is found in the range, the `else` block within the `for_!` macro is executed, notifying the user.
//!
//! See the `for_!` macro documentation for more detailed examples and usage information.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::token::Brace;
use syn::{
    parse2, parse_macro_input, Block, Expr, ExprBlock, ExprBreak, ExprForLoop, ExprIf, ExprLoop,
    ExprMatch, ExprUnsafe, ExprWhile, Pat, Stmt, Token,
};

struct ForLoop {
    var: Pat,
    expr: Expr,
    body: Block,
    else_block: Block,
    label: Option<syn::Label>,
}

impl Parse for ForLoop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for optional label at the beginning
        let label = if input.peek(syn::Lifetime) && input.peek2(Token![:]) {
            let lifetime: syn::Lifetime = input.parse()?;
            input.parse::<Token![:]>()?;
            Some(syn::Label {
                name: lifetime,
                colon_token: Token![:](proc_macro2::Span::call_site()),
            })
        } else {
            None
        };

        let var = Pat::parse_single(input)?;
        input.parse::<Token![in]>()?;

        // Use a fork to try parsing different amounts of the input as the expression
        // We'll keep extending until we can successfully parse what's left as "{ body } else { else_block }"
        let checkpoint = input.fork();
        let mut expr_tokens = proc_macro2::TokenStream::new();

        // Collect all tokens until we find a valid parse point
        while !input.is_empty() {
            // Check if the remaining input can be parsed as "{ body } else { else_block }"
            let remaining = input.fork();
            if remaining.peek(Brace) {
                // Try to parse: Block else Block
                let test_remaining = remaining.fork();
                if test_remaining.parse::<Block>().is_ok()
                    && test_remaining.peek(Token![else])
                    && test_remaining.peek2(Brace)
                {
                    let _ = test_remaining.parse::<Token![else]>();
                    if test_remaining.parse::<Block>().is_ok() {
                        // Successfully parsed the remaining as "{ body } else { else_block }"
                        break;
                    }
                }
            }

            // Add the next token to our expression
            let tt: proc_macro2::TokenTree = input.parse()?;
            expr_tokens.extend(std::iter::once(tt));
        }

        // Parse the expression from collected tokens
        let expr: Expr = if expr_tokens.is_empty() {
            return Err(syn::Error::new(
                checkpoint.span(),
                "expected expression after 'in'",
            ));
        } else {
            syn::parse2(expr_tokens)?
        };

        let body: Block = input.parse()?;
        input.parse::<Token![else]>()?;
        let else_block: Block = input.parse()?;

        Ok(ForLoop {
            var,
            expr,
            body,
            else_block,
            label,
        })
    }
}

fn modify_breaks_in_block(
    body: &mut Block,
    this_is_my_loop: bool,
    loops_label: Option<&syn::Label>,
) {
    for stmt in &mut body.stmts {
        if let Stmt::Expr(expr, _) = stmt {
            modify_breaks_in_expression(expr, this_is_my_loop, loops_label);
        }
    }
}

fn modify_breaks_in_expression(
    expression: &mut Expr,
    this_is_my_loop: bool,
    loops_label: Option<&syn::Label>,
) {
    match expression {
        Expr::Break(break_expr) => {
            let replacement = modify_single_break(break_expr, this_is_my_loop, loops_label);
            if let Some(replacement) = replacement {
                *expression = parse2(replacement).unwrap();
            }
        }
        Expr::Block(ExprBlock { block, .. }) => {
            modify_breaks_in_block(block, this_is_my_loop, loops_label);
        }
        Expr::Unsafe(ExprUnsafe { block, .. }) => {
            modify_breaks_in_block(block, this_is_my_loop, loops_label);
        }
        Expr::If(ExprIf {
            then_branch,
            else_branch,
            ..
        }) => {
            modify_breaks_in_block(then_branch, this_is_my_loop, loops_label);
            if let Some((_, else_block)) = else_branch {
                if let Expr::Block(ExprBlock { block, .. }) = &mut **else_block {
                    modify_breaks_in_block(block, this_is_my_loop, loops_label);
                }
            }
        }
        Expr::Match(ExprMatch { arms, .. }) => {
            for arm in arms {
                modify_breaks_in_expression(&mut arm.body, this_is_my_loop, loops_label);
            }
        }
        Expr::ForLoop(ExprForLoop { body, .. }) => {
            modify_breaks_in_block(body, false, loops_label);
        }
        Expr::While(ExprWhile { body, .. }) => {
            modify_breaks_in_block(body, false, loops_label);
        }
        Expr::Loop(ExprLoop { body, .. }) => {
            modify_breaks_in_block(body, false, loops_label);
        }
        _ => {}
    }
}

// We need to replace a stement with another statement, but we have two statements instead,
// so we put them into a block to make it a single statement
fn modify_single_break(
    break_expr: &ExprBreak,
    this_is_my_loop: bool,
    loops_label: Option<&syn::Label>,
) -> Option<proc_macro2::TokenStream> {
    let replacement = if let Some(breaks_label) = &break_expr.label {
        // We don't want to touch breaks with labels if it's not our label
        if let Some(loops_label) = loops_label {
            if breaks_label.ident != loops_label.name.ident {
                return None;
            }
        }
        quote! {
            {
                _for_else_break_occurred = true;
                break #breaks_label;
            }
        }
    } else {
        // We don't want to touch breaks in inner loops that don't have our label
        if !this_is_my_loop {
            return None;
        }
        quote! {
            {
                _for_else_break_occurred = true;
                break;
            }
        }
    };

    Some(replacement)
}

/// The `for_!` procedural macro with enhanced loop control.
///
/// This macro is an extension of the standard `for` loop in Rust. It allows users to
/// have an additional `else` block that executes if the loop completed without encountering a `break` statement.
///
/// # Syntax
///
/// ```ignore
/// for_! { variable in iterable {
///     // loop body
/// } else {
///     // else block
/// }}
///
/// // With optional label:
/// for_! { 'label: variable in iterable {
///     // loop body
/// } else {
///     // else block
/// }}
/// ```
///
/// # Behavior
///
/// - The loop iterates over all elements in the iterable
/// - If the loop completes by exhausting all elements, the `else` block is executed
/// - If the loop exits via a `break` statement, the `else` block is **not** executed
/// - `continue` statements work normally and do not affect the `else` block execution
///
/// # Examples
///
/// ## Prime number search
///
/// ```rust
/// use for_else::for_;
///
/// fn is_prime(n: u32) -> bool {
///     if n <= 1 { return false; }
///     for i in 2..n {
///         if n % i == 0 { return false; }
///     }
///     true
/// }
///
/// for_! { n in 2100..=2110 {
///     if is_prime(n) {
///         println!("Found prime: {}", n);
///         break;
///     }
/// } else {
///     println!("No prime numbers found in range");
/// }}
/// ```
///
/// ## Finding an element in a collection
///
/// ```rust
/// use for_else::for_;
///
/// fn find_user(users: &[&str], target: &str) -> bool {
///     for_! { user in users {
///         if *user == target {
///             println!("Found user: {}", user);
///             return true;
///         }
///     } else {
///         println!("User '{}' not found", target);
///     }}
///     false
/// }
///
/// let users = ["alice", "bob", "charlie"];
/// find_user(&users, "dave");  // Prints: User 'dave' not found
/// ```
///
/// ## Validation with early exit
///
/// ```rust
/// use for_else::for_;
///
/// fn validate_data(numbers: &[i32]) -> bool {
///     for_! { &num in numbers {
///         if num < 0 {
///             println!("Invalid negative number found: {}", num);
///             return false;
///         }
///         if num > 100 {
///             println!("Number too large: {}", num);
///             return false;
///         }
///     } else {
///         println!("All numbers are valid");
///         return true;
///     }}
///     false
/// }
/// ```
///
/// ## Working with complex expressions
///
/// ```rust
/// use for_else::for_;
///
/// struct DataSource;
/// impl DataSource {
///     fn items(&self) -> impl Iterator<Item = i32> {
///         vec![1, 2, 3, 4, 5].into_iter()
///     }
/// }
///
/// // Note: Complex expressions may need parentheses
/// for_! { item in (DataSource {}).items() {
///     if item > 10 {
///         println!("Found large item: {}", item);
///         break;
///     }
/// } else {
///     println!("No large items found");
/// }}
/// ```
///
/// ## Nested loops with labels
///
/// ```rust
/// use for_else::for_;
///
/// for_! { 'outer: i in 0..3 {
///     for_! { j in 0..3 {
///         if i == 1 && j == 1 {
///             break 'outer;  // Break outer loop
///         }
///         println!("({}, {})", i, j);
///     } else {
///         println!("Inner loop {} completed", i);
///     }}
/// } else {
///     println!("Both loops completed naturally");
/// }}
/// ```
///
/// # Comparison with Python
///
/// This macro brings Python-like `for-else` behavior to Rust:
///
/// **Python:**
/// ```python
/// for item in iterable:
///     # loop body
///     if some_condition:
///         break
/// else:
///     # executed if loop completed without break
///     pass
/// ```
///
/// **Rust with `for_!`:**
/// ```ignore
/// for_! { item in iterable {
///     // loop body
///     if some_condition {
///         break;
///     }
/// } else {
///     // executed if loop completed without break
/// }}
/// ```
///
/// # Notes
///
/// - The macro supports all the same iterables as standard `for` loops
/// - Loop labels work normally for controlling nested loops
/// - Complex expressions in the iterable position may require parentheses due to Rust's parsing rules
#[proc_macro]
pub fn for_(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ForLoop);

    let label = input.label;

    modify_breaks_in_block(&mut input.body, true, label.as_ref());

    let var = input.var;
    let expr = input.expr;
    let body = input.body;
    let else_block = input.else_block;

    let expanded = if let Some(label) = label {
        quote! {
            {
                let mut _for_else_break_occurred = false;
                #label for #var in #expr
                    #body
                if !_for_else_break_occurred
                    #else_block
            }
        }
    } else {
        quote! {
            {
                let mut _for_else_break_occurred = false;
                for #var in #expr
                    #body
                if !_for_else_break_occurred
                    #else_block
            }
        }
    };

    expanded.into()
}

struct WhileLoop {
    cond: Expr,
    body: Block,
    else_block: Block,
    label: Option<syn::Label>,
}

impl Parse for WhileLoop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for optional label at the beginning
        let label = if input.peek(syn::Lifetime) && input.peek2(Token![:]) {
            let lifetime: syn::Lifetime = input.parse()?;
            input.parse::<Token![:]>()?;
            Some(syn::Label {
                name: lifetime,
                colon_token: Token![:](proc_macro2::Span::call_site()),
            })
        } else {
            None
        };

        // Use the same lookahead approach as for_! macro
        let checkpoint = input.fork();
        let mut cond_tokens = proc_macro2::TokenStream::new();

        // Collect all tokens until we find a valid parse point
        while !input.is_empty() {
            // Check if the remaining input can be parsed as "{ body } else { else_block }"
            let remaining = input.fork();
            if remaining.peek(Brace) {
                // Try to parse: Block else Block
                let test_remaining = remaining.fork();
                if test_remaining.parse::<Block>().is_ok()
                    && test_remaining.peek(Token![else])
                    && test_remaining.peek2(Brace)
                {
                    let _ = test_remaining.parse::<Token![else]>();
                    if test_remaining.parse::<Block>().is_ok() {
                        // Successfully parsed the remaining as "{ body } else { else_block }"
                        break;
                    }
                }
            }

            // Add the next token to our condition expression
            let tt: proc_macro2::TokenTree = input.parse()?;
            cond_tokens.extend(std::iter::once(tt));
        }

        // Parse the condition from collected tokens
        let cond: Expr = if cond_tokens.is_empty() {
            return Err(syn::Error::new(
                checkpoint.span(),
                "expected condition expression",
            ));
        } else {
            syn::parse2(cond_tokens)?
        };

        let body: Block = input.parse()?;
        input.parse::<Token![else]>()?;
        let else_block: Block = input.parse()?;

        Ok(WhileLoop {
            cond,
            body,
            else_block,
            label,
        })
    }
}

/// The `while_!` procedural macro with enhanced loop control.
///
/// This macro is an extension of the standard `while` loop in Rust. It allows users to
/// have an additional `else` block that executes if the loop completed without encountering a `break` statement.
///
/// # Syntax
///
/// ```ignore
/// while_! { condition {
///     // loop body
/// } else {
///     // else block
/// }}
///
/// // With optional label:
/// while_! { 'label: condition {
///     // loop body
/// } else {
///     // else block
/// # Notes
///
/// - The macro supports all the same conditions as standard `while` loops
/// - Loop labels work normally for controlling nested loops
/// - Complex expressions in the condition position are fully supported
///
/// # Behavior
///
/// - The loop continues to execute as long as the `condition` evaluates to `true`
/// - If the loop exits naturally (condition becomes `false`), the `else` block is executed
/// - If the loop exits via a `break` statement, the `else` block is **not** executed
/// - `continue` statements work normally and do not affect the `else` block execution
///
/// # Examples
///
/// ## Basic usage
///
/// ```rust
/// use for_else::while_;
///
/// let mut count = 0;
/// let mut found = false;
///
/// while_! { count < 5 {
///     if count == 10 {  // This condition is never true
///         found = true;
///         break;
///     }
///     count += 1;
/// } else {
///     println!("Loop completed without finding target value");
/// }}
///
/// assert!(!found);  // Loop completed naturally, else block executed
/// ```
///
/// ## Search with early termination
///
/// ```rust
/// use for_else::while_;
///
/// fn find_target(data: &[i32], target: i32) -> Option<usize> {
///     let mut index = 0;
///     let mut result = None;
///     
///     while_! { index < data.len() {
///         if data[index] == target {
///             result = Some(index);
///             break;  // Found it, exit early
///         }
///         index += 1;
///     } else {
///         println!("Target {} not found in data", target);
///     }}
///     
///     result
/// }
/// ```
///
/// ## Retry mechanism with timeout
///
/// ```rust
/// use for_else::while_;
/// use std::time::{Duration, Instant};
///
/// fn retry_operation() -> bool {
///     let start = Instant::now();
///     let timeout = Duration::from_secs(5);
///     
///     while_! { start.elapsed() < timeout {
///         if attempt_operation() {
///             println!("Operation succeeded!");
///             return true;
///         }
///         std::thread::sleep(Duration::from_millis(100));
///     } else {
///         println!("Operation timed out after 5 seconds");
///     }}
///     
///     false
/// }
///
/// fn attempt_operation() -> bool {
///     // Some operation that might succeed or fail
///     false
/// }
/// ```
///
/// ## Nested loops with labels
///
/// ```rust
/// use for_else::while_;
///
/// let mut found_target = false;
/// let mut outer_count = 0;
///
/// while_! { 'outer: outer_count < 5 {
///     let mut inner_count = 0;
///     while_! { inner_count < 5 {
///         if outer_count == 2 && inner_count == 3 {
///             println!("Found target at ({}, {})", outer_count, inner_count);
///             found_target = true;
///             break 'outer;  // Break outer loop
///         }
///         inner_count += 1;
///     } else {
///         println!("Inner loop completed for outer_count = {}", outer_count);
///     }}
///     outer_count += 1;
/// } else {
///     println!("Search completed without early termination");
/// }}
///
/// assert!(found_target);
/// ```
///
/// # Comparison with Python
///
/// This macro brings Python-like `while-else` behavior to Rust:
///
/// **Python:**
/// ```python
/// while condition:
///     # loop body
///     if some_condition:
///         break
/// else:
///     # executed if loop completed without break
///     pass
/// ```
///
/// **Rust with `while_!`:**
/// ```ignore
/// while_! { condition {
///     // loop body
///     if some_condition {
///         break;
///     }
/// } else {
///     // executed if loop completed without break
/// }}
/// ```
#[proc_macro]
pub fn while_(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as WhileLoop);

    let label = input.label;

    modify_breaks_in_block(&mut input.body, true, label.as_ref());

    let cond = input.cond;
    let body = input.body;
    let else_block = input.else_block;

    let expanded = if let Some(label) = label {
        quote! {
            {
                let mut _for_else_break_occurred = false;
                #label while #cond
                    #body
                if !_for_else_break_occurred
                    #else_block
            }
        }
    } else {
        quote! {
            {
                let mut _for_else_break_occurred = false;
                while #cond
                    #body
                if !_for_else_break_occurred
                    #else_block
            }
        }
    };

    expanded.into()
}
