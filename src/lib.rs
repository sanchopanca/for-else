//! `for-else` - Enhanced loop control in Rust
//!
//! This crate provides two procedural macros, `for_!` and `else_!`, that enhance
//! the behavior of the standard `for` loop in Rust.
//!
//! The `for_!` macro functions similarly to a standard `for` loop but pairs with the `else_!`
//! macro to detect if the loop exited without encountering a `break` statement.
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
//! use for_else::{for_, else_};
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
//! fn main() {
//!     for_! { n in 2100..=2110 {
//!         if is_prime(n) {
//!             println!("Found a prime number: {}", n);
//!             break;
//!         }
//!     }}
//!     else_! {
//!         println!("No prime numbers found in the range.");
//!     }
//! }

//! ```
//!
//! In this example, the program searches for the first prime number in the range [2100, 2110]. If a prime is found, it prints out the number. If no prime is found in the range, the `else_!` block is executed, notifying the user.
//!
//! See the individual macro documentation (`for_!` and `else_!`) for more detailed examples and usage information.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    parse2, parse_macro_input, Block, Expr, ExprBlock, ExprBreak, ExprIf, ExprMatch, Pat, Result,
    Stmt, Token,
};

struct ForLoop {
    var: Pat,
    expr: Expr,
    body: Block,
}

impl Parse for ForLoop {
    fn parse(input: ParseStream) -> Result<Self> {
        let var = Pat::parse_single(input)?;
        input.parse::<Token![in]>()?;
        let expr: Expr = input.parse()?;
        let body: Block = input.parse()?;
        Ok(ForLoop { var, expr, body })
    }
}

fn modify_breaks(body: &mut Block) {
    for stmt in &mut body.stmts {
        match stmt {
            Stmt::Expr(Expr::Break(ExprBreak { label, .. }), _) => {
                // we need to replace a stement with another statement, but we have two statements instead,
                // so we put them into a block to make it a single statement
                let replacement = if let Some(label) = label {
                    quote! {
                        {
                            _for_else_break_occurred = true;
                            break #label;
                        }
                    }
                } else {
                    quote! {
                        {
                            _for_else_break_occurred = true;
                            break;
                        }
                    }
                };

                *stmt = parse2(replacement).unwrap();
            }
            Stmt::Expr(Expr::Block(ExprBlock { block, .. }), _) => {
                modify_breaks(block);
            }
            Stmt::Expr(
                Expr::If(ExprIf {
                    then_branch,
                    else_branch,
                    ..
                }),
                _,
            ) => {
                modify_breaks(then_branch);
                if let Some((_, else_block)) = else_branch {
                    if let Expr::Block(ExprBlock { block, .. }) = &mut **else_block {
                        modify_breaks(block);
                    }
                }
            }
            Stmt::Expr(Expr::Match(ExprMatch { arms, .. }), _) => {
                for arm in arms {
                    match &mut *arm.body {
                        Expr::Break(ExprBreak { label, .. }) => {
                            let replacement = if let Some(label) = label {
                                quote! {
                                    {
                                        _for_else_break_occurred = true;
                                        break #label;
                                    }
                                }
                            } else {
                                quote! {
                                    {
                                        _for_else_break_occurred = true;
                                        break;
                                    }
                                }
                            };

                            arm.body = Box::new(syn::parse2(replacement).unwrap());
                        }
                        Expr::Block(ExprBlock { block, .. }) => modify_breaks(block),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

/// The `for_!` procedural macro.
///
/// This macro is an extension of the standard `for` loop in Rust. It functions
/// similarly but provides an additional mechanism to detect if the loop exited
/// without encountering a `break` statement.
///
/// # Usage
///
/// The syntax for this macro is nearly identical to a standard `for` loop:
///
/// ```ignore
/// for_! { variable in expression {
///     // loop body
/// }}
/// ```
///
/// # Example
///
/// ```
/// # use for_else::for_;
/// let mut found = false;
/// for_! { i in 0..10 {
///     if i == 5 {
///         found = true;
///         break;
///     }
/// }}
/// ```
///
/// Pair this macro with the `else_!` macro to execute code when the loop does not break.
///
/// See the documentation for `else_!` for more details.
///
/// Note: Make sure that the paired `else_!` macro immediately follows the `for_!` macro, without any statements in between.
#[proc_macro]
pub fn for_(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ForLoop);

    modify_breaks(&mut input.body);

    let var = input.var;
    let expr = input.expr;
    let body = input.body;

    let expanded = quote! {
        let mut _for_else_break_occurred = false;
        for #var in #expr
            #body
    };

    expanded.into()
}

struct Statements {
    stmts: Vec<Stmt>,
}

impl Parse for Statements {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut stmts = Vec::new();

        while !input.is_empty() {
            stmts.push(input.parse()?);
        }

        Ok(Statements { stmts })
    }
}

/// The `else_!` procedural macro.
///
/// This macro is meant to be used in conjunction with the `for_!` macro. It allows
/// you to execute a block of code if the preceding `for_!` loop exits without encountering
/// a `break` statement.
///
/// # Usage
///
/// Place the `else_!` macro immediately after a `for_!` loop:
///
/// ```ignore
/// for_! { ... }
/// else_! {
///     // block of code
/// }
/// ```
///
/// You can also use single or multiple statements:
///
/// ```ignore
/// for_! { ... }
/// else_! { println!("Loop did not break"); }
/// ```
///
/// # Example
///
/// ```
/// # use for_else::{for_, else_};
/// let mut flag = false;
/// for_! { i in 1..10 {
///     if i % 10 == 0 {
///         break;
///     }
/// }}
/// else_! {
///     flag = true;
/// }
/// ```
///
/// In this example the loop exits without breaking, so `flag` will be set to `true`.
#[proc_macro]
pub fn else_(input: TokenStream) -> TokenStream {
    let Statements { stmts } = parse_macro_input!(input as Statements);

    let expanded = quote! {
        if !_for_else_break_occurred {
            #( #stmts )*
        }
    };

    expanded.into()
}
