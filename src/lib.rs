#![allow(dead_code)]
#![allow(unused_imports)]
#![deny(unused_must_use)]

#[macro_use]
mod error;
mod ast;
mod emit;
mod parser;
mod scanner;
mod stmt;
mod tag;
mod token;

pub use {
    ast::AST,
    emit::emit,
    error::{print_error, Error},
    parser::parse,
    scanner::scan,
    stmt::{Expr, Stmt},
    tag::Tag,
    token::{Token, TokenKind, TokenStream},
};

pub type Result<T> = std::result::Result<T, Error>;

pub fn compile(_ast: AST) -> Result<String> {
    Ok(String::new())
}

pub fn to_html(source: &str) -> Result<String> {
    Ok(source.to_string())
}
