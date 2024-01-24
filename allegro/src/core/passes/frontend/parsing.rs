use crate::core::resource::{
    ast::*,
    tokens::{Token, TokenKind, TokenKind::*},
};

pub struct Parser {
    pub tkvec: Vec<Token>,
    pub curr: usize,
    pub ast: Vec<Statement>,
}

impl Parser {
    pub fn new(tkvec: Vec<Token>) -> Self {
        Parser {
            tkvec,
            curr: 0,
            ast: vec![],
        }
    }

    fn peek(&mut self) -> Token {
        if self.curr < self.tkvec.len() {
            self.tkvec.get(self.curr).unwrap().clone()
        } else {
            Token {
                kind: TEof,
                literal: SymbolKind::Nothing,
                location: self.curr,
            }
        }
    }

    fn previous(&mut self) -> Token {
        self.tkvec.get(self.curr - 1).unwrap().clone()
    }

    fn is_at_end(&mut self) -> bool {
        self.peek().kind == TokenKind::TEof
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.curr += 1;
        }
        return self.previous();
    }

    fn check(&mut self, kind: TokenKind) -> bool {
        if self.is_at_end() {
            return false;
        };
        return self.peek().kind == kind;
    }

    fn search(&mut self, kinds: Vec<TokenKind>) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        return false;
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Token {
        if self.check(kind) {
            return self.advance();
        } else {
            println!("{message}");
            panic!();
        }
    }

    fn consume_vec(&mut self, kinds: Vec<TokenKind>, message: &str) -> Token {
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
        let tk = self.peek();
        if self.search(vec![TkKwFalse]) {
            return Expr::Literal(LiteralExpr { value: tk });
        } else if self.search(vec![TkKwTrue]) {
            return Expr::Literal(LiteralExpr { value: tk });
        } else if self.search(vec![TkLiteral, TkNumeric]) {
            return Expr::Literal(LiteralExpr { value: tk });
        } else if self.search(vec![TkSymbol]) {
            return Expr::Value(ValueExpr {
                name: self.previous(),
            });
        } else if self.search(vec![TkLparen]) {
            let expr: Expr = self.expression();
            self.consume(TkRparen, "Expected ')' after expression");
            return Expr::Grouping(GroupExpr {
                expression: Box::new(expr),
            });
        } else {
            Expr::Empty
        }
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

        return self.primary();
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
        self.consume(TkStatementEnd, "Unexpected end of print statement!");
        return Statement::Print(PrintStmt { expression: expr });
    }

    fn expr_stmt(&mut self) -> Statement {
        let expr: Expr = self.expression();
        self.consume(TkStatementEnd, "Unexpected end of expression!");
        return Statement::Expression(ExpressionStmt { expression: expr });
    }

    fn statement(&mut self) -> Statement {
        if self.search(vec![TkKwPrint]) {
            return self.print_stmt();
        } else if self.search(vec![TkKWVal]) {
            return self.val_decl();
        } else if self.search(vec![TkKwIf]) {
            return self.val_decl();
        }else {
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
        
        return Statement::Val(res)
    }

    fn init_param(&mut self) -> (Token, SymbolKind) {
        let name: Token = self.consume(TkSymbol, "Expected value name");

        let kind: SymbolKind = self
            .consume_vec(vec![TkTyFlt, TkTyInt, TkTyStr, TkTyMute, TkTyBool], "Invalid type")
            .literal;

        (name, kind)
    }

    fn val_signiture(&mut self) -> (Token, SymbolKind) {
        let name: Token = self.consume(TkSymbol, "Expected value name");

        let kind: SymbolKind = self
            .consume_vec(vec![TkTyFlt, TkTyInt, TkTyStr, TkTyMute, TkTyBool], "Invalid type")
            .literal;

        (name, kind)
    }

    

    fn op_decl(&mut self) -> Statement {
        let name: Token = self.consume(TkSymbol, "Expected operation name");
        let mut params: Vec<ValDecl> = vec![];
        let opreturnkind: SymbolKind;
        self.consume(TkEqual, "Expected '='");
        self.consume(TkLparen, "Expected '('");

        if self.tkvec[self.curr].kind != TkRparen {
            while self.tkvec[self.curr].kind == TkSymbol
                || self.tkvec[self.curr].kind == TkTyStr
                || self.tkvec[self.curr].kind == TkTyInt
                || self.tkvec[self.curr].kind == TkTyFlt
            {
                let nv = self.init_param();
                params.push(ValDecl {
                    name: nv.0,
                    kind: nv.1,
                    initializer: Expr::Empty,
                });
                //self.curr += 2;
            }
        } else {
            self.consume(TkRparen, "Expected ')'");
        }
        //println!("{:?}", self.tkvec[self.curr]);

        self.consume(TkSmallArr, "Expected '->'");
        opreturnkind = self.advance().literal;
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
        while self.tkvec[self.curr].kind != TkRBrace && self.tkvec[self.curr + 1].kind != TEof {
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
        let mut new_statements: Vec<Statement> = vec![];
        for (i, el) in statements.clone().iter().enumerate() {
            match el {
                Statement::Expression(es) => {
                    match es.expression {
                        Expr::Empty => {}
                        _ => {
                            new_statements.push(el.to_owned());
                        }
                    }
                }
                _ => {new_statements.push(el.to_owned())}
            }
        }
        self.ast = new_statements;
    }

    pub fn supply(&mut self) -> Vec<Statement> {
        return self.ast.clone();
    }
}
