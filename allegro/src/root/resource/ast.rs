use std::fmt::Display;

use super::itypes::Itype;

#[derive(Debug, PartialEq, PartialOrd, Clone)]
#[allow(dead_code)]
pub enum BinOp {
    Plus, Minus,
    Mult, Div,
    Equal, NotEqual,
    And, Or,
    Less, Greater, LessEq, GreaterEq,
    Assign,
}

impl BinOp {
    pub fn to_char(&self) -> char {
        match self {
            BinOp::Plus => '+',
            BinOp::Minus => '-',
            BinOp::Mult => '*',
            BinOp::Div => '/',
            BinOp::Equal => '=',
            BinOp::Less => '<',
            BinOp::Greater => '>',

            _ => todo!(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UnaOp {
    Neg, Not
}

#[derive(Debug, Clone)]
pub enum VTypeKind {
    Int,
    Flt,
    Str,
    Bool,
    Custom(String),
    Generic(String),
    Unknown,
}

impl Display for VTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            VTypeKind::Int => "int",
            VTypeKind::Flt => "flt",
            VTypeKind::Str => "str",
            VTypeKind::Bool => "bool",
            VTypeKind::Custom(c) => &c,
            VTypeKind::Generic(g) => &g,
            VTypeKind::Unknown => "unknown",
        })
    }
}

#[derive(Debug, Clone)]
pub struct VType {
    pub kind: VTypeKind,
    is_mut: bool,
}

impl VType {
    pub fn new(kind: VTypeKind, is_mut: bool) -> Self {
        VType {
            kind,
            is_mut,
        }
    }
}



#[derive(Debug, Clone)]
pub struct Pair {
    pub name: String,
    pub value: VType,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Scalar(Itype),
    Array(Vec<Expr>),
    Variable(String),
    BinaryOp(BinOp, Box<(Expr, Expr)>),
    UnaryOp(UnaOp, Box<Expr>),
    Call {name: String, on: Option<String>, args: Vec<Expr>},
    //Assign(Variable),
    FnExpr(String, Vec<Pair>, Box<Expr>)
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Block(Vec<Stmt>),
    If(Expr, Box<(Stmt, Option<Stmt>)>),
    While(Expr, Box<Stmt>),
    ForEach(String, String, Box<Stmt>),
    ForRange(String, Itype, Itype, Box<Stmt>),
    Return(Expr),
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Pair,
    pub extends: Option<VType>,
    pub args: Vec<Pair>,
    pub code: Stmt,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub ini: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub funcs: Vec<Function>,
}

impl Function {
    pub fn new(name: String, args: Vec<Pair>, code: Vec<Stmt>, extends: Option<VType>) -> Function {
        Function {
            name: Pair { name, value: VType::new(VTypeKind::Unknown, false) }, extends, args, code: Stmt::Block(code)
        }
    }
}

impl Variable {
    pub fn new(name: String, ini: Expr) -> Variable {
        Variable {
            name, ini: Box::new(ini)
        }
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            funcs: Vec::new(),
        }
    }
    pub fn add_function(&mut self, f: Function) {
        self.funcs.push(f);
    }
}