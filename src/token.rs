use std::ops::Range;

use logos::Logos;

use crate::error::ParseError;

pub type SpannedToken<'a> = (Token<'a>, Range<usize>);

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token<'a> {
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("^")]
    Caret,
    #[token("_")]
    Underscore,
    #[token("'")]
    Prime,
    #[token("|")]
    Pipe,
    #[token("!")]
    Bang,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("=")]
    Equals,
    #[token("*")]
    Star,
    #[token(",")]
    Comma,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("/")]
    Slash,

    #[token("&")]
    Ampersand,

    #[regex(r"\\[a-zA-Z]+", |lex| &lex.slice()[1..])]
    Command(&'a str),

    #[regex(r"\\[^a-zA-Z]", |lex| &lex.slice()[1..])]
    Escape(&'a str),

    #[regex(r"[0-9]+")]
    Number(&'a str),

    // can also be without a capture group but with is a bit nicer
    #[regex(r"([a-zA-Z]|[^ -~\s])")]
    Ident(&'a str),

    #[regex(r"\s+", logos::skip)]
    Whitespace,
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken<'_>>, ParseError> {
    Token::lexer(input)
        .spanned()
        .map(|(i, span)| {
            i.map(|i| (i, span.clone()))
                .map_err(|_| ParseError::from_range(span))
        })
        .collect()
}
