//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).

#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::{LexResult, Token, TokenKind};
use crate::span::{FileId, Span};

#[derive(Debug)]
pub struct ParseResult {
    pub module: Option<Module>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn parse_tokens(file_id: FileId, lex: LexResult) -> ParseResult {
    let mut parser = Parser {
        file_id,
        tokens: lex.tokens,
        pos: 0,
        diagnostics: lex.diagnostics,
        directives: Vec::new(),
        indent_width: lex.indent_width,
    };

    let module = parser.parse_module();
    ParseResult {
        module,
        diagnostics: parser.diagnostics,
    }
}

struct Parser {
    file_id: FileId,
    tokens: Vec<Token>,
    pos: usize,
    diagnostics: Vec<Diagnostic>,
    directives: Vec<Directive>,
    indent_width: usize,
}

impl Parser {
    fn parse_module(&mut self) -> Option<Module> {
        let root = self.parse_block_until(TokenEnd::Eof)?;
        Some(Module {
            indent_width: self.indent_width,
            directives: self.directives.clone(),
            root,
        })
    }

    fn parse_block_until(&mut self, end: TokenEnd) -> Option<Block> {
        let mut items = Vec::new();
        let mut start_span = self.peek_span().unwrap_or_else(Span::dummy);

        while !self.is_end(&end) {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            if matches!(self.peek_kind(), Some(TokenKind::Dedent)) && matches!(end, TokenEnd::Dedent) {
                break;
            }

            let stmt = self.parse_stmt()?;
            if let Stmt::Directive(dir) = &stmt {
                self.directives.push(dir.clone());
            }
            items.push(stmt);
        }

        let end_span = if let Some(last) = items.last() {
            self.stmt_span(last)
        } else {
            start_span
        };
        Some(Block {
            items,
            span: start_span.join(end_span).unwrap_or(start_span),
        })
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.peek_kind()? {
            TokenKind::DirEntry(_) => {
                let (name, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirEntry(n) = tok.kind.clone() {
                            (n, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(Directive::Entry {
                    name: Ident { name, span },
                }))
            }
            TokenKind::DirImport(_) => {
                let (path, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirImport(p) = tok.kind.clone() {
                            (p, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(Directive::Import { path, span }))
            }
            TokenKind::DirUse(_) => {
                let (path, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirUse(p) = tok.kind.clone() {
                            (p, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(Directive::Use { path, span }))
            }
            TokenKind::DirIfTarget(_) => {
                let (target, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirIfTarget(t) = tok.kind.clone() {
                            (t, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(Directive::IfTarget { target, span }))
            }
            TokenKind::DirIndentWidth(width) => {
                let span = self.next().unwrap().span;
                Some(Stmt::Directive(Directive::IndentWidth { width, span }))
            }
            TokenKind::DirWasm => {
                let span = self.next().unwrap().span;
                let wasm = self.parse_wasm_block(span)?;
                Some(Stmt::Wasm(wasm))
            }
            TokenKind::KwFn => self.parse_fn(),
            _ => {
                let expr = self.parse_prefix_expr()?;
                Some(Stmt::Expr(expr))
            }
        }
    }

    fn parse_fn(&mut self) -> Option<Stmt> {
        let fn_span = self.next()?.span;
        let name_tok = self.expect_ident()?;
        let name = Ident {
            name: name_tok.0,
            span: name_tok.1,
        };

        self.expect(TokenKind::LAngle)?;
        let signature = self.parse_type_expr()?;
        self.expect(TokenKind::RAngle)?;

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                let (pname, pspan) = self.expect_ident()?;
                params.push(Ident {
                    name: pname,
                    span: pspan,
                });
                if self.consume_if(TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::Colon)?;
        let body = self.parse_block_after_colon()?;

        let fn_body = match body.items.first() {
            Some(Stmt::Wasm(wb)) if body.items.len() == 1 => FnBody::Wasm(wb.clone()),
            _ => FnBody::Parsed(body),
        };

        Some(Stmt::FnDef(FnDef {
            name,
            signature,
            params,
            body: fn_body,
        }))
    }

    fn parse_wasm_block(&mut self, dir_span: Span) -> Option<WasmBlock> {
        self.consume_if(TokenKind::Colon);
        if self.consume_if(TokenKind::Newline) {
            // ok
        }
        self.expect(TokenKind::Indent)?;

        let mut lines = Vec::new();
        let mut start_span = dir_span;
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            match self.peek_kind() {
                Some(TokenKind::WasmText(text)) => {
                    let tok = self.next().unwrap();
                    lines.push(text.clone());
                    start_span = start_span.join(tok.span).unwrap_or(tok.span);
                    self.consume_if(TokenKind::Newline);
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics.push(Diagnostic::error(
                        "expected wasm text line",
                        span,
                    ));
                    self.next();
                }
            }
        }

        self.expect(TokenKind::Dedent)?;
        let end_span = self.peek_span().unwrap_or_else(Span::dummy);
        Some(WasmBlock {
            lines,
            span: dir_span.join(end_span).unwrap_or(dir_span),
        })
    }

    fn parse_block_after_colon(&mut self) -> Option<Block> {
        if !self.consume_if(TokenKind::Newline) {
            let span = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics.push(Diagnostic::error(
                "expected newline after ':'",
                span,
            ));
        }
        self.expect(TokenKind::Indent)?;
        let block = self.parse_block_until(TokenEnd::Dedent)?;
        self.expect(TokenKind::Dedent)?;
        Some(block)
    }

    fn parse_prefix_expr(&mut self) -> Option<PrefixExpr> {
        let start_span = self.peek_span().unwrap_or_else(Span::dummy);
        let mut items = Vec::new();

        while !self.is_end(&TokenEnd::Line) {
            match self.peek_kind()? {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Colon => {
                    let colon_span = self.next().unwrap().span;
                    let block = self.parse_block_after_colon()?;
                    let span = colon_span.join(block.span).unwrap_or(colon_span);
                    items.push(PrefixItem::Block(block, span));
                    break;
                }
                TokenKind::Semicolon => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Semi(span));
                }
                TokenKind::LAngle => {
                    let start = self.next().unwrap().span;
                    let ty = self.parse_type_expr()?;
                    self.expect(TokenKind::RAngle)?;
                    let end = self.peek_span().unwrap_or(start);
                    let span = start.join(end).unwrap_or(start);
                    items.push(PrefixItem::TypeAnnotation(ty, span));
                }
                TokenKind::IntLiteral(_) | TokenKind::FloatLiteral(_) | TokenKind::BoolLiteral(_) | TokenKind::UnitLiteral => {
                    let tok = self.next().unwrap();
                    let lit = match tok.kind {
                        TokenKind::IntLiteral(v) => Literal::Int(v),
                        TokenKind::FloatLiteral(v) => Literal::Float(v),
                        TokenKind::BoolLiteral(b) => Literal::Bool(b),
                        TokenKind::UnitLiteral => Literal::Unit,
                        _ => unreachable!(),
                    };
                    items.push(PrefixItem::Literal(lit, tok.span));
                }
                TokenKind::KwLet => {
                    let _ = self.next();
                    let is_mut = self.consume_if(TokenKind::KwMut);
                    let (name, span) = self.expect_ident()?;
                    items.push(PrefixItem::Symbol(Symbol::Let {
                        name: Ident { name, span },
                        mutable: is_mut,
                    }));
                }
                TokenKind::KwSet => {
                    let set_span = self.next().unwrap().span;
                    let (name, span) = self.expect_ident()?;
                    items.push(PrefixItem::Symbol(Symbol::Set {
                        name: Ident { name, span },
                    }));
                    let _ = set_span; // span kept in symbol name
                }
                TokenKind::KwIf => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::If(span)));
                }
                TokenKind::KwWhile => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::While(span)));
                }
                TokenKind::Ident(name) => {
                    let tok = self.next().unwrap();
                    items.push(PrefixItem::Symbol(Symbol::Ident(Ident {
                        name: name.clone(),
                        span: tok.span,
                    })));
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics
                        .push(Diagnostic::error("unexpected token in expression", span));
                    self.next();
                }
            }
        }

