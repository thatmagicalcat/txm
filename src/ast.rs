#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Number(String),
    Group(Box<Expr>),
    Parens(Box<Expr>),
    Brackets(Box<Expr>),
    Delimiter {
        left: char,
        right: char,
        inner: Box<Expr>,
    },
    Neg(Box<Expr>),
    Command {
        name: String,
        opts: Vec<Expr>,
        args: Vec<Expr>,
    },
    Superscript(Box<Expr>, Box<Expr>),
    Subscript(Box<Expr>, Box<Expr>),
    BothScripts(Box<Expr>, Box<Expr>, Box<Expr>),
    Prime(Box<Expr>, usize),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    Juxtapose(Vec<Expr>),
    Escape(String),
    Empty,
    Matrix {
        name: String,
        rows: Vec<Vec<Expr>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Eq,
    Mul,
}
