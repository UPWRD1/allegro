use core::panic;

use crate::core::resource::ast::*;

#[derive(Debug, Clone)]
pub struct GenLine {
    i: usize,
    c: String,
}

pub struct Generator {
    ast: Vec<Statement>,
    loc: usize,
    output: Vec<GenLine>,
}

#[allow(missing_fragment_specifier)]
macro_rules! line {
    ($i:expr, $($arg:tt)*) => {
        GenLine {
            i: $i,
            c: format!("{}", format_args!($($arg)*))
        }
    };
}

impl Generator {
    pub fn new(ast: Vec<Statement>) -> Self {
        return Generator {
            ast,
            loc: 0,
            output: vec![],
        };
    }

    fn add(&mut self, st: GenLine) {
        self.output.push(st);
    }

    fn get_cval(&mut self, sk: SymbolKind) -> String {
        match sk {
            SymbolKind::Int(i) => i.to_string(),
            SymbolKind::Str(s) => s.to_string(),
            SymbolKind::Float(f) => f.to_string(),
            _ => {
                panic!("Unkown type!")
            }
        }
    }

    fn get_ctype(&mut self, vd: ValDecl) -> String {
        println!("{vd:?}");
        match vd.kind {
            SymbolKind::TyStr => return "char*".to_string(),

            SymbolKind::TyInt => return "int".to_string(),

            _ => panic!("Unsupported type!"),
        }
    }

    pub fn generate(&mut self) {
        while self.loc < self.ast.len() {
            let el = self.ast[self.loc].clone();
            match el {
                Statement::Val(vd) => {
                    let cname: String;
                    let ckind = self.get_ctype(vd.clone());
                    let cval: String = self.get_cval(match vd.initializer {
                        Expr::Literal(mut le) => le.get_literal_value(),
                        _ => panic!("Unkown type!"),
                    });
                    match vd.name.literal {
                        SymbolKind::Identity(n, v) => cname = n,
                        _ => panic!("Unknown name!"),
                    }
                    self.add(line!(0, "{} {} = {}", ckind, cname, cval));
                }
                _ => {
                    panic!("Unsuportted ast Token {:?}", el);
                }
            };

            self.loc += 1
        }
    }

    pub fn supply(&mut self) -> String {
        let mut accum: String = "void main() {\n".to_string();
        for i in self.output.clone() {
            accum = format!("{accum}{};\n", i.c)
        }
        accum = format!("{accum}}}");
        return accum;
    }
}