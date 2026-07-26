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

    #[token("?")]
    Question,
    #[token("@")]
    At,
    #[token("#")]
    Hash,
    #[token("$")]
    Dollar,
    #[token("%")]
    Percent,
    #[token("~")]
    Tilde,
    #[token("\"")]
    Quote,
    #[token("`")]
    Backtick,

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

impl<'a> Token<'a> {
    pub fn text_char(&self) -> Option<char> {
        match self {
            Token::Bang => Some('!'),
            Token::Question => Some('?'),
            Token::Comma => Some(','),
            Token::Dot => Some('.'),
            Token::Colon => Some(':'),
            Token::Semicolon => Some(';'),
            Token::Slash => Some('/'),
            Token::Less => Some('<'),
            Token::Greater => Some('>'),
            Token::Plus => Some('+'),
            Token::Minus => Some('-'),
            Token::Equals => Some('='),
            Token::Star => Some('*'),
            Token::Pipe => Some('|'),
            Token::Ampersand => Some('&'),
            Token::LParen => Some('('),
            Token::RParen => Some(')'),
            Token::LBracket => Some('['),
            Token::RBracket => Some(']'),
            Token::LBrace => Some('{'),
            Token::RBrace => Some('}'),
            Token::Prime => Some('\''),
            Token::Caret => Some('^'),
            Token::Underscore => Some('_'),
            Token::At => Some('@'),
            Token::Hash => Some('#'),
            Token::Dollar => Some('$'),
            Token::Percent => Some('%'),
            Token::Tilde => Some('~'),
            Token::Quote => Some('"'),
            Token::Backtick => Some('`'),
            Token::Ident(s) => s.chars().next(),
            Token::Number(s) if s.len() == 1 => s.chars().next(),
            Token::Escape(s) => s.chars().next(),
            _ => None,
        }
    }

    pub fn as_char(&self) -> Option<char> {
        match self {
            Token::LParen | Token::Escape("(") => Some('('),
            Token::RParen | Token::Escape(")") => Some(')'),
            Token::LBracket | Token::Escape("[") => Some('['),
            Token::RBracket | Token::Escape("]") => Some(']'),
            Token::LBrace | Token::Escape("{") => Some('{'),
            Token::RBrace | Token::Escape("}") => Some('}'),
            Token::Pipe | Token::Escape("|") => Some('|'),
            _ => None,
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken<'_>>, ParseError> {
    Token::lexer(input)
        .spanned()
        .map(|(i, span)| {
            i.map(|i| (i, span.clone()))
                .map_err(|_| ParseError::InvalidToken { byte: span.start })
        })
        .collect()
}
