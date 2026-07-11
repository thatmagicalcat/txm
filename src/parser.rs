use crate::ParseError;
use crate::ast::*;
use crate::glyph::SymbolRegistry;
use crate::token::Token;

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    registry: &'a SymbolRegistry,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], registry: &'a SymbolRegistry) -> Self {
        Self {
            tokens,
            pos: 0,
            registry,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn expect(&mut self, tok: Token) -> Result<(), ParseError> {
        match self.peek() {
            None => Err(ParseError(format!(
                "expected {:?} but reached end of input",
                tok
            ))),
            Some(actual) if *actual == tok => {
                self.advance();
                Ok(())
            }
            Some(actual) => Err(ParseError(format!("expected {:?}, got {:?}", tok, actual))),
        }
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
                | Some(Token::LBracket)
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
        | Some(Token::LBracket)
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
        | Some(Token::Greater) = self.peek()
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
                return Err(ParseError(
                    "internal parser error: limits argument base was not a command".into(),
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
        match self.peek() {
            Some(Token::Number(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Number(s))
            }
            Some(Token::Ident(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Ident(s))
            }
            Some(Token::LBrace) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::RBrace)?;
                Ok(Expr::Group(Box::new(inner)))
            }
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::Parens(Box::new(inner)))
            }
            Some(Token::LBracket) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::RBracket)?;
                Ok(Expr::Brackets(Box::new(inner)))
            }
            Some(Token::Command(name)) => {
                let name = name.clone();
                self.advance();
                if name == "begin" {
                    self.parse_begin()
                } else {
                    self.parse_command(&name)
                }
            }
            Some(Token::Escape(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Escape(s))
            }
            Some(Token::Pipe) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::Pipe)?;
                Ok(Expr::Command {
                    name: "|".into(),
                    opts: vec![],
                    args: vec![inner],
                })
            }
            Some(Token::Minus) => {
                self.advance();
                // Unary minus if an operand follows (`-x`), otherwise a bare
                // symbol (`\pm`-style groups, `x^{-}`).
                if self.can_start_atom() {
                    Ok(Expr::Neg(Box::new(self.parse_atom()?)))
                } else {
                    Ok(Expr::Ident("-".into()))
                }
            }
            Some(Token::Bang) => {
                self.advance();
                Ok(Expr::Ident("!".into()))
            }
            Some(Token::Ampersand) => {
                self.advance();
                Ok(Expr::Ident("&".into()))
            }
            Some(Token::Plus) => {
                self.advance();
                Ok(Expr::Ident("+".into()))
            }
            // Punctuation and relations that carry no special layout: render the
            // literal symbol. Without these the tokens are dropped or collide
            // with a closing-delimiter expectation (e.g. `(3,0)`, `x^{a/b}`).
            Some(Token::Slash) => {
                self.advance();
                Ok(Expr::Ident("/".into()))
            }
            Some(Token::Comma) => {
                self.advance();
                Ok(Expr::Ident(",".into()))
            }
            Some(Token::Dot) => {
                self.advance();
                Ok(Expr::Ident(".".into()))
            }
            Some(Token::Colon) => {
                self.advance();
                Ok(Expr::Ident(":".into()))
            }
            Some(Token::Semicolon) => {
                self.advance();
                Ok(Expr::Ident(";".into()))
            }
            Some(Token::Less) => {
                self.advance();
                Ok(Expr::Ident("<".into()))
            }
            Some(Token::Greater) => {
                self.advance();
                Ok(Expr::Ident(">".into()))
            }
            other => Err(ParseError(format!(
                "Unexpected token at position {}: {:?}",
                self.pos,
                other.cloned()
            ))),
        }
    }

    fn parse_command(&mut self, name: &str) -> Result<Expr, ParseError> {
        let glyph = self.registry.get(name);
        let has_opt = glyph.is_some_and(|g| g.has_optional());
        let n_req = glyph.map_or(0, |g| g.required_args());
        let has_limits = glyph.is_some_and(|g| g.has_limits());
        let mut opts = Vec::new();
        let mut args = Vec::new();

        if has_opt && self.peek() == Some(&Token::LBracket) {
            self.advance();
            let opt = self.parse_expr()?;
            self.expect(Token::RBracket)?;
            opts.push(opt);
        }

        if !has_limits {
            for _ in 0..n_req {
                // A macro argument is either a braced group `{...}` or, following
                // LaTeX, the single following atom (so `\mathbf x` and `\frac12`
                // work, not only `\mathbf{x}` and `\frac{1}{2}`).
                if self.peek() == Some(&Token::LBrace) {
                    self.advance();
                    let arg = self.parse_expr()?;
                    self.expect(Token::RBrace)?;
                    args.push(arg);
                } else {
                    args.push(self.parse_atom()?);
                }
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
        let env_name = match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                return Err(ParseError(
                    "expected environment name in \\begin{...}".into(),
                ));
            }
        };

        self.expect(Token::RBrace)?;

        if !matches!(env_name.as_str(), "matrix" | "bmatrix" | "pmatrix") {
            return Err(ParseError(format!(
                "unknown matrix environment: {env_name}"
            )));
        }

        let body_start = self.pos;
        let mut depth = 0u32;
        let end_pos = loop {
            match self.tokens.get(self.pos) {
                None => return Err(ParseError(format!("unclosed \\begin{{{}}}", env_name))),
                Some(Token::Command(name)) if name == "begin" => {
                    depth += 1;
                    self.pos += 1;
                }
                Some(Token::Command(name)) if name == "end" => {
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
        let end_name = match self.peek() {
            Some(Token::Ident(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => return Err(ParseError("expected environment name in \\end{...}".into())),
        };
        if end_name != env_name {
            return Err(ParseError(format!(
                "mismatched \\begin{{{}}} and \\end{{{}}}",
                env_name, end_name
            )));
        }
        self.expect(Token::RBrace)?;

        Ok(Expr::Matrix {
            name: env_name,
            rows,
        })
    }

    fn parse_matrix_body(&self, tokens: &'a [Token]) -> Result<Vec<Vec<Expr>>, ParseError> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        let mut current_row: Vec<Expr> = Vec::new();
        let mut cell_start: usize = 0;
        let mut depth: u32 = 0;
        let mut env_depth: u32 = 0;

        for (i, token) in tokens.iter().enumerate() {
            match token {
                Token::LBrace | Token::LBracket | Token::LParen => depth += 1,
                Token::RBrace | Token::RBracket | Token::RParen => depth = depth.saturating_sub(1),
                Token::Command(name) if name == "begin" => env_depth += 1,
                Token::Command(name) if name == "end" => env_depth = env_depth.saturating_sub(1),
                Token::Ampersand if depth == 0 && env_depth == 0 => {
                    let cell = self.parse_tokens(&tokens[cell_start..i])?;
                    current_row.push(cell);
                    cell_start = i + 1;
                }
                Token::Escape(s) if s == "\\" && depth == 0 && env_depth == 0 => {
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

    fn parse_tokens(&self, tokens: &'a [Token]) -> Result<Expr, ParseError> {
        if tokens.is_empty() {
            return Ok(Expr::Empty);
        }
        let mut sub = Parser {
            tokens,
            pos: 0,
            registry: self.registry,
        };
        sub.parse_expr()
    }
}