        let end_span = items
            .last()
            .map(|i| self.item_span(i))
            .unwrap_or(start_span);
        Some(PrefixExpr {
            items,
            span: start_span.join(end_span).unwrap_or(start_span),
        })
    }

    fn parse_type_expr(&mut self) -> Option<TypeExpr> {
        match self.peek_kind()? {
            TokenKind::UnitLiteral => {
                self.next();
                Some(TypeExpr::Unit)
            }
            TokenKind::Ident(name) => {
                let tok = self.next().unwrap();
                match name.as_str() {
                    "i32" => Some(TypeExpr::I32),
                    "f32" => Some(TypeExpr::F32),
                    "bool" => Some(TypeExpr::Bool),
                    _ => Some(TypeExpr::Label(Some(name.clone()))),
                }
            }
            TokenKind::Dot => {
                let dot_span = self.next().unwrap().span;
                if let Some(TokenKind::Ident(name)) = self.peek_kind() {
                    let tok = self.next().unwrap();
                    Some(TypeExpr::Label(Some(name.clone())))
                } else {
                    Some(TypeExpr::Label(None))
                }
            }
            TokenKind::LParen => {
                self.next();
                let mut params = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        let ty = self.parse_type_expr()?;
                        params.push(ty);
                        if self.consume_if(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                let effect = match self.peek_kind() {
                    Some(TokenKind::Arrow(eff)) => {
                        let eff_copy = eff;
                        self.next();
                        eff_copy
                    }
                    _ => {
                        let span = self.peek_span().unwrap_or_else(Span::dummy);
                        self.diagnostics.push(Diagnostic::error(
                            "expected -> or *> after parameter list",
                            span,
                        ));
                        Effect::Pure
                    }
                };
                let result = self.parse_type_expr()?;
                Some(TypeExpr::Function {
                    params,
                    result: Box::new(result),
                    effect,
                })
            }
            _ => {
                let span = self.peek_span().unwrap_or_else(Span::dummy);
                self.diagnostics
                    .push(Diagnostic::error("invalid type expression", span));
                self.next();
                None
            }
        }
    }

    // ------------------------------------------------------------------
    // helpers
    // ------------------------------------------------------------------

    fn expect(&mut self, kind: TokenKind) -> Option<()> {
        if self.consume_if(kind.clone()) {
            Some(())
        } else {
            let span = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics.push(Diagnostic::error(
                alloc::format!("expected {:?}", kind),
                span,
            ));
            None
        }
    }

    fn expect_ident(&mut self) -> Option<(String, Span)> {
        match self.peek_kind()? {
            TokenKind::Ident(name) => {
                let tok = self.next().unwrap();
                if let TokenKind::Ident(n) = tok.kind {
                    Some((n, tok.span))
                } else {
                    None
                }
            }
            _ => {
                let span = self.peek_span().unwrap_or_else(Span::dummy);
                self.diagnostics
                    .push(Diagnostic::error("expected identifier", span));
                None
            }
        }
    }

    fn consume_if(&mut self, kind: TokenKind) -> bool {
        if self.check(kind.clone()) {
            self.next();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        matches!(self.peek_kind(), Some(k) if token_kind_eq(&k, &kind))
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.pos).map(|t| t.kind.clone())
    }

    fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.pos).map(|t| t.span)
    }

    fn next(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(t)
        } else {
            None
        }
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek_kind(), Some(TokenKind::Eof))
    }

    fn is_end(&self, end: &TokenEnd) -> bool {
        match end {
            TokenEnd::Eof => self.is_eof(),
            TokenEnd::Dedent => matches!(self.peek_kind(), Some(TokenKind::Dedent)|Some(TokenKind::Eof)),
            TokenEnd::Line => matches!(self.peek_kind(), Some(TokenKind::Newline)|Some(TokenKind::Dedent)|Some(TokenKind::Eof)),
        }
    }

    fn item_span(&self, item: &PrefixItem) -> Span {
        match item {
            PrefixItem::Literal(_, sp) => *sp,
            PrefixItem::Symbol(Symbol::Ident(id)) => id.span,
            PrefixItem::Symbol(Symbol::Let { name, .. }) => name.span,
            PrefixItem::Symbol(Symbol::Set { name }) => name.span,
            PrefixItem::Symbol(Symbol::If(sp)) => *sp,
            PrefixItem::Symbol(Symbol::While(sp)) => *sp,
            PrefixItem::TypeAnnotation(_, sp) => *sp,
            PrefixItem::Block(_, sp) => *sp,
            PrefixItem::Semi(sp) => *sp,
        }
    }

    fn stmt_span(&self, stmt: &Stmt) -> Span {
        match stmt {
            Stmt::Directive(d) => match d {
                Directive::Entry { name } => name.span,
                Directive::Import { span, .. } => *span,
                Directive::Use { span, .. } => *span,
                Directive::IfTarget { span, .. } => *span,
                Directive::IndentWidth { span, .. } => *span,
            },
            Stmt::FnDef(f) => f.name.span,
            Stmt::Wasm(w) => w.span,
            Stmt::Expr(e) => e.span,
        }
    }
}

#[derive(Clone, Copy)]
enum TokenEnd {
    Eof,
    Dedent,
    Line,
}

fn token_kind_eq(a: &TokenKind, b: &TokenKind) -> bool {
    use TokenKind::*;
    match (a, b) {
        (Arrow(e1), Arrow(e2)) => e1 == e2,
        (DirIndentWidth(x), DirIndentWidth(y)) => x == y,
        _ => core::mem::discriminant(a) == core::mem::discriminant(b),
    }
}
