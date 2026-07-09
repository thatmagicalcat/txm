use crate::ast::*;
use crate::glyph::SymbolRegistry;
use crate::token::Token;

#[derive(Debug)]
#[allow(dead_code)]
pub struct ParseError(pub String);

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

    fn expect(&mut self, tok: Token) {
        let actual = self.advance();
        assert_eq!(
            actual, tok,
            "Parse error: expected {:?}, got {:?}",
            tok, actual
        );
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
        | Some(Token::Bang) = self.peek()
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

        let mut result = base;

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
                self.expect(Token::RBrace);
                Ok(Expr::Group(Box::new(inner)))
            }
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::RParen);
                Ok(Expr::Parens(Box::new(inner)))
            }
            Some(Token::LBracket) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::RBracket);
                Ok(Expr::Brackets(Box::new(inner)))
            }
            Some(Token::Command(name)) => {
                let name = name.clone();
                self.advance();
                self.parse_command(&name)
            }
            Some(Token::Escape(s)) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Escape(s))
            }
            Some(Token::Pipe) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect(Token::Pipe);
                Ok(Expr::Command {
                    name: "|".into(),
                    args: vec![inner],
                })
            }
            Some(Token::Minus) => {
                self.advance();
                let inner = self.parse_atom()?;
                Ok(Expr::Neg(Box::new(inner)))
            }
            Some(Token::Bang) => {
                self.advance();
                Ok(Expr::Ident("!".into()))
            }
            Some(Token::Plus) => {
                self.advance();
                Ok(self.parse_atom()?)
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
        let mut args = Vec::new();

        if has_opt && self.peek() == Some(&Token::LBracket) {
            self.advance();
            let opt = self.parse_expr()?;
            self.expect(Token::RBracket);
            args.push(opt);
        }

        for _ in 0..n_req {
            self.expect(Token::LBrace);
            let arg = self.parse_expr()?;
            self.expect(Token::RBrace);
            args.push(arg);
        }

        Ok(Expr::Command {
            name: name.to_string(),
            args,
        })
    }
}
