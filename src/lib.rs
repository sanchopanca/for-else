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
use syn::{
    parse2, parse_macro_input, Block, Expr, ExprBlock, ExprBreak, ExprIf, ExprMatch, Pat, Result,
    Stmt, Token,
};

struct ForLoop {
    var: Pat,
    expr: Expr,
    body: Block,
    else_block: Block,
}

impl Parse for ForLoop {
    fn parse(input: ParseStream) -> Result<Self> {
        let var = Pat::parse_single(input)?;
        input.parse::<Token![in]>()?;
        let expr: Expr = input.parse()?;
        let body: Block = input.parse()?;
        input.parse::<Token![else]>()?;
        let else_block: Block = input.parse()?;
        Ok(ForLoop {
            var,
            expr,
            body,
            else_block,
        })
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

/// The `for_!` procedural macro with enhanced loop control.
///
/// This macro is an extension of the standard `for` loop in Rust. It allows users to
/// have an additional `else` block that executes if the loop completed without encountering a `break` statement.
///
/// # Syntax
///
/// ```ignore
/// for_! { variable in expression {
///     // loop body
/// } else {
///     // else block
/// }}
/// ```
///
/// # Example
///
/// ```rust
/// use for_else::for_;
///
/// # fn some_condition(i: u32) -> bool {
/// #     true
/// # }
/// # fn main() {
/// for_! { i in 0..10 {
///     if some_condition(i) {
///         // Some action
///         break;
///     }
/// } else {
///     // This block executes if the loop never breaks
/// }}
/// # }
/// ```
///
/// In the example above, if `some_condition(i)` never evaluates to `true` for any `i` in the range `0..10`,
/// then the `else` block will be executed after the loop completes.
#[proc_macro]
pub fn for_(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ForLoop);

    modify_breaks(&mut input.body);

    let var = input.var;
    let expr = input.expr;
    let body = input.body;
    let else_block = input.else_block;

    let expanded = quote! {
        let mut _for_else_break_occurred = false;
        for #var in #expr
            #body
        if !_for_else_break_occurred
            #else_block

    };

    expanded.into()
}
