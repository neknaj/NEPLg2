//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).
//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::lexer::{LexResult, Token, TokenKind};
use crate::span::{FileId, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum IfRole {
    Cond,
    Then,
    Else,
}

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
            if matches!(self.peek_kind(), Some(TokenKind::Dedent))
                && matches!(end, TokenEnd::Dedent)
            {
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
            TokenKind::DirTarget(_) => {
                let (target, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirTarget(t) = tok.kind.clone() {
                            (t, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(Directive::Target { target, span }))
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
            TokenKind::DirExtern {
                module,
                name,
                func,
                signature,
            } => {
                let span = self.next().unwrap().span;
                let sig = parse_type_expr_str(&signature, span, &mut self.diagnostics)?;
                Some(Stmt::Directive(Directive::Extern {
                    module,
                    name,
                    func: Ident { name: func, span },
                    signature: sig,
                    span,
                }))
            }
            TokenKind::DirWasm => {
                let span = self.next().unwrap().span;
                let wasm = self.parse_wasm_block(span)?;
                Some(Stmt::Wasm(wasm))
            }
            TokenKind::DirInclude(path) => {
                let span = self.next().unwrap().span;
                Some(Stmt::Directive(Directive::Include { path, span }))
            }
            TokenKind::KwStruct => self.parse_struct(),
            TokenKind::KwEnum => self.parse_enum(),
            TokenKind::KwFn => self.parse_fn(),
            TokenKind::KwTrait => self.parse_trait(),
            TokenKind::KwImpl => self.parse_impl(),
            _ => {
                let expr = self.parse_prefix_expr()?;
                // semicolons are collected into PrefixExpr.trailing_semis
                let semi_span = if expr.trailing_semis > 0 {
                    expr.trailing_semi_span
                } else {
                    None
                };
                if expr.trailing_semis > 0 {
                    Some(Stmt::ExprSemi(expr, semi_span))
                } else {
                    Some(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_struct(&mut self) -> Option<Stmt> {
        let kw_span = self.next()?.span;
        let (name, nspan) = self.expect_ident()?;
        let type_params = self.parse_generic_params();
        self.expect(TokenKind::Colon)?;
        let mut fields = Vec::new();
        if !self.consume_if(TokenKind::Newline) {
            let span = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics.push(Diagnostic::error(
                "expected newline after struct header",
                span,
            ));
        }
        self.expect(TokenKind::Indent)?;
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            let (fname, fspan) = self.expect_ident()?;
            self.expect(TokenKind::LAngle)?;
            let fty = self.parse_type_expr()?;
            self.expect(TokenKind::RAngle)?;
            fields.push((
                Ident {
                    name: fname,
                    span: fspan,
                },
                fty,
            ));
            self.consume_if(TokenKind::Newline);
        }
        self.expect(TokenKind::Dedent)?;
        Some(Stmt::StructDef(StructDef {
            name: Ident { name, span: nspan },
            type_params,
            fields,
        }))
    }

    fn parse_enum(&mut self) -> Option<Stmt> {
        let kw_span = self.next()?.span;
        let (name, nspan) = self.expect_ident()?;
        let type_params = self.parse_generic_params();
        self.expect(TokenKind::Colon)?;
        if !self.consume_if(TokenKind::Newline) {
            let span = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics.push(Diagnostic::error(
                "expected newline after enum header",
                span,
            ));
        }
        self.expect(TokenKind::Indent)?;
        let mut variants = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            let (vname, vspan) = self.expect_ident()?;
            let payload = if self.consume_if(TokenKind::LAngle) {
                let ty = self.parse_type_expr()?;
                self.expect(TokenKind::RAngle)?;
                Some(ty)
            } else {
                None
            };
            variants.push(EnumVariant {
                name: Ident {
                    name: vname,
                    span: vspan,
                },
                payload,
            });
            self.consume_if(TokenKind::Newline);
        }
        self.expect(TokenKind::Dedent)?;
        Some(Stmt::EnumDef(EnumDef {
            name: Ident { name, span: nspan },
            type_params,
            variants,
        }))
    }

    fn parse_fn(&mut self) -> Option<Stmt> {
        let fn_span = self.next()?.span;
        let name_tok = self.expect_ident()?;
        let name = Ident {
            name: name_tok.0,
            span: name_tok.1,
        };

        // If next is LAngle, it could be generics OR signature.
        let mut type_params = Vec::new();
        if self.check(TokenKind::LAngle) {
            // Peek further if needed?
            // In NEPL, if there are two < > blocks, the first is always generics.
            // If only one, it's signature.
            // This is a bit tricky to look ahead with the current parser.
            // Let's assume if it looks like generics (idents only), it is generics.
            // NO, let's just try to parse generics first, and if what follows is ALSO an LAngle,
            // then we were correct.
            let saved_pos = self.pos;
            let saved_diags_len = self.diagnostics.len();
            let tentative_params = self.parse_generic_params();
            if self.check(TokenKind::LAngle) {
                // Success, we had generics.
                type_params = tentative_params;
            } else {
                // Not generics, backtrack.
                self.pos = saved_pos;
                self.diagnostics.truncate(saved_diags_len);
            }
        }

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
            type_params,
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
                    self.diagnostics
                        .push(Diagnostic::error("expected wasm text line", span));
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
        while self.consume_if(TokenKind::Newline) {}
        self.expect(TokenKind::Indent)?;
        let block = self.parse_block_until(TokenEnd::Dedent)?;
        self.expect(TokenKind::Dedent)?;
        Some(block)
    }

    fn parse_trait(&mut self) -> Option<Stmt> {
        let kw_span = self.next()?.span;
        let (name, nspan) = self.expect_ident()?;
        let type_params = self.parse_generic_params();
        self.expect(TokenKind::Colon)?;
        self.consume_if(TokenKind::Newline);
        self.expect(TokenKind::Indent)?;
        let mut methods = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            if let Some(Stmt::FnDef(f)) = self.parse_fn() {
                methods.push(f);
            } else {
                self.next();
            }
        }
        self.expect(TokenKind::Dedent)?;
        let end_span = self.peek_span().unwrap_or(nspan);
        Some(Stmt::Trait(TraitDef {
            name: Ident { name, span: nspan },
            type_params,
            methods,
            span: kw_span.join(end_span).unwrap_or(kw_span),
        }))
    }

    fn parse_impl(&mut self) -> Option<Stmt> {
        let kw_span = self.next()?.span;
        let type_params = self.parse_generic_params();

        let first_ty = self.parse_type_expr()?;

        let (trait_name, target_ty) = if self.consume_if(TokenKind::KwFor) {
            let target = self.parse_type_expr()?;
            let trait_ident = match first_ty {
                TypeExpr::Named(n) => Some(Ident {
                    name: n,
                    span: kw_span,
                }), // Approximation
                _ => {
                    self.diagnostics.push(Diagnostic::error(
                        "expected trait name before 'for'",
                        kw_span,
                    ));
                    None
                }
            };
            (trait_ident, target)
        } else {
            (None, first_ty)
        };

        self.expect(TokenKind::Colon)?;
        self.consume_if(TokenKind::Newline);
        self.expect(TokenKind::Indent)?;
        let mut methods = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            if let Some(Stmt::FnDef(f)) = self.parse_fn() {
                methods.push(f);
            } else {
                self.next();
            }
        }
        self.expect(TokenKind::Dedent)?;
        let end_span = self.peek_span().unwrap_or(kw_span);
        Some(Stmt::Impl(ImplDef {
            type_params,
            trait_name,
            target_ty,
            methods,
            span: kw_span.join(end_span).unwrap_or(kw_span),
        }))
    }

    fn parse_prefix_expr(&mut self) -> Option<PrefixExpr> {
        let start_span = self.peek_span().unwrap_or_else(Span::dummy);
        let mut items = Vec::new();
        let mut trailing_semis: u32 = 0;
        let mut in_trailing_semis = false;
        let mut last_semi_span: Option<Span> = None;

        while !self.is_end(&TokenEnd::Line) {
            match self.peek_kind()? {
                TokenKind::Semicolon => {
                    // semicolon marks statement end; collect and ensure nothing follows on same line
                    let sp = self.next().unwrap().span;
                    trailing_semis += 1;
                    in_trailing_semis = true;
                    last_semi_span = Some(sp);
                    continue;
                }

                // If we've seen trailing semicolons, any other token on the same
                // line is an error: only one statement per line is allowed.
                _ if in_trailing_semis => {
                    let sp = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics.push(Diagnostic::error(
                        "unexpected token after ';' (only one statement per line)",
                        sp,
                    ));
                    // recovery: skip to end of line
                    while !self.is_end(&TokenEnd::Line) {
                        self.next();
                    }
                    break;
                }
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Colon => {
                    let colon_span = self.next().unwrap().span;
                    let block = self.parse_block_after_colon()?;
                    let span = colon_span.join(block.span).unwrap_or(colon_span);
                    // Emit diagnostic showing current header item shapes (debug)
                    {
                        let mut shapes = alloc::string::String::new();
                        for (i, it) in items.iter().enumerate() {
                            if i > 0 { shapes.push_str(", "); }
                            let label = match it {
                                PrefixItem::Symbol(sym) => match sym {
                                    Symbol::Ident(id, _) => alloc::format!("Ident({})", id.name),
                                    Symbol::If(_) => alloc::format!("If"),
                                    Symbol::While(_) => alloc::format!("While"),
                                    Symbol::Let { .. } => alloc::format!("Let"),
                                    Symbol::Set { .. } => alloc::format!("Set"),
                                },
                                PrefixItem::Literal(_, _) => alloc::format!("Literal"),
                                PrefixItem::Block(_, _) => alloc::format!("Block"),
                                PrefixItem::Match(_, _) => alloc::format!("Match"),
                                PrefixItem::TypeAnnotation(_, _) => alloc::format!("TypeAnnotation"),
                                PrefixItem::Pipe(_) => alloc::format!("Pipe"),
                                PrefixItem::Tuple(_, _) => alloc::format!("Tuple"),
                                PrefixItem::Group(_, _) => alloc::format!("Group"),
                                PrefixItem::Intrinsic(_, _) => alloc::format!("Intrinsic"),
                            };
                            shapes.push_str(&label);
                        }
                        self.diagnostics.push(Diagnostic::warning(alloc::format!("pre-extract header_items=[{}]", shapes), colon_span));
                    }
                    // If this prefix line contains an `if`, try to split the following
                    // block into then/else branch blocks when top-level `else:` markers exist.
                    if items
                        .iter()
                        .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                    {
                        // Handle `if:` / `if <cond>:` layout forms by extracting
                        // 2 or 3 expressions from the indented block and splicing
                        // their items into this prefix expression. This desugars
                        // the layout into normal prefix arguments so later passes
                        // don't need a special-case split.
                        let expected = if Self::if_layout_needs_cond(&items) {
                            3
                        } else {
                            2
                        };
                        match self.extract_if_layout_exprs(block.clone(), expected, colon_span) {
                            Ok(mut args) => {
                                // Debug: emit a diagnostic describing header and arg shapes
                                let header_count = items.len();
                                let mut arg_shapes = alloc::string::String::new();
                                for (i, a) in args.iter().enumerate() {
                                    let first = a.items.first();
                                    let label = match first {
                                        Some(PrefixItem::Symbol(Symbol::Ident(id, _))) =>
                                            alloc::format!("Ident({})", id.name),
                                        Some(PrefixItem::Block(_, _)) => alloc::format!("Block"),
                                        Some(PrefixItem::Literal(_, _)) => alloc::format!("Literal"),
                                        Some(PrefixItem::Match(_, _)) => alloc::format!("Match"),
                                        Some(PrefixItem::TypeAnnotation(_, _)) => alloc::format!("TypeAnnotation"),
                                        Some(PrefixItem::Pipe(_)) => alloc::format!("Pipe"),
                                        None => alloc::format!("Empty"),
                                        _ => alloc::format!("Other"),
                                    };
                                    if i > 0 { arg_shapes.push_str(", "); }
                                    arg_shapes.push_str(&alloc::format!("{}:{}", a.items.len(), label));
                                }
                                self.diagnostics.push(Diagnostic::warning(
                                    alloc::format!("if-layout: header_items={} args=[{}]", header_count, arg_shapes),
                                    colon_span,
                                ));
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        // single item: splice it in
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
                                        // multiple items: wrap into a single Block item
                                        let wrapped = Block {
                                            items: vec![Stmt::Expr(a.clone())],
                                            span: a.span,
                                        };
                                        items.push(PrefixItem::Block(wrapped, a.span));
                                    }
                                }
                            }
                            Err(diag) => {
                                self.diagnostics.push(diag);
                                items.push(PrefixItem::Block(block, span));
                            }
                        }
                    } else {
                        items.push(PrefixItem::Block(block, span));
                    }
                    break;
                }
                // Semicolons are handled at statement level (parse_stmt),
                // do not include them inside PrefixExpr items.
                TokenKind::Pipe => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Pipe(span));
                }
                TokenKind::LAngle => {
                    let start = self.next().unwrap().span;
                    let ty = self.parse_type_expr()?;
                    self.expect(TokenKind::RAngle)?;
                    let end = self.peek_span().unwrap_or(start);
                    let span = start.join(end).unwrap_or(start);
                    items.push(PrefixItem::TypeAnnotation(ty, span));
                }
                TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::BoolLiteral(_)
                | TokenKind::UnitLiteral
                | TokenKind::StringLiteral(_) => {
                    let tok = self.next().unwrap();
                    let lit = match tok.kind {
                        TokenKind::IntLiteral(v) => Literal::Int(v),
                        TokenKind::FloatLiteral(v) => Literal::Float(v),
                        TokenKind::BoolLiteral(b) => Literal::Bool(b),
                        TokenKind::StringLiteral(s) => Literal::Str(s),
                        TokenKind::UnitLiteral => Literal::Unit,
                        _ => unreachable!(),
                    };
                    items.push(PrefixItem::Literal(lit, tok.span));
                }
                TokenKind::LParen => {
                    let lp = self.next().unwrap().span;
                    if self.consume_if(TokenKind::RParen) {
                        let rp = self.peek_span().unwrap_or(lp);
                        let span = lp.join(rp).unwrap_or(lp);
                        items.push(PrefixItem::Literal(Literal::Unit, span));
                    } else {
                        if let Some((elems, tup_span, saw_comma)) =
                            self.parse_tuple_items(lp)
                        {
                            if saw_comma {
                                items.push(PrefixItem::Tuple(elems, tup_span));
                            } else {
                                if let Some(first) = elems.into_iter().next() {
                                    items.push(PrefixItem::Group(first, tup_span));
                                }
                            }
                        } else {
                            // recovery: skip to end of line
                            while !self.is_end(&TokenEnd::Line) {
                                self.next();
                            }
                        }
                    }
                }
                TokenKind::KwLet => {
                    let _ = self.next();
                    let is_mut = self.consume_if(TokenKind::KwMut);
                    let (name, span) = self.expect_ident()?;
                    self.consume_if(TokenKind::Equals);
                    items.push(PrefixItem::Symbol(Symbol::Let {
                        name: Ident { name, span },
                        mutable: is_mut,
                    }));
                }
                TokenKind::KwSet => {
                    let set_span = self.next().unwrap().span;
                    let (name, span) = self.expect_ident()?;
                    self.consume_if(TokenKind::Equals);
                    items.push(PrefixItem::Symbol(Symbol::Set {
                        name: Ident { name, span },
                    }));
                    let _ = set_span; // span kept in symbol name
                }
                TokenKind::DirIntrinsic => {
                    let kw_span = self.next().unwrap().span;
                    let (name, _name_span) = match self.peek_kind() {
                        Some(TokenKind::StringLiteral(s)) => {
                            let tok = self.next().unwrap();
                            (s.clone(), tok.span)
                        }
                        _ => {
                            let sp = self.peek_span().unwrap_or(kw_span);
                            self.diagnostics.push(Diagnostic::error(
                                "expected string literal for intrinsic name",
                                sp,
                            ));
                            return None;
                        }
                    };

                    let mut type_args = Vec::new();
                    if self.check(TokenKind::LAngle) {
                        self.consume_if(TokenKind::LAngle);
                        loop {
                            if self.check(TokenKind::RAngle) {
                                break;
                            }
                            type_args.push(self.parse_type_expr()?);
                            if !self.consume_if(TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(TokenKind::RAngle)?;
                    }

                    let lp = if self.check(TokenKind::LParen) {
                        self.next().unwrap().span
                    } else {
                        let sp = self.peek_span().unwrap_or(kw_span);
                        self.diagnostics.push(Diagnostic::error(
                            "expected '(' after intrinsic name/res",
                            sp,
                        ));
                        return None;
                    };

                    let (args, args_span, _) = if self.consume_if(TokenKind::RParen) {
                        let rp = self.peek_span().unwrap_or(lp);
                        (Vec::new(), lp.join(rp).unwrap_or(lp), false)
                    } else {
                        self.parse_tuple_items(lp)?
                    };
                    
                    let end_span = args_span;
                    items.push(PrefixItem::Intrinsic(
                        IntrinsicExpr {
                            name,
                            type_args,
                            args,
                            span: kw_span.join(end_span).unwrap_or(kw_span),
                        },
                        kw_span.join(end_span).unwrap_or(kw_span),
                    ));
                }
                TokenKind::KwIf => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::If(span)));
                }
                TokenKind::KwWhile => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::While(span)));
                }
                TokenKind::KwMatch => {
                    let m = self.parse_match_expr()?;
                    let sp = m.span;
                    items.push(PrefixItem::Match(m, sp));
                    break;
                }
                TokenKind::Ident(name) => {
                    let tok = self.next().unwrap();
                    let mut full = name.clone();
                    let mut end_span = tok.span;
                    let mut type_args = Vec::new();
                    loop {
                        if self.check(TokenKind::LAngle) {
                            let saved = self.pos;
                            let saved_diags = self.diagnostics.len();
                            self.next(); // consume <
                            let mut ok = true;
                            let mut temp_args = Vec::new();
                            loop {
                                if let Some(ty) = self.parse_type_expr() {
                                    temp_args.push(ty);
                                } else {
                                    ok = false;
                                    break;
                                }
                                if self.consume_if(TokenKind::Comma) {
                                    continue;
                                }
                                break;
                            }
                            if ok && self.consume_if(TokenKind::RAngle) {
                                type_args.extend(temp_args);
                            } else {
                                self.pos = saved;
                                self.diagnostics.truncate(saved_diags);
                                break;
                            }
                        } else if self.check(TokenKind::PathSep) {
                            let _ = self.next();
                            if let Some(TokenKind::Ident(n2)) = self.peek_kind() {
                                let tok2 = self.next().unwrap();
                                full.push_str("::");
                                full.push_str(&n2);
                                end_span = end_span.join(tok2.span).unwrap_or(tok2.span);
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    items.push(PrefixItem::Symbol(Symbol::Ident(
                        Ident {
                            name: full,
                            span: end_span,
                        },
                        type_args,
                    )));
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics
                        .push(Diagnostic::error("unexpected token in expression", span));
                    self.next();
                }
            }
        }

        // Normalize `cond`/`then`/`else` markers so forms like
        // `if cond then A else B` and the layout `if:` forms are reduced
        // to the basic `if cond A B` shape expected by later passes.
        self.normalize_then_else(&mut items);

        // Debug: if this prefix contains an If marker, emit a diagnostic showing item shapes
        if items.iter().any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_)))) {
            let mut shapes = alloc::string::String::new();
            for (i, it) in items.iter().enumerate() {
                if i > 0 { shapes.push_str(", "); }
                let label = match it {
                    PrefixItem::Symbol(sym) => match sym {
                        Symbol::Ident(id, _) => alloc::format!("Ident({})", id.name),
                        Symbol::If(_) => alloc::format!("If"),
                        Symbol::While(_) => alloc::format!("While"),
                        Symbol::Let { .. } => alloc::format!("Let"),
                        Symbol::Set { .. } => alloc::format!("Set"),
                    },
                    PrefixItem::Literal(_, _) => alloc::format!("Literal"),
                    PrefixItem::Block(_, _) => alloc::format!("Block"),
                    PrefixItem::Match(_, _) => alloc::format!("Match"),
                    PrefixItem::TypeAnnotation(_, _) => alloc::format!("TypeAnnotation"),
                    PrefixItem::Pipe(_) => alloc::format!("Pipe"),
                    PrefixItem::Tuple(_, _) => alloc::format!("Tuple"),
                    PrefixItem::Group(_, _) => alloc::format!("Group"),
                    PrefixItem::Intrinsic(_, _) => alloc::format!("Intrinsic"),
                };
                shapes.push_str(&label);
            }
            self.diagnostics.push(Diagnostic::warning(alloc::format!("prefix-if items=[{}]", shapes), items.first().map(|i| self.item_span(i)).unwrap_or(start_span)));
        }

        let end_span = if trailing_semis > 0 {
            last_semi_span.unwrap_or(
                items
                    .last()
                    .map(|i| self.item_span(i))
                    .unwrap_or(start_span),
            )
        } else {
            items
                .last()
                .map(|i| self.item_span(i))
                .unwrap_or(start_span)
        };
        Some(PrefixExpr {
            items,
            trailing_semis,
            trailing_semi_span: last_semi_span,
            span: start_span.join(end_span).unwrap_or(start_span),
        })
    }

    fn parse_tuple_items(&mut self, lp_span: Span) -> Option<(Vec<PrefixExpr>, Span, bool)> {
        let mut elems = Vec::new();
        let mut saw_comma = false;
        loop {
            let elem = self.parse_prefix_expr_until_tuple_delim()?;
            elems.push(elem);
            if self.consume_if(TokenKind::Comma) {
                saw_comma = true;
                if self.check(TokenKind::RParen) {
                    let sp = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics.push(Diagnostic::error(
                        "tuple literal cannot end with a comma",
                        sp,
                    ));
                    let rp = self.next().unwrap().span;
                    let span = lp_span.join(rp).unwrap_or(lp_span);
                    return Some((elems, span, saw_comma));
                }
                continue;
            }
            let rp = if self.consume_if(TokenKind::RParen) {
                self.peek_span().unwrap_or(lp_span)
            } else {
                let sp = self.peek_span().unwrap_or_else(Span::dummy);
                self.diagnostics
                    .push(Diagnostic::error("expected ')' after tuple literal", sp));
                sp
            };
            let span = lp_span.join(rp).unwrap_or(lp_span);
            return Some((elems, span, saw_comma));
        }
    }

    fn parse_prefix_expr_until_tuple_delim(&mut self) -> Option<PrefixExpr> {
        let start_span = self.peek_span().unwrap_or_else(Span::dummy);
        let mut items = Vec::new();
        let mut trailing_semis: u32 = 0;
        let mut in_trailing_semis = false;
        let mut last_semi_span: Option<Span> = None;

        while !self.is_end(&TokenEnd::Line)
            && !matches!(self.peek_kind(), Some(TokenKind::Comma | TokenKind::RParen))
        {
            match self.peek_kind()? {
                TokenKind::Semicolon => {
                    let sp = self.next().unwrap().span;
                    trailing_semis += 1;
                    in_trailing_semis = true;
                    last_semi_span = Some(sp);
                    continue;
                }
                _ if in_trailing_semis => {
                    let sp = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics.push(Diagnostic::error(
                        "unexpected token after ';' (only one statement per line)",
                        sp,
                    ));
                    while !self.is_end(&TokenEnd::Line)
                        && !matches!(self.peek_kind(), Some(TokenKind::Comma | TokenKind::RParen))
                    {
                        self.next();
                    }
                    break;
                }
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Colon => {
                    let colon_span = self.next().unwrap().span;
                    let block = self.parse_block_after_colon()?;
                    let span = colon_span.join(block.span).unwrap_or(colon_span);
                    if items
                        .iter()
                        .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                    {
                        let expected = if Self::if_layout_needs_cond(&items) {
                            3
                        } else {
                            2
                        };
                        match self.extract_if_layout_exprs(block.clone(), expected, colon_span) {
                            Ok(mut args) => {
                                // Debug: emit a diagnostic describing header and arg shapes
                                let header_count = items.len();
                                let mut arg_shapes = alloc::string::String::new();
                                for (i, a) in args.iter().enumerate() {
                                    let first = a.items.first();
                                    let label = match first {
                                        Some(PrefixItem::Symbol(Symbol::Ident(id, _))) =>
                                            alloc::format!("Ident({})", id.name),
                                        Some(PrefixItem::Block(_, _)) => alloc::format!("Block"),
                                        Some(PrefixItem::Literal(_, _)) => alloc::format!("Literal"),
                                        Some(PrefixItem::Match(_, _)) => alloc::format!("Match"),
                                        Some(PrefixItem::TypeAnnotation(_, _)) => alloc::format!("TypeAnnotation"),
                                        Some(PrefixItem::Pipe(_)) => alloc::format!("Pipe"),
                                        None => alloc::format!("Empty"),
                                        _ => alloc::format!("Other"),
                                    };
                                    if i > 0 { arg_shapes.push_str(", "); }
                                    arg_shapes.push_str(&alloc::format!("{}:{}", a.items.len(), label));
                                }
                                self.diagnostics.push(Diagnostic::warning(
                                    alloc::format!("if-layout: header_items={} args=[{}]", header_count, arg_shapes),
                                    colon_span,
                                ));
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        // single item: splice it in
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
                                        // multiple items: wrap into a single Block item
                                        let wrapped = Block {
                                            items: vec![Stmt::Expr(a.clone())],
                                            span: a.span,
                                        };
                                        items.push(PrefixItem::Block(wrapped, a.span));
                                    }
                                }
                            }
                            Err(diag) => {
                                self.diagnostics.push(diag);
                                items.push(PrefixItem::Block(block, span));
                            }
                        }
                    } else {
                        items.push(PrefixItem::Block(block, span));
                    }
                    break;
                }
                TokenKind::Pipe => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Pipe(span));
                }
                TokenKind::LAngle => {
                    let start = self.next().unwrap().span;
                    let ty = self.parse_type_expr()?;
                    self.expect(TokenKind::RAngle)?;
                    let end = self.peek_span().unwrap_or(start);
                    let span = start.join(end).unwrap_or(start);
                    items.push(PrefixItem::TypeAnnotation(ty, span));
                }
                TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::BoolLiteral(_)
                | TokenKind::UnitLiteral
                | TokenKind::StringLiteral(_) => {
                    let tok = self.next().unwrap();
                    let lit = match tok.kind {
                        TokenKind::IntLiteral(v) => Literal::Int(v),
                        TokenKind::FloatLiteral(v) => Literal::Float(v),
                        TokenKind::BoolLiteral(b) => Literal::Bool(b),
                        TokenKind::StringLiteral(s) => Literal::Str(s),
                        TokenKind::UnitLiteral => Literal::Unit,
                        _ => unreachable!(),
                    };
                    items.push(PrefixItem::Literal(lit, tok.span));
                }
                TokenKind::LParen => {
                    let lp = self.next().unwrap().span;
                    if self.consume_if(TokenKind::RParen) {
                        let rp = self.peek_span().unwrap_or(lp);
                        let span = lp.join(rp).unwrap_or(lp);
                        items.push(PrefixItem::Literal(Literal::Unit, span));
                    } else {
                        if let Some((elems, tup_span, saw_comma)) =
                            self.parse_tuple_items(lp)
                        {
                            if saw_comma {
                                items.push(PrefixItem::Tuple(elems, tup_span));
                            } else {
                                let span = self.peek_span().unwrap_or(lp);
                                self.diagnostics.push(Diagnostic::error(
                                    "parenthesized expressions are not supported; use tuple syntax with commas",
                                    span,
                                ));
                                if let Some(first) = elems.into_iter().next() {
                                    items.extend(first.items);
                                }
                            }
                        } else {
                            while !self.is_end(&TokenEnd::Line)
                                && !matches!(self.peek_kind(), Some(TokenKind::Comma | TokenKind::RParen))
                            {
                                self.next();
                            }
                        }
                    }
                }
                TokenKind::KwLet => {
                    let _ = self.next();
                    let is_mut = self.consume_if(TokenKind::KwMut);
                    let (name, span) = self.expect_ident()?;
                    self.consume_if(TokenKind::Equals);
                    items.push(PrefixItem::Symbol(Symbol::Let {
                        name: Ident { name, span },
                        mutable: is_mut,
                    }));
                }
                TokenKind::KwSet => {
                    let _ = self.next();
                    let (name, span) = self.expect_ident()?;
                    items.push(PrefixItem::Symbol(Symbol::Set {
                        name: Ident { name, span },
                    }));
                }
                TokenKind::KwIf => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::If(span)));
                }
                TokenKind::KwWhile => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::While(span)));
                }
                TokenKind::KwMatch => {
                    let m = self.parse_match_expr()?;
                    let sp = m.span;
                    items.push(PrefixItem::Match(m, sp));
                    break;
                }
                TokenKind::Ident(name) => {
                    let tok = self.next().unwrap();
                    let mut full = name.clone();
                    let mut end_span = tok.span;
                    while self.check(TokenKind::PathSep) {
                        let _ = self.next();
                        if let Some(TokenKind::Ident(n2)) = self.peek_kind() {
                            let tok2 = self.next().unwrap();
                            full.push_str("::");
                            full.push_str(&n2);
                            end_span = end_span.join(tok2.span).unwrap_or(tok2.span);
                        } else {
                            break;
                        }
                    }
                    items.push(PrefixItem::Symbol(Symbol::Ident(
                        Ident {
                            name: full,
                            span: end_span,
                        },
                        Vec::new(),
                    )));
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics
                        .push(Diagnostic::error("unexpected token in expression", span));
                    self.next();
                }
            }
        }

        self.normalize_then_else(&mut items);

        let end_span = if trailing_semis > 0 {
            last_semi_span.unwrap_or(
                items
                    .last()
                    .map(|i| self.item_span(i))
                    .unwrap_or(start_span),
            )
        } else {
            items
                .last()
                .map(|i| self.item_span(i))
                .unwrap_or(start_span)
        };
        Some(PrefixExpr {
            items,
            trailing_semis,
            trailing_semi_span: last_semi_span,
            span: start_span.join(end_span).unwrap_or(start_span),
        })
    }

    fn parse_match_expr(&mut self) -> Option<MatchExpr> {
        let match_span = self.next()?.span;
        // parse scrutinee until Colon
        let scrutinee = self.parse_prefix_expr_until_colon()?;
        let colon_span = if self.check(TokenKind::Colon) {
            self.next().unwrap().span
        } else {
            let sp = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics
                .push(Diagnostic::error("expected ':' after match", sp));
            Span::dummy()
        };
        let arms = self.parse_match_arms()?;
        let span = match_span.join(colon_span).unwrap_or(match_span);
        Some(MatchExpr {
            scrutinee,
            arms,
            span,
        })
    }

    fn parse_prefix_expr_until_colon(&mut self) -> Option<PrefixExpr> {
        let start_span = self.peek_span().unwrap_or_else(Span::dummy);
        let mut items = Vec::new();
        let mut trailing_semis: u32 = 0;
        let mut last_semi_span: Option<Span> = None;
        while !self.is_end(&TokenEnd::Line) {
            if matches!(self.peek_kind(), Some(TokenKind::Colon)) {
                break;
            }
            match self.peek_kind()? {
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Colon => break,
                TokenKind::Semicolon => {
                    let sp = self.next().unwrap().span;
                    self.diagnostics
                        .push(Diagnostic::error("';' must appear at end of a line", sp));
                    // recovery: skip until colon or end of line
                    while !self.is_end(&TokenEnd::Line) {
                        self.next();
                    }
                    trailing_semis += 1;
                    last_semi_span = Some(sp);
                    break;
                }
                TokenKind::Pipe => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Pipe(span));
                }
                // Semicolons are handled at statement level (parse_stmt),
                // do not include them inside PrefixExpr items.
                TokenKind::LAngle => {
                    let start = self.next().unwrap().span;
                    let ty = self.parse_type_expr()?;
                    self.expect(TokenKind::RAngle)?;
                    let end = self.peek_span().unwrap_or(start);
                    let span = start.join(end).unwrap_or(start);
                    items.push(PrefixItem::TypeAnnotation(ty, span));
                }
                TokenKind::IntLiteral(_)
                | TokenKind::FloatLiteral(_)
                | TokenKind::BoolLiteral(_)
                | TokenKind::UnitLiteral
                | TokenKind::StringLiteral(_) => {
                    let tok = self.next().unwrap();
                    let lit = match tok.kind {
                        TokenKind::IntLiteral(v) => Literal::Int(v),
                        TokenKind::FloatLiteral(v) => Literal::Float(v),
                        TokenKind::BoolLiteral(b) => Literal::Bool(b),
                        TokenKind::StringLiteral(s) => Literal::Str(s),
                        TokenKind::UnitLiteral => Literal::Unit,
                        _ => unreachable!(),
                    };
                    items.push(PrefixItem::Literal(lit, tok.span));
                }
                TokenKind::LParen => {
                    let lp = self.next().unwrap().span;
                    if self.consume_if(TokenKind::RParen) {
                        let rp = self.peek_span().unwrap_or(lp);
                        let span = lp.join(rp).unwrap_or(lp);
                        items.push(PrefixItem::Literal(Literal::Unit, span));
                    } else {
                        if let Some((elems, tup_span, saw_comma)) =
                            self.parse_tuple_items(lp)
                        {
                            if saw_comma {
                                items.push(PrefixItem::Tuple(elems, tup_span));
                            } else {
                                let span = self.peek_span().unwrap_or(lp);
                                self.diagnostics.push(Diagnostic::error(
                                    "parenthesized expressions are not supported; use tuple syntax with commas",
                                    span,
                                ));
                                if let Some(first) = elems.into_iter().next() {
                                    items.extend(first.items);
                                }
                            }
                        } else {
                            while !self.is_end(&TokenEnd::Line) {
                                self.next();
                            }
                        }
                    }
                }
                TokenKind::Ident(name) => {
                    let tok = self.next().unwrap();
                    items.push(PrefixItem::Symbol(Symbol::Ident(
                        Ident {
                            name: name.clone(),
                            span: tok.span,
                        },
                        Vec::new(),
                    )));
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
                    let _ = self.next();
                    let (name, span) = self.expect_ident()?;
                    items.push(PrefixItem::Symbol(Symbol::Set {
                        name: Ident { name, span },
                    }));
                }
                TokenKind::KwIf => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::If(span)));
                }
                TokenKind::KwWhile => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::While(span)));
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics.push(Diagnostic::error(
                        "unexpected token in match scrutinee",
                        span,
                    ));
                    self.next();
                }
            }
        }
        // Normalize 'then'/'else' markers inside scrutinee/prefix until colon.
        self.normalize_then_else(&mut items);

        let end_span = items
            .last()
            .map(|i| self.item_span(i))
            .unwrap_or(start_span);
        Some(PrefixExpr {
            items,
            trailing_semis,
            trailing_semi_span: last_semi_span,
            span: start_span.join(end_span).unwrap_or(start_span),
        })
    }

    fn normalize_then_else(&mut self, items: &mut Vec<PrefixItem>) {
        // Remove inline `cond`/`then`/`else` tokens, but preserve a leading
        // marker when it is the first item on the line (marker form).
        let has_then_else = items.iter().any(|item| {
            matches!(
                item,
                PrefixItem::Symbol(crate::ast::Symbol::Ident(ident, _))
                    if ident.name == "then" || ident.name == "else"
            )
        });
        let mut i = 0;
        while i < items.len() {
            let remove = match &items[i] {
                PrefixItem::Symbol(crate::ast::Symbol::Ident(ident, _)) => {
                    let n = ident.name.as_str();
                    // remove only if not the first item (i != 0)
                    ((n == "then" || n == "else") && i != 0) || (n == "cond" && i != 0 && has_then_else)
                }
                _ => false,
            };
            if remove {
                items.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn take_role_from_expr(expr: &mut PrefixExpr) -> Option<IfRole> {
        // Direct marker case: first item is the marker identifier
        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _))) = expr.items.first() {
            if id.name == "cond" {
                expr.items.remove(0);
                return Some(IfRole::Cond);
            } else if id.name == "then" {
                expr.items.remove(0);
                return Some(IfRole::Then);
            } else if id.name == "else" {
                expr.items.remove(0);
                return Some(IfRole::Else);
            }
        }
        // The marker may be wrapped inside a Block item anywhere in the expr's items.
        // Scan each item and, if found, remove the marker and normalize the whole
        // expr into a single Block PrefixItem containing the remaining inner expr.
        for i in 0..expr.items.len() {
            if let PrefixItem::Block(block, block_span) = &mut expr.items[i] {
                if let Some(stmt) = block.items.first_mut() {
                    if let Stmt::Expr(inner) = stmt {
                        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _))) = inner.items.first() {
                            let role = if id.name == "cond" {
                                IfRole::Cond
                            } else if id.name == "then" {
                                IfRole::Then
                            } else if id.name == "else" {
                                IfRole::Else
                            } else {
                                continue;
                            };
                            // remove the marker token
                            inner.items.remove(0);
                            // Wrap the remaining inner expression(s) into a single Block
                            // so downstream code sees a single PrefixItem for the branch.
                            let wrapped_block = Block {
                                items: vec![Stmt::Expr(inner.clone())],
                                span: *block_span,
                            };
                            expr.items = vec![PrefixItem::Block(wrapped_block, *block_span)];
                            return Some(role);
                        }
                    }
                }
            }
        }

        None
    }

    fn if_layout_needs_cond(items: &[PrefixItem]) -> bool {
        // Detect `... if:` or `... if cond:` (no real cond-expr yet)
        // Ignore trailing type annotations for detection.
        let mut tail: Vec<&PrefixItem> = items.iter().collect();
        while let Some(PrefixItem::TypeAnnotation(_, _)) = tail.last().copied() {
            tail.pop();
        }
        match tail.as_slice() {
            [.., PrefixItem::Symbol(Symbol::If(_))] => true,
            _ => false,
        }
    }

    fn extract_if_layout_exprs(
        &mut self,
        block: Block,
        expected: usize,
        header_span: Span,
    ) -> Result<Vec<PrefixExpr>, Diagnostic> {
        // Collect only expression statements
        let mut exprs = Vec::new();
        for stmt in block.items {
            match stmt {
                Stmt::Expr(e) => exprs.push(e),
                _ => {
                    return Err(Diagnostic::error(
                        "if-layout block may contain only expressions",
                        header_span,
                    ));
                }
            }
        }

        // DEBUG: record basic shape of extracted exprs for troubleshooting
        for (i, ex) in exprs.iter().enumerate() {
            let first = ex.items.first();
            let label = match first {
                Some(PrefixItem::Symbol(Symbol::Ident(id, _))) =>
                    alloc::format!("Ident({})", id.name),
                Some(PrefixItem::Block(_, _)) => alloc::format!("Block"),
                Some(PrefixItem::Literal(_, _)) => alloc::format!("Literal"),
                Some(PrefixItem::Match(_, _)) => alloc::format!("Match"),
                Some(PrefixItem::TypeAnnotation(_, _)) => alloc::format!("TypeAnnotation"),
                Some(PrefixItem::Pipe(_)) => alloc::format!("Pipe"),
                None => alloc::format!("Empty"),
                _ => alloc::format!("Other"),
            };
            self.diagnostics.push(Diagnostic::warning(
                alloc::format!("if-layout expr[{}] first={}", i, label),
                header_span,
            ));
        }

        // Assign by role labels if present, otherwise by order.
        let mut slots: Vec<Option<PrefixExpr>> = vec![None; expected];

        let role_to_index = |role: IfRole| -> Option<usize> {
            match (expected, role) {
                (3, IfRole::Cond) => Some(0),
                (3, IfRole::Then) => Some(1),
                (3, IfRole::Else) => Some(2),
                (2, IfRole::Then) => Some(0),
                (2, IfRole::Else) => Some(1),
                _ => None,
            }
        };

        let mut next_unfilled = 0usize;
        for mut e in exprs {
            let role = Self::take_role_from_expr(&mut e);
            if let Some(r) = role {
                let idx = match role_to_index(r) {
                    Some(i) => i,
                    None => {
                        return Err(Diagnostic::error(
                            "invalid marker in this if-layout form",
                            e.span,
                        ));
                    }
                };
                if slots[idx].is_some() {
                    return Err(Diagnostic::error(
                        "duplicate marker in if-layout block",
                        e.span,
                    ));
                }
                // If the payload is a single Block item and the marker was present,
                // unwrap it so the downstream typechecker sees a Block when needed.
                if let Some(PrefixItem::Block(_, _)) = e.items.first() {
                    // leave as-is; marker removal already happened
                }
                slots[idx] = Some(e);
            } else {
                // positional fill
                while next_unfilled < expected && slots[next_unfilled].is_some() {
                    next_unfilled += 1;
                }
                if next_unfilled >= expected {
                    return Err(Diagnostic::error(
                        "too many expressions in if-layout block",
                        e.span,
                    ));
                }
                slots[next_unfilled] = Some(e);
                next_unfilled += 1;
            }
        }

        if slots.iter().any(|s| s.is_none()) {
            return Err(Diagnostic::error(
                "missing expression(s) in if-layout block",
                header_span,
            ));
        }

        Ok(slots.into_iter().map(|s| s.unwrap()).collect())
    }

    fn parse_match_arms(&mut self) -> Option<Vec<MatchArm>> {
        if !self.consume_if(TokenKind::Newline) {
            let sp = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics
                .push(Diagnostic::error("expected newline after match ':'", sp));
        }
        self.expect(TokenKind::Indent)?;
        let mut arms = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(TokenKind::Newline) {
                continue;
            }
            let (vname_tok, vspan_tok) = self.expect_ident()?;
            let mut vname = vname_tok;
            let mut vspan = vspan_tok;
            while self.consume_if(TokenKind::PathSep) {
                let (part, pspan) = self.expect_ident()?;
                vname.push_str("::");
                vname.push_str(&part);
                vspan = vspan.join(pspan).unwrap_or(vspan);
            }

            let bind = if let Some(TokenKind::Ident(bn)) = self.peek_kind() {
                let (bnm, bspan) = self.expect_ident()?;
                Some(Ident {
                    name: bnm,
                    span: bspan,
                })
            } else {
                None
            };
            self.expect(TokenKind::Colon)?;
            let body = self.parse_block_after_colon()?;
            arms.push(MatchArm {
                variant: Ident {
                    name: vname,
                    span: vspan,
                },
                bind,
                body: body.clone(),
                span: vspan,
            });
        }
        self.expect(TokenKind::Dedent)?;
        Some(arms)
    }

    fn parse_generic_params(&mut self) -> Vec<Ident> {
        let mut params = Vec::new();
        if self.consume_if(TokenKind::LAngle) {
            loop {
                let mut has_dot = false;
                if self.consume_if(TokenKind::Dot) {
                    has_dot = true;
                }
                if let Some((name, span)) = self.expect_ident() {
                    if !has_dot {
                        self.diagnostics.push(Diagnostic::error(
                            "type parameter must be written as .T",
                            span,
                        ));
                    }
                    params.push(Ident { name, span });
                } else {
                    break;
                }
                if !self.consume_if(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RAngle);
        }
        params
    }

    fn parse_type_expr(&mut self) -> Option<TypeExpr> {
        match self.peek_kind()? {
            TokenKind::UnitLiteral => {
                self.next();
                Some(TypeExpr::Unit)
            }
            TokenKind::Ident(name) => {
                let _ = self.next();
                let mut ty = match name.as_str() {
                    "i32" => TypeExpr::I32,
                    "f32" => TypeExpr::F32,
                    "bool" => TypeExpr::Bool,
                    "never" => TypeExpr::Never,
                    "str" => TypeExpr::Str,
                    "Box" => {
                        self.expect(TokenKind::LAngle)?;
                        let inner = self.parse_type_expr()?;
                        self.expect(TokenKind::RAngle)?;
                        return Some(TypeExpr::Boxed(Box::new(inner)));
                    }
                    _ => TypeExpr::Named(name.clone()),
                };

                if self.consume_if(TokenKind::LAngle) {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type_expr()?);
                        if self.consume_if(TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                    self.expect(TokenKind::RAngle)?;
                    ty = TypeExpr::Apply(Box::new(ty), args);
                }
                Some(ty)
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
                let lp_span = self.next().unwrap().span;
                // Empty parens: could be unit type or zero-arg function params.
                if self.check(TokenKind::RParen) {
                    let rp_span = self.next().unwrap().span;
                    if let Some(TokenKind::Arrow(eff)) = self.peek_kind() {
                        // zero-arg function type
                        let eff_copy = eff;
                        self.next();
                        let result = self.parse_type_expr()?;
                        Some(TypeExpr::Function {
                            params: Vec::new(),
                            result: Box::new(result),
                            effect: eff_copy,
                        })
                    } else {
                        // treat as unit type
                        Some(TypeExpr::Unit)
                    }
                } else {
                    let mut params = Vec::new();
                    let mut saw_comma = false;
                    loop {
                        let ty = self.parse_type_expr()?;
                        params.push(ty);
                        if self.consume_if(TokenKind::Comma) {
                            saw_comma = true;
                            continue;
                        }
                        break;
                    }
                    self.expect(TokenKind::RParen)?;
                    if let Some(TokenKind::Arrow(eff)) = self.peek_kind() {
                        let eff_copy = eff;
                        self.next();
                        let result = self.parse_type_expr()?;
                        Some(TypeExpr::Function {
                            params,
                            result: Box::new(result),
                            effect: eff_copy,
                        })
                    } else if saw_comma || params.len() > 1 {
                        Some(TypeExpr::Tuple(params))
                    } else {
                        params.into_iter().next()
                    }
                }
            }
            TokenKind::Ampersand => {
                let _ = self.next();
                let is_mut = self.consume_if(TokenKind::KwMut);
                let inner = self.parse_type_expr()?;
                Some(TypeExpr::Reference(Box::new(inner), is_mut))
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
            let (span, found) = if let Some(tok) = self.peek() {
                (tok.span, alloc::format!("{:?}", tok.kind))
            } else {
                (Span::dummy(), "EOF".to_string())
            };
            self.diagnostics.push(Diagnostic::error(
                alloc::format!("expected {:?}, found {}", kind, found),
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

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
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
            TokenEnd::Dedent => matches!(
                self.peek_kind(),
                Some(TokenKind::Dedent) | Some(TokenKind::Eof)
            ),
            TokenEnd::Line => matches!(
                self.peek_kind(),
                Some(TokenKind::Newline) | Some(TokenKind::Dedent) | Some(TokenKind::Eof)
            ),
        }
    }

    fn item_span(&self, item: &PrefixItem) -> Span {
        match item {
            PrefixItem::Literal(_, sp) => *sp,
            PrefixItem::Symbol(Symbol::Ident(id, _)) => id.span,
            PrefixItem::Symbol(Symbol::Let { name, .. }) => name.span,
            PrefixItem::Symbol(Symbol::Set { name }) => name.span,
            PrefixItem::Symbol(Symbol::If(sp)) => *sp,
            PrefixItem::Symbol(Symbol::While(sp)) => *sp,
            PrefixItem::TypeAnnotation(_, sp) => *sp,
            PrefixItem::Block(_, sp) => *sp,
            PrefixItem::Match(_, sp) => *sp,
            PrefixItem::Pipe(sp) => *sp,
            PrefixItem::Tuple(_, sp) => *sp,
            PrefixItem::Group(_, sp) => *sp,
            PrefixItem::Intrinsic(_, sp) => *sp,
        }
    }

    fn stmt_span(&self, stmt: &Stmt) -> Span {
        match stmt {
            Stmt::Directive(d) => match d {
                Directive::Entry { name } => name.span,
                Directive::Target { span, .. } => *span,
                Directive::Import { span, .. } => *span,
                Directive::Use { span, .. } => *span,
                Directive::IfTarget { span, .. } => *span,
                Directive::IndentWidth { span, .. } => *span,
                Directive::Extern { span, .. } => *span,
                Directive::Include { span, .. } => *span,
            },
            Stmt::FnDef(f) => f.name.span,
            Stmt::StructDef(s) => s.name.span,
            Stmt::EnumDef(e) => e.name.span,
            Stmt::Wasm(w) => w.span,
            Stmt::Expr(e) => e.span,
            Stmt::ExprSemi(e, _) => e.span,
            Stmt::Trait(t) => t.span,
            Stmt::Impl(i) => i.span,
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
        (DirInclude(_), DirInclude(_)) => true,
        (DirExtern { .. }, DirExtern { .. }) => true,
        (DirTarget(_), DirTarget(_)) => true,
        _ => core::mem::discriminant(a) == core::mem::discriminant(b),
    }
}

fn parse_type_expr_str(s: &str, span: Span, diags: &mut Vec<Diagnostic>) -> Option<TypeExpr> {
    // Very small parser for signatures like <(i32,i32)->i32>
    let trimmed = s.trim();
    if !trimmed.starts_with('<') || !trimmed.ends_with('>') {
        diags.push(Diagnostic::error("invalid type signature in #extern", span));
        return None;
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    // find -> or *>
    let effect = if let Some(idx) = inner.find("*>") {
        (Effect::Impure, idx)
    } else if let Some(idx) = inner.find("->") {
        (Effect::Pure, idx)
    } else {
        diags.push(Diagnostic::error("missing -> or *> in signature", span));
        return None;
    };
    let (eff, split_idx) = effect;
    let (params_part, ret_part) = inner.split_at(split_idx);
    let ret_part = &ret_part[2..];
    let params_clean = params_part
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')');
    let mut params = Vec::new();
    if !params_clean.is_empty() {
        for p in params_clean.split(',') {
            params.push(simple_type_atom(p.trim(), span, diags)?);
        }
    }
    let result = simple_type_atom(ret_part.trim(), span, diags)?;
    Some(TypeExpr::Function {
        params,
        result: Box::new(result),
        effect: eff,
    })
}

fn simple_type_atom(t: &str, span: Span, diags: &mut Vec<Diagnostic>) -> Option<TypeExpr> {
    match t {
        "i32" => Some(TypeExpr::I32),
        "f32" => Some(TypeExpr::F32),
        "bool" => Some(TypeExpr::Bool),
        "never" => Some(TypeExpr::Never),
        "str" => Some(TypeExpr::Str),
        "()" => Some(TypeExpr::Unit),
        _ if t.starts_with('.') => {
            Some(TypeExpr::Label(Some(t.trim_start_matches('.').to_string())))
        }
        _ if t.is_empty() => Some(TypeExpr::Label(None)),
        _ => {
            diags.push(Diagnostic::error("unknown type in signature", span));
            None
        }
    }
}
