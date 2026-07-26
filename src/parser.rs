use std::ops::Range;

use crate::ParseError;
use crate::ast::*;
use crate::glyph::SymbolRegistry;
use crate::token::SpannedToken;
use crate::token::Token;

pub struct Parser<'a> {
    tokens: &'a [SpannedToken<'a>],
    input: &'a str,
    pos: usize,
    registry: &'a SymbolRegistry,
}

impl<'a> Parser<'a> {
    pub fn new(
        input: &'a str,
        tokens: &'a [SpannedToken<'a>],
        registry: &'a SymbolRegistry,
    ) -> Self {
        Self {
            input,
            tokens,
            pos: 0,
            registry,
        }
    }

    fn peek(&self) -> Option<&Token<'_>> {
        self.tokens.get(self.pos).map(|(i, _)| i)
    }

    fn current_span(&self) -> Option<&Range<usize>> {
        self.tokens.get(self.pos).map(|(_, j)| j)
    }

    fn advance(&mut self) -> Token<'a> {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t.0
    }

    fn expect(&mut self, tok: Token) -> Result<(), ParseError> {
        match self.peek() {
            None => Err(ParseError::ExpectedToken {
                expected: format!("{tok:?}"),
            }
            .at_eof(self.input)),
            Some(actual) if *actual == tok => {
                self.advance();
                Ok(())
            }
            Some(actual) => Err(ParseError::ExpectedTokenGot {
                expected: format!("{tok:?}"),
                got: format!("{actual:?}"),
            }
            .at(self.current_span().unwrap().clone(), self.input)),
        }
    }

    /// For input = "xyz", the tokenizer returns:
    /// `[(Ident("x"), 0..1), (Ident("y"), 1..2), (Ident("z"), 2..3)]`
    ///
    /// This function takes that stream of tokens and returns Ok("xyz")
    /// if those tokens are continuous. If the input string had whitespaces,
    /// for example, "x y z", the tokenizer returns:
    /// `[(Ident("x"), 0..1), (Ident("y"), 2..3), (Ident("z"), 4..5)]`
    /// it returns an error.
    fn parse_continuous_string(&mut self, deliminted_by: Token) -> Result<String, ParseError> {
        let mut name = String::new();
        let mut last_end: Option<usize> = None;

        loop {
            if let Some(Token::Ident(segment)) = self.peek() {
                let span = self.current_span().ok_or(ParseError::UnexpectedEof)?;

                if let Some(prev_end) = last_end
                    && prev_end != span.start
                {
                    return Err(ParseError::UnexpectedWhitespace.at(span.clone(), self.input));
                }

                name.push_str(segment);
                last_end = Some(span.end);
                self.advance();
            } else if self.peek() == Some(&deliminted_by) {
                break;
            } else {
                return Err(ParseError::ExpectedString.at(
                    self.current_span()
                        .ok_or(ParseError::UnexpectedEof)?
                        .clone(),
                    self.input,
                ));
            }
        }

        Ok(name)
    }

    fn parse_string_arg(&mut self) -> Result<String, ParseError> {
        self.expect(Token::LBrace)?;
        let mut name = String::new();
        let mut last_end: Option<usize> = None;

        loop {
            match self.peek() {
                Some(Token::Ident(segment)) | Some(Token::Number(segment)) => {
                    let span = self.current_span().ok_or(ParseError::UnexpectedEof)?;

                    if let Some(prev_end) = last_end
                        && prev_end != span.start
                    {
                        name.push(' ');
                    }

                    name.push_str(segment);
                    last_end = Some(span.end);
                    self.advance();
                }
                Some(Token::RBrace) => break,
                Some(tok) if tok.text_char().is_some() => {
                    let span = self.current_span().ok_or(ParseError::UnexpectedEof)?;

                    if let Some(prev_end) = last_end
                        && prev_end != span.start
                    {
                        name.push(' ');
                    }

                    name.push(tok.text_char().unwrap());
                    last_end = Some(span.end);
                    self.advance();
                }
                _ => {
                    return Err(ParseError::ExpectedString.at(
                        self.current_span()
                            .ok_or(ParseError::UnexpectedEof)?
                            .clone(),
                        self.input,
                    ));
                }
            }
        }

        self.advance(); // eat RBrace
        Ok(name)
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_binop()
    }

    fn parse_binop(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_juxtapose()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let rhs = self.parse_juxtapose()?;
                    lhs = Expr::BinOp(Box::new(lhs), BinOp::Add, Box::new(rhs));
                }
                Some(Token::Minus) => {
                    self.advance();
                    let rhs = self.parse_juxtapose()?;
                    lhs = Expr::BinOp(Box::new(lhs), BinOp::Sub, Box::new(rhs));
                }
                Some(Token::Equals) => {
                    self.advance();
                    let rhs = self.parse_juxtapose()?;
                    lhs = Expr::BinOp(Box::new(lhs), BinOp::Eq, Box::new(rhs));
                }
                Some(Token::Star) => {
                    self.advance();
                    let rhs = self.parse_juxtapose()?;
                    lhs = Expr::BinOp(Box::new(lhs), BinOp::Mul, Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn can_start_atom(&self) -> bool {
        matches!(
            self.peek(),
            Some(Token::LBrace)
                | Some(Token::LParen)
                | Some(Token::RParen)
                | Some(Token::LBracket)
                | Some(Token::RBracket)
                | Some(Token::Number(_))
                | Some(Token::Ident(_))
                | Some(Token::Command(_))
                | Some(Token::Escape(_))
                | Some(Token::Bang)
                | Some(Token::Pipe)
                | Some(Token::Minus)
                | Some(Token::Plus)
                | Some(Token::Ampersand)
                | Some(Token::Slash)
                | Some(Token::Comma)
                | Some(Token::Dot)
                | Some(Token::Colon)
                | Some(Token::Semicolon)
                | Some(Token::Less)
                | Some(Token::Greater)
                | Some(Token::Question)
                | Some(Token::At)
                | Some(Token::Hash)
                | Some(Token::Dollar)
                | Some(Token::Percent)
                | Some(Token::Tilde)
                | Some(Token::Quote)
                | Some(Token::Backtick)
        )
    }

    fn parse_juxtapose(&mut self) -> Result<Expr, ParseError> {
        if !self.can_start_atom() {
            return Ok(Expr::Empty);
        }

        let mut exprs = Vec::new();
        exprs.push(self.parse_scripted()?);

        while let Some(Token::LBrace)
        | Some(Token::LParen)
        | Some(Token::RParen)
        | Some(Token::LBracket)
        | Some(Token::RBracket)
        | Some(Token::Number(_))
        | Some(Token::Ident(_))
        | Some(Token::Command(_))
        | Some(Token::Escape(_))
        | Some(Token::Bang)
        | Some(Token::Ampersand)
        | Some(Token::Slash)
        | Some(Token::Comma)
        | Some(Token::Dot)
        | Some(Token::Colon)
        | Some(Token::Semicolon)
        | Some(Token::Less)
        | Some(Token::Greater)
        | Some(Token::Question)
        | Some(Token::At)
        | Some(Token::Hash)
        | Some(Token::Dollar)
        | Some(Token::Percent)
        | Some(Token::Tilde)
        | Some(Token::Quote)
        | Some(Token::Backtick) = self.peek()
        {
            exprs.push(self.parse_scripted()?);
        }

        if exprs.len() == 1 {
            Ok(exprs.into_iter().next().unwrap())
        } else {
            Ok(Expr::Juxtapose(exprs))
        }
    }

    fn parse_scripted(&mut self) -> Result<Expr, ParseError> {
        let base = self.parse_atom()?;
        let mut sub: Option<Box<Expr>> = None;
        let mut sup: Option<Box<Expr>> = None;
        let mut primes: usize = 0;

        loop {
            match self.peek() {
                Some(Token::Underscore) if sub.is_none() => {
                    self.advance();
                    sub = Some(Box::new(self.parse_atom()?));
                }
                Some(Token::Caret) if sup.is_none() => {
                    self.advance();
                    sup = Some(Box::new(self.parse_atom()?));
                }
                Some(Token::Prime) => {
                    self.advance();
                    primes += 1;
                }
                _ => break,
            }
        }

        // i want to parse things like `\int_a^b{argument}`
        // we've already parsed the base: `\int`, scripts: `_a^b`
        // now we check if the base was present in symbol registry and
        // has_limits = true and if the next symbol after parsing scripts
        // was LBrace, it is an argument

        let mut result = if let Expr::Command {
            name,
            opts: _,
            args,
        } = &base
            && let Some(glyph) = self.registry.get(name)
            && glyph.has_limits()
            && self.peek() == Some(&Token::LBrace)
            && args.len() < glyph.required_args()
        {
            // move
            let Expr::Command { name, mut args, .. } = base else {
                return Err(ParseError::Internal(
                    "limits argument base was not a command".into(),
                ));
            };

            self.advance(); // eat {
            let body = self.parse_expr()?;
            self.expect(Token::RBrace)?;
            args.push(body);

            Expr::Command {
                name,
                opts: Vec::new(),
                args,
            }
        } else {
            base
        };

        if primes > 0 {
            result = Expr::Prime(Box::new(result), primes);
        }

        match (sub, sup) {
            (None, None) => Ok(result),
            (Some(s), None) => Ok(Expr::Subscript(Box::new(result), s)),
            (None, Some(s)) => Ok(Expr::Superscript(Box::new(result), s)),
            (Some(s), Some(p)) => Ok(Expr::BothScripts(Box::new(result), s, p)),
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let next = self.advance().clone();
        match next {
            Token::Number(s) => {
                let s = s.to_string();
                Ok(Expr::Number(s))
            }
            Token::Ident(s) => {
                let s = s.to_string();
                Ok(Expr::Ident(s))
            }
            Token::LBrace => {
                let inner = self.parse_expr()?;
                self.expect(Token::RBrace)?;
                Ok(inner)
            }
            Token::LParen => Ok(Expr::Ident("(".into())),
            Token::RParen => Ok(Expr::Ident(")".into())),
            Token::LBracket => Ok(Expr::Ident("[".into())),
            Token::RBracket => Ok(Expr::Ident("]".into())),
            Token::Command(name) => {
                let name = name.to_string();
                if name == "begin" {
                    self.parse_begin()
                } else if name == "left" {
                    self.parse_left_delimited()
                } else if name == "right" {
                    Err(ParseError::UnexpectedRightWithoutLeft)
                } else {
                    self.parse_command(&name)
                }
            }
            Token::Escape(s) => {
                let s = s.to_string();
                Ok(Expr::Escape(s))
            }
            Token::Pipe => {
                let inner = self.parse_expr()?;
                self.expect(Token::Pipe)?;
                Ok(Expr::Command {
                    name: "|".into(),
                    opts: vec![],
                    args: vec![inner],
                })
            }
            Token::Minus => {
                // Unary minus if an operand follows (`-x`), otherwise a bare
                // symbol (`\pm`-style groups, `x^{-}`).
                if self.can_start_atom() {
                    Ok(Expr::Neg(Box::new(self.parse_atom()?)))
                } else {
                    Ok(Expr::Ident("-".into()))
                }
            }
            Token::Bang => Ok(Expr::Ident("!".into())),
            Token::Ampersand => Ok(Expr::Ident("&".into())),
            Token::Plus => Ok(Expr::Ident("+".into())),
            // Punctuation and relations that carry no special layout: render the
            // literal symbol. Without these the tokens are dropped or collide
            // with a closing-delimiter expectation (e.g. `(3,0)`, `x^{a/b}`).
            Token::Slash => Ok(Expr::Ident("/".into())),
            Token::Comma => Ok(Expr::Ident(",".into())),
            Token::Dot => Ok(Expr::Ident(".".into())),
            Token::Colon => Ok(Expr::Ident(":".into())),
            Token::Semicolon => Ok(Expr::Ident(";".into())),
            Token::Less => Ok(Expr::Ident("<".into())),
            Token::Greater => Ok(Expr::Ident(">".into())),
            Token::Question => Ok(Expr::Ident("?".into())),
            Token::At => Ok(Expr::Ident("@".into())),
            Token::Hash => Ok(Expr::Ident("#".into())),
            Token::Dollar => Ok(Expr::Ident("$".into())),
            Token::Percent => Ok(Expr::Ident("%".into())),
            Token::Tilde => Ok(Expr::Ident("~".into())),
            Token::Quote => Ok(Expr::Ident("\"".into())),
            Token::Backtick => Ok(Expr::Ident("`".into())),
            other => Err(ParseError::UnexpectedToken {
                position: self.pos - 1,
                token: format!("{other:?}"),
            }),
        }
    }

    fn parse_left_delimited(&mut self) -> Result<Expr, ParseError> {
        // Peek left and convert it back to it's character representation
        let left = self
            .peek()
            .and_then(|tok| tok.as_char())
            .ok_or(ParseError::ExpectedDelimiter { side: "left" })?;

        let inner_start = self.pos;
        let mut depth = 0usize;
        let mut match_idx = None;

        for (idx, (token, _)) in self.tokens[inner_start..].iter().enumerate() {
            match token {
                Token::Command(name) if *name == "left" => depth += 1,
                Token::Command(name) if *name == "right" => {
                    if depth == 0 {
                        match_idx = Some(inner_start + idx);
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }

        let Some(match_idx) = match_idx else {
            return Err(ParseError::UnclosedLeftRight);
        };

        let Some((Token::Command(name), _)) = self.tokens.get(match_idx) else {
            return Err(ParseError::Internal("missing \\right command".into()));
        };
        if *name != "right" {
            return Err(ParseError::Internal("mismatched delimiter scan".into()));
        }

        // Convert the token after the matched index back into it's character representation
        let right = self
            .tokens
            .get(match_idx + 1)
            .and_then(|(t, _)| t.as_char())
            .ok_or(ParseError::ExpectedDelimiter { side: "right" })?;

        // Expect left and right delimiters to match
        // TODO: this should probably get it's own strongly-typed error variant. it's not really
        // unreachable code, and this branch could reasonably be called in production, resulting in
        // a hard panic.
        let expected_right = match left {
            '(' => ')',
            '[' => ']',
            '{' => '}',
            '|' => '|',
            _ => unreachable!("unsupported left delimiter"),
        };
        if right != expected_right {
            return Err(ParseError::MismatchedDelimiters { left, right });
        }

        let inner = self.parse_tokens(&self.tokens[inner_start..match_idx])?;
        self.pos = match_idx + 2;
        Ok(Expr::Delimiter {
            left,
            right,
            inner: Box::new(inner),
        })
    }

    fn parse_delimited_arg(&mut self, close: Token) -> Result<Expr, ParseError> {
        self.advance();
        let inner_start = self.pos;
        let mut depth = 1usize;
        let mut match_idx = None;
        for (idx, (token, _)) in self.tokens[inner_start..].iter().enumerate() {
            if Self::is_opener_for(token, &close) {
                depth += 1;
            } else if Self::token_eq(token, &close) {
                if depth == 1 {
                    match_idx = Some(inner_start + idx);
                    break;
                }
                depth -= 1;
            }
        }
        match match_idx {
            Some(idx) => {
                let arg_tokens = &self.tokens[inner_start..idx];
                self.pos = idx + 1;
                self.parse_tokens(arg_tokens)
            }
            None => Err(ParseError::UnclosedLeftRight),
        }
    }

    fn is_opener_for(token: &Token, close: &Token) -> bool {
        matches!(
            (token, close),
            (Token::LBrace, Token::RBrace)
                | (Token::LParen, Token::RParen)
                | (Token::LBracket, Token::RBracket)
        )
    }

    fn token_eq(a: &Token, b: &Token) -> bool {
        matches!(
            (a, b),
            (Token::RBrace, Token::RBrace)
                | (Token::RParen, Token::RParen)
                | (Token::RBracket, Token::RBracket)
        )
    }

    fn parse_command(&mut self, name: &str) -> Result<Expr, ParseError> {
        let glyph = self.registry.get(name);
        let has_opt = glyph.is_some_and(|g| g.has_optional());
        let mut n_req = glyph.map_or(0, |g| g.required_args());
        let has_limits = glyph.is_some_and(|g| g.has_limits());
        let takes_str = glyph.is_some_and(|g| g.takes_string_arg());
        let mut opts = Vec::new();
        let mut args = Vec::new();

        if has_opt && self.peek() == Some(&Token::LBracket) {
            let opt = self.parse_delimited_arg(Token::RBracket)?;
            opts.push(opt);
        }

        if !has_limits {
            if takes_str {
                let text = self.parse_string_arg()?;
                args.push(Expr::Ident(text));
                n_req -= 1;
            }

            for _ in 0..n_req {
                let arg = match self.peek() {
                    Some(Token::LBrace) => self.parse_delimited_arg(Token::RBrace)?,
                    Some(Token::LParen) => self.parse_delimited_arg(Token::RParen)?,
                    Some(Token::LBracket) => self.parse_delimited_arg(Token::RBracket)?,
                    _ => self.parse_atom()?,
                };
                args.push(arg);
            }
        }

        Ok(Expr::Command {
            name: name.to_string(),
            opts,
            args,
        })
    }

    fn parse_begin(&mut self) -> Result<Expr, ParseError> {
        self.expect(Token::LBrace)?;

        let env_name = self.parse_continuous_string(Token::RBrace)?;
        self.expect(Token::RBrace)?;

        if !matches!(env_name.as_str(), "matrix" | "bmatrix" | "pmatrix") {
            return Err(ParseError::UnknownEnvironment { name: env_name });
        }

        let body_start = self.pos;
        let mut depth = 0u32;
        let end_pos = loop {
            match self.tokens.get(self.pos) {
                None => return Err(ParseError::UnclosedEnvironment { name: env_name }),
                Some((Token::Command(name), _)) if *name == "begin" => {
                    depth += 1;
                    self.pos += 1;
                }
                Some((Token::Command(name), _)) if *name == "end" => {
                    if depth == 0 {
                        break self.pos;
                    }
                    depth -= 1;
                    self.pos += 1;
                }
                Some(_) => {
                    self.pos += 1;
                }
            }
        };

        let body = &self.tokens[body_start..end_pos];
        let rows = self.parse_matrix_body(body)?;

        self.advance();
        self.expect(Token::LBrace)?;

        let end_name = self.parse_continuous_string(Token::RBrace)?;
        if *end_name != env_name {
            return Err(ParseError::MismatchedEnvironment {
                begin: env_name,
                end: end_name,
            });
        }

        self.expect(Token::RBrace)?;

        Ok(Expr::Matrix {
            name: env_name,
            rows,
        })
    }

    fn parse_matrix_body(&self, tokens: &'a [SpannedToken]) -> Result<Vec<Vec<Expr>>, ParseError> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut current_row: Vec<Expr> = Vec::new();
        let mut cell_start: usize = 0;
        let mut depth: u32 = 0;
        let mut env_depth: u32 = 0;

        for (i, (token, _)) in tokens.iter().enumerate() {
            match token {
                Token::LBrace | Token::LBracket | Token::LParen => depth += 1,
                Token::RBrace | Token::RBracket | Token::RParen => depth = depth.saturating_sub(1),
                Token::Command(name) if *name == "begin" => env_depth += 1,
                Token::Command(name) if *name == "end" => env_depth = env_depth.saturating_sub(1),
                Token::Ampersand if depth == 0 && env_depth == 0 => {
                    let cell = self.parse_tokens(&tokens[cell_start..i])?;
                    current_row.push(cell);
                    cell_start = i + 1;
                }
                Token::Escape(s) if *s == "\\" && depth == 0 && env_depth == 0 => {
                    let cell = self.parse_tokens(&tokens[cell_start..i])?;
                    current_row.push(cell);
                    rows.push(current_row);
                    current_row = Vec::new();
                    cell_start = i + 1;
                }
                _ => {}
            }
        }

        if cell_start < tokens.len() {
            let cell = self.parse_tokens(&tokens[cell_start..])?;
            current_row.push(cell);
        }
        if !current_row.is_empty() || rows.is_empty() {
            rows.push(current_row);
        }

        Ok(rows)
    }

    fn parse_tokens(&self, tokens: &'a [SpannedToken]) -> Result<Expr, ParseError> {
        if tokens.is_empty() {
            return Ok(Expr::Empty);
        }

        let mut sub = Parser {
            input: self.input,
            tokens,
            pos: 0,
            registry: self.registry,
        };

        sub.parse_expr()
    }
}
