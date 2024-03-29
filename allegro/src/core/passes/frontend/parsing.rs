use std::io::Empty;

use crate::core::resource::{
    ast::*,
    environment::AKind,
    tokens::{Token, TokenType, TokenType::*},
};

pub struct Parser {
    pub tkvec: Vec<Token>,
    pub curr: usize,
    pub ast: Vec<Statement>,
    pub new_stmts: Vec<Statement>,
}

impl Parser {
    pub fn new(tkvec: Vec<Token>) -> Self {
        Parser {
            tkvec,
            curr: 0,
            ast: vec![],
            new_stmts: vec![],
        }
    }

    fn peek(&mut self) -> Token {
        if self.curr < self.tkvec.len() {
            self.tkvec.get(self.curr).unwrap().clone()
        } else {
            Token {
                tokentype: TEof,
                value: SymbolValue::Nothing,
                kind: Some(AKind::TyEof),
                location: self.curr,
            }
        }
    }

    fn previous(&mut self) -> Token {
        self.tkvec.get(self.curr - 1).unwrap().clone()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().tokentype == TokenType::TEof
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr += 1;
        }
        return self.previous();
    }

    fn check(&mut self, kind: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        };
        return self.peek().tokentype == kind;
    }

    fn search(&mut self, kinds: Vec<TokenType>) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn consume(&mut self, kind: TokenType, message: &str) -> Token {
        if self.check(kind) {
            return self.advance();
        } else {
            println!("{message}");
            panic!();
        }
    }

    fn consume_vec(&mut self, kinds: Vec<TokenType>, message: &str) -> Token {
        for kind in kinds {
            if self.check(kind) {
                return self.advance();
            } else {
                continue;
            }
        }
        println!("{message}");
        panic!()
    }

    fn primary(&mut self) -> Expr {
        let mut tk = self.peek();
        if self.search(vec![TkSymbol]) {
            return Expr::Value(ValueExpr { name: tk });
        } else if self.search(vec![TkKwFalse]) {
            return Expr::Literal(LiteralExpr { value: tk });
        } else if self.search(vec![TkKwTrue]) {
            return Expr::Literal(LiteralExpr { value: tk });
        } else if self.search(vec![TkLiteral, TkNumeric]) {
            return Expr::Literal(LiteralExpr { value: tk });
            //value: Token
            //tokentype: tk.tokentype,
            //value: SymbolValue::Identity(
            //Ident { name: None, kind: Some(AKind::TyStr), value: SymbolValue::Str())
            //),
            //kind:
            //location: tk.location,
        } else if self.search(vec![TkLparen]) {
            let expr: Expr = self.expression();
            self.consume(TkRparen, "Expected ')' after expression");
            return Expr::Grouping(GroupExpr {
                expression: Box::new(expr),
            });
        } else {
            return Expr::Empty;
        }
    }

    fn secondary(&mut self) -> Expr {
        let expr = self.primary();
        loop {
            if self.search(vec![TkLparen]) {
                let paren: Token;
                let mut args: Vec<Expr> = vec![];
                if self.search(vec![TkRparen]) {
                    paren = self.previous();
                } else {
                    loop {
                        let argument = self.expression();
                        args.push(argument);
                        if !self.search(vec![TkComma]) {
                            break;
                        }
                    }
                    paren = self.consume(TkRparen, "expect ')' after arguments")
                }

                return Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    paren,
                    args,
                });
            } else {
                break;
            }
        }
        return expr;
    }

    fn unary_expr(&mut self) -> Expr {
        if self.search(vec![TkMinus]) {
            let operator: Token = self.previous();
            let right = self.unary_expr();
            return Expr::Unary(UnaryExpr {
                operator,
                right: Box::new(right),
            });
        }

        return self.secondary();
    }

    fn factor_expr(&mut self) -> Expr {
        let mut expr: Expr = self.unary_expr();
        while self.search(vec![TkStar, TkSlash]) {
            let operator: Token = self.previous();
            let right: Expr = self.unary_expr();
            expr = Expr::Binary(BinExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn term_expr(&mut self) -> Expr {
        let mut expr: Expr = self.factor_expr();
        while self.search(vec![TkMinus, TkPlus]) {
            let operator: Token = self.previous();
            let right: Expr = self.factor_expr();
            expr = Expr::Binary(BinExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn comparison_expr(&mut self) -> Expr {
        let mut expr = self.term_expr();
        while self.search(vec![TkCGT, TkCGE, TkCLT, TkCLE]) {
            let operator: Token = self.previous();
            let right: Expr = self.term_expr();
            expr = Expr::Binary(BinExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        return expr;
    }

    fn equality_expr(&mut self) -> Expr {
        let mut expr: Expr = self.comparison_expr();

        while self.search(vec![TkCNE, TkCEQ]) {
            let operator: Token = self.previous();
            let right: Expr = self.comparison_expr();
            expr = Expr::Binary(BinExpr {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            });
        }

        return expr;
    }

    fn expression(&mut self) -> Expr {
        return self.equality_expr();
    }

    fn print_stmt(&mut self) -> Statement {
        let expr: Expr = self.expression();
        //self.consume(TkStatementEnd, "Unexpected end of print statement!");
        return Statement::Print(PrintStmt { expression: expr });
    }

    fn expr_stmt(&mut self) -> Statement {
        let expr: Expr = self.expression();
        self.consume(TkStatementEnd, "Unexpected end of expression!");
        return Statement::Expression(ExpressionStmt { expression: expr });
    }

    fn return_stmt(&mut self) -> Statement {
        let value: Expr = self.expression();
        return Statement::Return(ReturnStmt { value });
    }

    fn statement(&mut self) -> Statement {
        if self.search(vec![TkKwPrint]) {
            return self.print_stmt();
        } else if self.search(vec![TkKwVal]) {
            return self.val_decl();
        } else if self.search(vec![TkKwReturn]) {
            return self.return_stmt();
        } else {
            return self.expr_stmt();
        }
    }

    fn val_decl(&mut self) -> Statement {
        let (name, kind) = self.val_signiture();
        self.consume(TkEqual, "Expected '='");

        let initializer: Expr = self.expression();

        if !self.search(vec![TkStatementEnd]) {
            panic!("Unexpected end of value declaration!")
        }
        let res = ValDecl {
            name,
            kind,
            initializer,
        };

        return Statement::Val(res);
    }

    fn init_param(&mut self) -> (Token, AKind) {
        let name: Token = self.advance();
        self.consume(TkColon, "Expected ':'");

        let newkind: AKind = self
            .consume_vec(
                vec![
                    TkType(AKind::TyInt),
                    TkType(AKind::TyFlt),
                    TkType(AKind::TyStr),
                    TkType(AKind::TyBool),
                    TkType(AKind::TyMute),
                    TkType(AKind::TyEof),
                ],
                "Expected type",
            )
            .value
            .to_akind();
        let new = Token {
            tokentype: name.tokentype,
            kind: Some(newkind.clone()),
            value: SymbolValue::Identity(Ident {
                name: Some(name.value.get_string().expect("Expected param name")),
                kind: Some(Box::new(newkind)),
                value: Box::new(SymbolValue::Unknown),
            }),
            location: name.location,
        };
        (new.clone(), new.kind.unwrap())
    }

    fn val_signiture(&mut self) -> (Token, AKind) {
        let name: Token = self.consume(TkSymbol, "Expected value name");
        self.consume(TkColon, "Expected ':' in vdecl");

        let kind = self
            .consume_vec(
                vec![
                    TkType(AKind::TyInt),
                    TkType(AKind::TyFlt),
                    TkType(AKind::TyStr),
                    TkType(AKind::TyBool),
                    TkType(AKind::TyEof),
                    TkType(AKind::TyMute),
                ],
                "Expected type",
            )
            .value.to_akind();
        let new = Token {
            tokentype: name.clone().tokentype,
            kind: Some(kind.clone()),
            value: SymbolValue::Identity(Ident {
                name: Some(
                    name.clone()
                        .value
                        .get_string()
                        .expect("Expected param name"),
                ),
                kind: Some(Box::new(kind.clone())),
                value: Box::new(SymbolValue::Unknown),
            }),
            location: name.location,
        };

        (name, kind)
    }

    fn op_decl(&mut self) -> Statement {
        let name: Token = self.consume(TkSymbol, "Expected operation name");
        self.consume(TkEqual, "Expected '='");
        self.consume(TkLparen, "Expected '('");

        let mut params: Vec<ValDecl> = vec![];
        if self.peek().tokentype != TkRparen {
            let nv = self.init_param();
            params.push(ValDecl {
                name: nv.0,
                kind: nv.1,
                initializer: Expr::Empty,
            });
        }

        //self.curr += 2;
        self.consume(TkRparen, "Expected ')'");
        self.consume(TkSmallArr, "Expected '->'");
        let opreturnkind = self.advance().value.to_akind();
        self.consume(TkKwIs, "Expected keyword 'is'");

        return Statement::Operation(OpDecl {
            name,
            params,
            kind: opreturnkind,
            body: BlockStmt {
                statements: self.opblock(),
            },
        });
    }

    fn opblock(&mut self) -> Vec<Statement> {
        let mut collector: Vec<Statement> = vec![];
        self.consume(TkLBrace, "Expected '{'");
        while self.tkvec[self.curr].tokentype != TkRBrace
            && self.tkvec[self.curr + 1].tokentype != TEof
        {
            collector.push(self.statement());
        }
        self.consume(TkRBrace, "Expected '}'");
        collector
    }

    fn declaration(&mut self) -> Statement {
        if self.search(vec![TkKWOp]) {
            return self.op_decl();
        }
        return self.statement();
    }

    pub fn parse(&mut self) {
        let mut statements: Vec<Statement> = vec![];
        while !self.is_at_end() {
            statements.push(self.declaration());
        }
        for (_i, el) in statements.clone().iter().enumerate() {
            match el {
                Statement::Expression(es) => match es.expression {
                    Expr::Empty => {}
                    _ => {
                        self.new_stmts.push(el.to_owned());
                    }
                },
                _ => self.new_stmts.push(el.to_owned()),
            }
        }
    }

    pub fn supply(&mut self) -> Vec<Statement> {
        return self.new_stmts.clone();
    }
}
