use core::panic;

use crate::core::resource::{
    ast::*,
    environment::Environment,
    tokens::{Token, TokenType},
};

#[derive(Debug, Clone)]
pub struct GenLine {
    c: String,
}

pub struct Generator {
    ast: Vec<Statement>,
    env: Environment,
    loc: usize,
    output: Vec<GenLine>,
}

#[allow(missing_fragment_specifier)]
macro_rules! line {
    ($i:expr, $($arg:tt)*) => {
        GenLine {
            c: format!("{}", format_args!($($arg)*))
        }
    };
}

impl Generator {
    pub fn new(ast: Vec<Statement>, e: Environment) -> Self {
        Generator {
            ast,
            env: e,
            loc: 0,
            output: vec![],
        }
    }

    fn add(&mut self, st: GenLine) {
        self.output.push(st);
    }

    fn get_cval(&mut self, sk: SymbolValue) -> String {
        match sk {
            SymbolValue::Scalar(s) => match s {
                Scalar::Int(i) => i.to_string(),
                Scalar::Str(s) => format!("\"{}\"", s),
                Scalar::Float(f) => f.to_string(),
                Scalar::Bool(b) => {
                    if b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
            },
            SymbolValue::Identity(i) => {
                let name = format!("{}()", i.name);
                name
            }
            _ => {
                panic!("Unkown type {:?}!", sk)
            }
        }
    }

    fn get_ctype(&mut self, s: Statement) -> String {
        //println!("{s:?}");
        match s {
            Statement::Val(vd) => self.env.get_akind(vd.name.name).to_ctype(),
            Statement::Operation(o) => self
                .env
                .get_akind(o.name.value.unwrap().get_string().unwrap())
                .to_ctype(),
            _ => panic!("Unsupported statement {s:?}"),
        }
    }

    fn gen_cparams(&mut self, s: Statement) -> String {
        match s {
            Statement::Operation(o) => {
                let p = o.params;
                let mut accum: String = "".to_string();
                for (count, v) in p.iter().enumerate() {
                    let n = &v.name.name;
                    let t = v.name.kind.to_ctype();
                    if count == &p.len() - 1 {
                        accum = format!("{accum}{t} {n}");
                    } else {
                        accum = format!("{accum}{t} {n}, ");
                    }
                }
                accum
            }
            _ => panic!("{:?} is not an operation!", s),
        }
    }

    fn gen_operator(&mut self, o: Token) -> String {
        match o.tokentype {
            TokenType::TkPlus => "+".to_string(),
            TokenType::TkMinus => "-".to_string(),
            TokenType::TkStar => "*".to_string(),
            TokenType::TkSlash => "/".to_string(),
            TokenType::TkPercent => "%".to_string(),
            TokenType::TkCEQ => "==".to_string(),
            TokenType::TkCNE => "!=".to_string(),
            TokenType::TkCLT => "<".to_string(),
            TokenType::TkCLE => "<=".to_string(),
            TokenType::TkCGT => ">".to_string(),
            TokenType::TkCGE => ">=".to_string(),
            TokenType::TkAnd => "&&".to_string(),
            TokenType::TkOr => "||".to_string(),
            _ => panic!("{:?} is not an operator", o),
        }
    }

    pub fn generate(&mut self) {
        while self.loc < self.ast.len() {
            let el = self.ast[self.loc].clone();
            let res = self.gen_code(el);
            //println!("{res}");
            self.add(line!(0, "{res}"));
            self.loc += 1
        }
    }

    fn gen_code(&mut self, el: Statement) -> String {
        match el.clone() {
            Statement::Val(vd) => {
                let ckind = self.get_ctype(el);
                let cval: String = match vd.initializer {
                    Expr::ScalarEx(le) => self.get_cval(le.value.value.unwrap()),
                    Expr::Call(c) => self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: Expr::Call(c),
                    })),
                    Expr::Binary(b) => self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: Expr::Binary(b),
                    })),
                    Expr::Grouping(g) => self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: Expr::Grouping(g),
                    })),
                    _ => panic!("Unknown value {:?}!", vd.initializer),
                };
                let cname: String = vd.name.name;
                let x = format!("{} {} = {};", ckind, cname, cval);
                x
            }

            Statement::Operation(o) => {
                let cname: String = o.name.value.unwrap().get_string().unwrap();
                let ckind = self.get_ctype(el.clone());
                let cparams = self.gen_cparams(el);
                let b = self.gen_code(Statement::Block(o.body));
                let x = format!("{ckind} {cname}({cparams}) {b}");
                x
            }

            Statement::Block(b) => {
                let mut accum: String = "{".to_string();
                for s in b.statements {
                    if s != Statement::Empty
                        && s != Statement::Expression(ExpressionStmt {
                            expression: Expr::Empty,
                        })
                    {
                        accum = format!("{accum}\n    {}", self.gen_code(s))
                    }
                }
                accum = format!("{accum}\n}}\n");
                accum
            }

            Statement::Expression(e) => match e.expression {
                Expr::Assign(a) => {
                    let cvname = a.name.value.clone().unwrap().get_string().unwrap();
                    let cvtype = self
                        .env
                        .get_akind(a.name.value.unwrap().get_string().unwrap())
                        .to_ctype();
                    let cvval = self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: *a.value,
                    }));
                    format!("{cvtype} {cvname} = {cvval}")
                }
                Expr::Binary(b) => {
                    let cl = self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: *b.left,
                    }));
                    let cr = self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: *b.right,
                    }));
                    let o = self.gen_operator(b.operator);

                    format!("{cl} {o} {cr}")
                }
                Expr::Call(c) => {
                    let name = c.callee.value.unwrap().get_string().unwrap();
                    let mut argstring = "".to_string();
                    let mut argcount = 0;
                    for arg in c.args {
                        argcount += 1;
                        if argcount > 1 {
                            argstring = format!(
                                "{}, {}",
                                argstring,
                                self.gen_code(Statement::Expression(ExpressionStmt {
                                    expression: arg
                                }))
                            )
                        } else {
                            argstring = format!(
                                "{}{}",
                                argstring,
                                self.gen_code(Statement::Expression(ExpressionStmt {
                                    expression: arg
                                }))
                            )
                        }
                    }
                    format!("{}({})", name, argstring)
                    //todo!()
                }
                Expr::Grouping(g) => {
                    let x = format!(
                        "({})",
                        self.gen_code(Statement::Expression(ExpressionStmt {
                            expression: *g.expression
                        }))
                    );
                    x
                }
                Expr::ScalarEx(l) => l.value.value.unwrap().get_string().unwrap(),
                Expr::Logical(l) => {
                    let cl = self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: *l.left,
                    }));
                    let cr = self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: *l.right,
                    }));
                    let o = self.gen_operator(l.operator);

                    format!("{cl} {o} {cr}")
                }
                Expr::Unary(u) => {
                    format!(
                        "{}{}",
                        self.gen_operator(u.operator),
                        self.gen_code(Statement::Expression(ExpressionStmt {
                            expression: *u.right,
                        }))
                    )
                }
                Expr::Value(v) => {
                    v.name.value.unwrap().get_string().unwrap()
                }
                Expr::Empty => "".to_string(),
            },

            Statement::Return(r) => {
                let mut accum = "return".to_string();
                accum = format!(
                    "{accum} {};",
                    self.gen_code(Statement::Expression(ExpressionStmt {
                        expression: r.value
                    }))
                );
                accum
            }

            Statement::Print(p) => match p.expression.get_expr_value().tokentype {
                TokenType::TkKwTrue => todo!(),
                TokenType::TkKwFalse => todo!(),
                TokenType::TkSymbol => {
                    //println!("{:?}", p.expression.get_expr_value().value);
                    "//printf".to_string()
                    //return format!("printf({}", key,)
                }
                TokenType::TkScalar => match p.expression.get_expr_value().value.unwrap() {
                    SymbolValue::Scalar(s) => match s {
                        Scalar::Str(_st) => {
                            let accum = format!(
                                "printf(\"{}\");",
                                self.gen_code(Statement::Expression(ExpressionStmt {
                                    expression: p.expression
                                }))
                            );
                            accum
                        }
                        Scalar::Float(f) => {
                            let accum = format!("printf(\"%d\", {});", f);
                            accum
                        }
                        Scalar::Int(i) => {
                            let accum = format!("printf(\"%d\", {});", i);
                            accum
                        }
                        Scalar::Bool(b) => {
                            let accum = format!("printf({});", b);
                            accum
                        }
                    },
                    _ => panic!("{:?} cannot be printed!", p.expression),
                },
                _ => panic!("{:?} cannot be printed!", p.expression),
            },

            _ => {
                panic!("Unsupported ast Token: {:?}", el);
            }
        }
    }

    pub fn supply(&mut self) -> String {
        let mut accum: String =
            "//This file was automatically generated by allegro.\n//Submit bug reports to https://github.com/UPWRD1/allegro\n\n#include <stdio.h>\n\n".to_string();
        for i in self.output.clone() {
            accum = format!("{accum}{}", i.c)
        }
        accum = accum.to_string();
        accum
    }
}
