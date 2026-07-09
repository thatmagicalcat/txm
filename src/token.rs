use logos::Logos;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
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

    #[regex(r"\\[a-zA-Z]+", |lex| lex.slice()[1..].to_string())]
    Command(String),

    #[regex(r"\\[^a-zA-Z]", |lex| lex.slice()[1..].to_string())]
    Escape(String),

    #[regex(r"[0-9]+", |lex| lex.slice().to_string())]
    Number(String),

    //can also be without a capture group but with is a bit nicer
    #[regex(r"([a-zA-Z]|[^ -~\s])+", |lex| lex.slice().to_string())] 
    Ident(String),

    #[regex(r"\s+", logos::skip)]
    Whitespace,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    Token::lexer(input).filter_map(|t| t.ok()).collect()
}
