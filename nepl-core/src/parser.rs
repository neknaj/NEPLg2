//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).
//! Parser for NEPLG2 surface syntax (prefix + indentation blocks).
#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WhileRole {
    Cond,
    Do,
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
        depth: 0,
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
    depth: usize,
}

impl Parser {
    fn try_extract_lambda_params(item: &PrefixItem) -> Option<(Vec<Ident>, Span)> {
        fn expr_to_ident(expr: &PrefixExpr) -> Option<Ident> {
            if expr.items.len() != 1 {
                return None;
            }
            match &expr.items[0] {
                PrefixItem::Symbol(Symbol::Ident(id, targs, _)) if targs.is_empty() => Some(id.clone()),
                _ => None,
            }
        }

        match item {
            PrefixItem::Group(expr, span) => {
                let id = expr_to_ident(expr)?;
                Some((vec![id], *span))
            }
            PrefixItem::Tuple(exprs, span) => {
                let mut params = Vec::new();
                for e in exprs {
                    params.push(expr_to_ident(e)?);
                }
                if params.is_empty() {
                    return None;
                }
                Some((params, *span))
            }
            _ => None,
        }
    }

    fn build_lambda_block(&self, params: Vec<Ident>, params_span: Span, body: Block) -> PrefixItem {
        let lambda_name = alloc::format!(
            "__lambda_{}_{}_{}",
            params_span.file_id.0,
            params_span.start,
            params_span.end
        );
        let name_ident = Ident {
            name: lambda_name.clone(),
            span: params_span,
        };
        let lambda_span = params_span.join(body.span).unwrap_or(params_span);
        let fn_def = FnDef {
            vis: Visibility::Private,
            name: name_ident.clone(),
            type_params: Vec::new(),
            signature: Self::infer_signature_from_params(params.len()),
            params,
            body: FnBody::Parsed(body),
        };
        let value_expr = PrefixExpr {
            items: vec![PrefixItem::Symbol(Symbol::Ident(
                name_ident,
                Vec::new(),
                false,
            ))],
            trailing_semis: 0,
            trailing_semi_span: None,
            span: params_span,
        };
        let block = Block {
            items: vec![Stmt::FnDef(fn_def), Stmt::Expr(value_expr)],
            span: lambda_span,
        };
        PrefixItem::Block(block, lambda_span)
    }

    fn peek_kind_at(&self, offset: usize) -> Option<&TokenKind> {
        self.tokens.get(self.pos + offset).map(|t| &t.kind)
    }
    fn parse_visibility(&mut self) -> Visibility {
        if self.consume_if(&TokenKind::KwPub) {
            Visibility::Pub
        } else {
            Visibility::Private
        }
    }

    fn expect_with_span(&mut self, kind: &TokenKind) -> Option<Span> {
        if self.consume_if(kind) {
            if let Some(tok) = self.tokens.get(self.pos.saturating_sub(1)) {
                Some(tok.span)
            } else {
                Some(Span::dummy())
            }
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
    fn parse_module(&mut self) -> Option<Module> {
        let root = self.parse_block_until(TokenEnd::Eof)?;
        Some(Module {
            indent_width: self.indent_width,
            directives: self.directives.clone(),
            root,
        })
    }

    fn parse_block_until(&mut self, end: TokenEnd) -> Option<Block> {
        self.parse_block_until_internal(end)
    }


    fn parse_block_until_internal(&mut self, end: TokenEnd) -> Option<Block> {
        let mut items = Vec::new();
        let mut start_span = self.peek_span().unwrap_or_else(Span::dummy);
        // O(1) flag: tracks if the previous statement contains an 'if' expression
        let mut prev_has_if = false;

        while !self.is_end(&end) {
            if self.consume_if(&TokenKind::Newline) {
                continue;
            }
            if matches!(self.peek_kind(), Some(TokenKind::Dedent))
                && matches!(end, TokenEnd::Dedent)
            {
                break;
            }

            let mut stmt = self.parse_stmt()?;

            // Glued Else Logic: merge 'else:' marker statements into preceding 'if:' expressions.
            // O(1) check using prev_has_if flag and peek_role_from_expr
            let mut merged = false;
            if prev_has_if {
                // Check if current statement starts with 'else' marker - O(1)
                let is_else = if let Stmt::Expr(curr_e) | Stmt::ExprSemi(curr_e, _) = &stmt {
                    Self::peek_role_from_expr(curr_e) == Some(IfRole::Else)
                } else {
                    false
                };
                
                if is_else {
                    if let Some(prev) = items.last_mut() {
                        if let Stmt::Expr(pe) | Stmt::ExprSemi(pe, _) = prev {
                            self.collapse_if_then_tail_for_glued_else(pe);
                            // Extract and modify the current expression
                            if let Stmt::Expr(ref mut curr_e) | Stmt::ExprSemi(ref mut curr_e, _) = &mut stmt {
                                // Remove the 'else' marker and get remaining items
                                Self::take_role_from_expr(curr_e);
                                let mut curr_items = core::mem::take(&mut curr_e.items);
                                pe.items.extend(curr_items);
                                pe.span = pe.span.join(curr_e.span).unwrap_or(pe.span);
                                merged = true;
                                // After merging, the combined statement still has 'if'
                                // prev_has_if remains true
                            }
                        }
                    }
                }
            }

            if merged {
                continue;
            }

            // Update prev_has_if flag - O(1) check for if symbol
            prev_has_if = match &stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => {
                    // Check if expression contains Symbol::If (typically near the end before Block)
                    e.items.iter().any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                }
                _ => false,
            };

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

    fn collapse_if_then_tail_for_glued_else(&self, expr: &mut PrefixExpr) {
        let if_pos = expr
            .items
            .iter()
            .position(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))));
        let Some(if_pos) = if_pos else {
            return;
        };
        if matches!(expr.items.last(), Some(PrefixItem::Block(_, _))) {
            return;
        }
        let then_start = if_pos + 2;
        if expr.items.len() <= then_start + 1 {
            return;
        }

        let trailing_items = expr.items.split_off(then_start);
        let mut block_items = Vec::new();
        for item in trailing_items {
            let span = self.item_span(&item);
            block_items.push(Stmt::Expr(PrefixExpr {
                items: vec![item],
                trailing_semis: 0,
                trailing_semi_span: None,
                span,
            }));
        }
        if block_items.is_empty() {
            return;
        }
        let block_span = if let (Some(first), Some(last)) = (block_items.first(), block_items.last()) {
            self.stmt_span(first)
                .join(self.stmt_span(last))
                .unwrap_or_else(|| self.stmt_span(first))
        } else {
            expr.span
        };
        expr.items.push(PrefixItem::Block(
            Block {
                items: block_items,
                span: block_span,
            },
            block_span,
        ));
        expr.span = expr.span.join(block_span).unwrap_or(expr.span);
    }

    fn parse_import_directive(&mut self, text: &str, span: Span) -> Directive {
        let mut rest = text.trim();
        let mut vis = Visibility::Private;
        if let Some(r) = rest.strip_prefix("pub") {
            vis = Visibility::Pub;
            rest = r.trim();
        }
        // path
        let mut path = String::new();
        if rest.starts_with('"') {
            if let Some(end) = rest[1..].find('"') {
                path = rest[1..1 + end].to_string();
                rest = &rest[1 + end + 1..];
            }
        } else {
            let mut parts = rest.splitn(2, char::is_whitespace);
            path = parts.next().unwrap_or("").to_string();
            rest = parts.next().unwrap_or("");
        }
        rest = rest.trim();
        let clause = if rest.is_empty() {
            ImportClause::DefaultAlias
        } else if let Some(r) = rest.strip_prefix("as") {
            let mut c = r.trim();
            if c.starts_with('*') {
                ImportClause::Open
            } else if c.starts_with("@merge") {
                ImportClause::Merge
            } else if c.starts_with('{') {
                let mut items = Vec::new();
                if let Some(end) = c.find('}') {
                    let inner = &c[1..end];
                    for part in inner.split(',') {
                        let p = part.trim();
                        if p.is_empty() {
                            continue;
                        }
                        let glob = p.ends_with("::*");
                        if glob {
                            let name = p.trim_end_matches("::*").to_string();
                            items.push(ImportItem {
                                name,
                                alias: None,
                                glob: true,
                            });
                            continue;
                        }
                        let mut segs = p.split_whitespace();
                        let first = segs.next().unwrap_or("").to_string();
                        let mut alias = None;
                        if let Some("as") = segs.next() {
                            alias = segs.next().map(|s| s.to_string());
                        }
                        items.push(ImportItem {
                            name: first,
                            alias,
                            glob: false,
                        });
                    }
                    ImportClause::Selective(items)
                } else {
                    ImportClause::DefaultAlias
                }
            } else {
                let alias = c.split_whitespace().next().unwrap_or("").to_string();
                ImportClause::Alias(alias)
            }
        } else {
            ImportClause::DefaultAlias
        };
        Directive::Import {
            path,
            clause,
            vis,
            span,
        }
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
                let (text, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirImport(p) = tok.kind.clone() {
                            (p, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(self.parse_import_directive(&text, span)))
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
            TokenKind::DirIfProfile(_) => {
                let (profile, span) = match self.next() {
                    Some(tok) => {
                        if let TokenKind::DirIfProfile(p) = tok.kind.clone() {
                            (p, tok.span)
                        } else {
                            unreachable!()
                        }
                    }
                    None => return None,
                };
                Some(Stmt::Directive(Directive::IfProfile { profile, span }))
            }
            TokenKind::DirIndentWidth(width) => {
                let span = self.next().unwrap().span;
                Some(Stmt::Directive(Directive::IndentWidth { width: width, span }))
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
                    module: module.to_string(),
                    name: name.to_string(),
                    func: Ident { name: func.to_string(), span },
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
                Some(Stmt::Directive(Directive::Include { path: path.to_string(), span }))
            }
            TokenKind::DirPrelude(p) => {
                let span = self.next().unwrap().span;
                Some(Stmt::Directive(Directive::Prelude { path: p.to_string(), span }))
            }
            TokenKind::DirNoPrelude => {
                let span = self.next().unwrap().span;
                Some(Stmt::Directive(Directive::NoPrelude { span }))
            }
            TokenKind::KwPub => {
                match self.peek_kind_at(1) {
                    Some(TokenKind::KwStruct) => self.parse_struct(),
                    Some(TokenKind::KwEnum) => self.parse_enum(),
                    Some(TokenKind::KwFn) => self.parse_fn(),
                    Some(TokenKind::KwTrait) => self.parse_trait(),
                    Some(TokenKind::KwImpl) => self.parse_impl(),
                    _ => {
                        let span = self.peek_span().unwrap_or_else(Span::dummy);
                        self.diagnostics.push(Diagnostic::error(
                            "unexpected token after pub",
                            span,
                        ));
                        self.next();
                        None
                    }
                }
            },
            TokenKind::KwStruct => self.parse_struct(),
            TokenKind::KwEnum => self.parse_enum(),
            TokenKind::KwFn => self.parse_fn(),
            TokenKind::KwTrait => self.parse_trait(),
            TokenKind::KwImpl => self.parse_impl(),
            TokenKind::KwLet => {
                if let Some(def) = self.parse_let_fn_def() {
                    Some(def)
                } else {
                    self.parse_expr_stmt()
                }
            }
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_expr_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_prefix_expr()?;
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

    fn infer_signature_from_params(param_count: usize) -> TypeExpr {
        let mut params = Vec::new();
        for _ in 0..param_count {
            params.push(TypeExpr::Label(None));
        }
        TypeExpr::Function {
            params,
            result: Box::new(TypeExpr::Label(None)),
            effect: Effect::Pure,
        }
    }

    fn parse_param_list(&mut self) -> Option<Vec<Ident>> {
        self.expect(&TokenKind::LParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let (pname, pspan) = self.expect_ident()?;
                params.push(Ident {
                    name: pname,
                    span: pspan,
                });
                if self.consume_if(&TokenKind::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(&TokenKind::RParen)?;
        Some(params)
    }

    fn parse_let_fn_def(&mut self) -> Option<Stmt> {
        let saved_pos = self.pos;
        let saved_diags_len = self.diagnostics.len();

        if !self.consume_if(&TokenKind::KwLet) {
            return None;
        }
        let (name, nspan) = match self.expect_ident() {
            Some(v) => v,
            None => {
                self.pos = saved_pos;
                self.diagnostics.truncate(saved_diags_len);
                return None;
            }
        };

        let signature = if self.consume_if(&TokenKind::LAngle) {
            let sig = match self.parse_type_expr() {
                Some(s) => s,
                None => {
                    self.pos = saved_pos;
                    self.diagnostics.truncate(saved_diags_len);
                    return None;
                }
            };
            if !self.consume_if(&TokenKind::RAngle) {
                self.pos = saved_pos;
                self.diagnostics.truncate(saved_diags_len);
                return None;
            }
            Some(sig)
        } else {
            None
        };

        if !self.check(&TokenKind::LParen) {
            self.pos = saved_pos;
            self.diagnostics.truncate(saved_diags_len);
            return None;
        }
        let params = match self.parse_param_list() {
            Some(p) => p,
            None => {
                self.pos = saved_pos;
                self.diagnostics.truncate(saved_diags_len);
                return None;
            }
        };
        if !self.consume_if(&TokenKind::Colon) {
            self.pos = saved_pos;
            self.diagnostics.truncate(saved_diags_len);
            return None;
        }
        let body = match self.parse_block_after_colon() {
            Some(b) => b,
            None => {
                self.pos = saved_pos;
                self.diagnostics.truncate(saved_diags_len);
                return None;
            }
        };

        let fn_body = match body.items.first() {
            Some(Stmt::Wasm(wb)) if body.items.len() == 1 => FnBody::Wasm(wb.clone()),
            _ => FnBody::Parsed(body),
        };

        Some(Stmt::FnDef(FnDef {
            vis: Visibility::Private,
            name: Ident { name, span: nspan },
            type_params: Vec::new(),
            signature: signature
                .unwrap_or_else(|| Self::infer_signature_from_params(params.len())),
            params,
            body: fn_body,
        }))
    }

    fn parse_struct(&mut self) -> Option<Stmt> {
        let vis = self.parse_visibility();
        let kw_span = self.expect_with_span(&TokenKind::KwStruct)?;
        let (name, nspan) = self.expect_ident()?;
        let type_params = self.parse_generic_params();
        self.expect(&TokenKind::Colon)?;
        let mut fields = Vec::new();
        if self.consume_if(&TokenKind::Newline) {
            self.expect(&TokenKind::Indent)?;
            while !self.check(&TokenKind::Dedent) && !self.is_eof() {
                if self.consume_if(&TokenKind::Newline) {
                    continue;
                }
                let (fname, fspan) = self.expect_ident()?;
                self.expect(&TokenKind::LAngle)?;
                let fty = self.parse_type_expr()?;
                self.expect(&TokenKind::RAngle)?;
                fields.push((
                    Ident {
                        name: fname,
                        span: fspan,
                    },
                    fty,
                ));
                self.consume_if(&TokenKind::Newline);
            }
            self.expect(&TokenKind::Dedent)?;
        } else {
            while !self.is_end(&TokenEnd::Line) {
                let (fname, fspan) = self.expect_ident()?;
                self.expect(&TokenKind::LAngle)?;
                let fty = self.parse_type_expr()?;
                self.expect(&TokenKind::RAngle)?;
                fields.push((
                    Ident {
                        name: fname,
                        span: fspan,
                    },
                    fty,
                ));
                if !self.consume_if(&TokenKind::Semicolon) {
                    break;
                }
            }
        }
        Some(Stmt::StructDef(StructDef {
            vis,
            name: Ident { name, span: nspan },
            type_params,
            fields,
        }))
    }

    fn parse_enum(&mut self) -> Option<Stmt> {
        let vis = self.parse_visibility();
        let kw_span = self.expect_with_span(&TokenKind::KwEnum)?;
        let (name, nspan) = self.expect_ident()?;
        let type_params = self.parse_generic_params();
        self.expect(&TokenKind::Colon)?;
        let mut variants = Vec::new();
        if self.consume_if(&TokenKind::Newline) {
            self.expect(&TokenKind::Indent)?;
            while !self.check(&TokenKind::Dedent) && !self.is_eof() {
                if self.consume_if(&TokenKind::Newline) {
                    continue;
                }
                let (vname, vspan) = self.expect_ident()?;
                let payload = if self.consume_if(&TokenKind::LAngle) {
                    let ty = self.parse_type_expr()?;
                    self.expect(&TokenKind::RAngle)?;
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
                self.consume_if(&TokenKind::Newline);
            }
            self.expect(&TokenKind::Dedent)?;
        } else {
            while !self.is_end(&TokenEnd::Line) {
                let (vname, vspan) = self.expect_ident()?;
                let payload = if self.consume_if(&TokenKind::LAngle) {
                    let ty = self.parse_type_expr()?;
                    self.expect(&TokenKind::RAngle)?;
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
                if !self.consume_if(&TokenKind::Semicolon) {
                    break;
                }
            }
        }
        Some(Stmt::EnumDef(EnumDef {
            vis,
            name: Ident { name, span: nspan },
            type_params,
            variants,
        }))
    }

    fn parse_fn(&mut self) -> Option<Stmt> {
        let vis = self.parse_visibility();
        let fn_span = self.expect_with_span(&TokenKind::KwFn)?;
        let name_tok = self.expect_ident()?;
        let name = Ident {
            name: name_tok.0.clone(),
            span: name_tok.1,
        };


        if (matches!(self.peek_kind(), Some(TokenKind::Ident(_)))
            && matches!(self.peek_kind_at(1), Some(TokenKind::Semicolon)))
            || (matches!(self.peek_kind(), Some(TokenKind::At))
                && matches!(self.peek_kind_at(1), Some(TokenKind::Ident(_)))
                && matches!(self.peek_kind_at(2), Some(TokenKind::Semicolon)))
        {
            self.consume_if(&TokenKind::At);
            let target_tok = self.expect_ident()?;
            let target = Ident {
                name: target_tok.0,
                span: target_tok.1,
            };
            self.expect(&TokenKind::Semicolon)?;
            return Some(Stmt::FnAlias(FnAlias { vis, name, target }));
        }

        // If next is LAngle, it could be generics OR signature.
        let mut type_params = Vec::new();
        if self.check(&TokenKind::LAngle) {
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
            if self.check(&TokenKind::LAngle) {
                // Success, we had generics.
                type_params = tentative_params;
            } else {
                // Not generics, backtrack.
                self.pos = saved_pos;
                self.diagnostics.truncate(saved_diags_len);
            }
        }

        let signature = if self.consume_if(&TokenKind::LAngle) {
            let sig = self.parse_type_expr()?;
            self.expect(&TokenKind::RAngle)?;
            sig
        } else {
            Self::infer_signature_from_params(0)
        };

        let params = self.parse_param_list()?;
        let signature = match signature {
            TypeExpr::Function {
                params: p,
                result,
                effect,
            } if p.is_empty() => {
                let mut inferred_params = Vec::new();
                for _ in 0..params.len() {
                    inferred_params.push(TypeExpr::Label(None));
                }
                TypeExpr::Function {
                    params: inferred_params,
                    result,
                    effect,
                }
            }
            other => other,
        };
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block_after_colon()?;

        let fn_body = match body.items.first() {
            Some(Stmt::Wasm(wb)) if body.items.len() == 1 => FnBody::Wasm(wb.clone()),
            _ => FnBody::Parsed(body),
        };

        Some(Stmt::FnDef(FnDef {
            vis,
            name,
            type_params,
            signature,
            params,
            body: fn_body,
        }))
    }

    fn parse_wasm_block(&mut self, dir_span: Span) -> Option<WasmBlock> {
        self.consume_if(&TokenKind::Colon);
        if self.consume_if(&TokenKind::Newline) {
            // ok
        }
        self.expect(&TokenKind::Indent)?;

        let mut lines = Vec::new();
        let mut start_span = dir_span;
        while !self.check(&TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(&TokenKind::Newline) {
                continue;
            }
            match self.peek_kind() {
                Some(TokenKind::WasmText(_)) => {
                    let tok = self.next().unwrap();
                    if let TokenKind::WasmText(text) = tok.kind {
                        lines.push(text);
                    }
                    start_span = start_span.join(tok.span).unwrap_or(tok.span);
                    self.consume_if(&TokenKind::Newline);
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics
                        .push(Diagnostic::error("expected wasm text line", span));
                    self.next();
                }
            }
        }

        self.expect(&TokenKind::Dedent)?;
        let end_span = self.peek_span().unwrap_or_else(Span::dummy);
        Some(WasmBlock {
            lines,
            span: dir_span.join(end_span).unwrap_or(dir_span),
        })
    }

    fn parse_block_after_colon(&mut self) -> Option<Block> {
        if self.consume_if(&TokenKind::Newline) {
            while self.consume_if(&TokenKind::Newline) {}
            self.expect(&TokenKind::Indent)?;
            let block = self.parse_block_until(TokenEnd::Dedent)?;
            self.expect(&TokenKind::Dedent)?;
            Some(block)
        } else {
            let start = self.peek_span().unwrap_or_else(Span::dummy);
            self.parse_single_line_block(start)
        }
    }

    fn parse_trait(&mut self) -> Option<Stmt> {
        let vis = self.parse_visibility();
        let kw_span = self.expect_with_span(&TokenKind::KwTrait)?;
        let (name, nspan) = self.expect_ident()?;
        let type_params = self.parse_generic_params();
        self.expect(&TokenKind::Colon)?;
        self.consume_if(&TokenKind::Newline);
        self.expect(&TokenKind::Indent)?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(&TokenKind::Newline) {
                continue;
            }
            match self.parse_fn() {
                Some(Stmt::FnDef(f)) => methods.push(f),
                Some(_) => {}
                None => {
                    self.next();
                }
            }
        }
        self.expect(&TokenKind::Dedent)?;
        let end_span = self.peek_span().unwrap_or(nspan);
        Some(Stmt::Trait(TraitDef {
            vis,
            name: Ident { name, span: nspan },
            type_params,
            methods,
            span: kw_span.join(end_span).unwrap_or(kw_span),
        }))
    }

    fn parse_impl(&mut self) -> Option<Stmt> {
        let _vis = self.parse_visibility(); // impl の公開可視性は現状未使用
        let kw_span = self.expect_with_span(&TokenKind::KwImpl)?;
        let type_params = self.parse_generic_params();

        let first_ty = self.parse_type_expr()?;

        let (trait_name, target_ty) = if self.consume_if(&TokenKind::KwFor) {
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

        self.expect(&TokenKind::Colon)?;
        self.consume_if(&TokenKind::Newline);
        self.expect(&TokenKind::Indent)?;
        let mut methods = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(&TokenKind::Newline) {
                continue;
            }
            match self.parse_fn() {
                Some(Stmt::FnDef(f)) => methods.push(f),
                Some(_) => {}
                None => {
                    self.next();
                }
            }
        }
        self.expect(&TokenKind::Dedent)?;
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

        while !self.is_end(&TokenEnd::Line) || self.is_pipe_continuation() {
            if self.is_pipe_continuation() && self.check(&TokenKind::Newline) {
                self.next(); // skip newline to continue expression
            }
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
                // line is an error: only one statement per line is allowed,
                // UNLESS we are in a block that allows multiple statements per line.
                // But parse_prefix_expr itself should stop here.
                _ if in_trailing_semis => {
                    break;
                }
                TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof => break,
                TokenKind::Colon => {
                    let colon_span = self.next().unwrap().span;
                    let block = self.parse_block_after_colon()?;
                    let span = colon_span.join(block.span).unwrap_or(colon_span);

                    if let Some(last) = items.last() {
                        if let Some((params, params_span)) = Self::try_extract_lambda_params(last) {
                            items.pop();
                            items.push(self.build_lambda_block(params, params_span, block));
                            break;
                        }
                    }

                    if let Some(marker) = Self::tail_block_marker(&items) {
                        if marker == "block" {
                            items.pop();
                        }
                        items.push(PrefixItem::Block(block, span));
                    } else if items
                        .iter()
                        .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                    {
                        let expected = if Self::if_layout_needs_cond(&items) {
                            3
                        } else {
                            2
                        };
                        if expected == 2 {
                            Self::drop_if_optional_cond_marker(&mut items);
                        }
                        match self.extract_if_layout_exprs(block.clone(), expected, colon_span) {
                            Ok(mut args) => {
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
                                        let wrapped = Block {
                                            items: vec![Stmt::Expr(a.clone())],
                                            span: a.span,
                                        };
                                        items.push(PrefixItem::Block(wrapped, a.span));
                                    }
                                }
                            }
                            Err(diag) => {
                                if diag.message == "missing expression(s) in if-layout block" {
                                    match self.extract_if_layout_exprs_lenient(
                                        block.clone(),
                                        expected,
                                        colon_span,
                                    ) {
                                        Ok(mut args) => {
                                            for mut a in args.drain(..) {
                                                if a.items.len() == 1 {
                                                    let only = a.items.remove(0);
                                                    items.push(only);
                                                } else {
                                                    let wrapped = Block {
                                                        items: vec![Stmt::Expr(a.clone())],
                                                        span: a.span,
                                                    };
                                                    items.push(PrefixItem::Block(wrapped, a.span));
                                                }
                                            }
                                        }
                                        Err(diag2) => {
                                            // `if <cond>:` の直後は then 側だけ先に読める。
                                            // else は同インデントの後続行で巻き上げ合成されるため、
                                            // この段階での missing-expression は確定エラーにしない。
                                            if !(expected == 2
                                                && diag2.message
                                                    == "missing expression(s) in if-layout block")
                                            {
                                                self.diagnostics.push(diag2);
                                            }
                                            items.push(PrefixItem::Block(block, span));
                                        }
                                    }
                                } else {
                                    self.diagnostics.push(diag);
                                    items.push(PrefixItem::Block(block, span));
                                }
                            }
                        }
                    } else if items
                        .iter()
                        .any(|it| matches!(it, PrefixItem::Symbol(Symbol::While(_))))
                    {
                        let expected = if Self::while_layout_needs_cond(&items) {
                            2
                        } else {
                            1
                        };
                        match self.extract_while_layout_exprs(block.clone(), expected, colon_span) {
                            Ok(mut args) => {
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
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
                        match self.extract_arg_layout_exprs(block.clone(), colon_span) {
                            Ok(mut args) => {
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
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
                    self.expect(&TokenKind::RAngle)?;
                    let end = self.peek_span().unwrap_or(start);
                    let span = start.join(end).unwrap_or(start);
                    items.push(PrefixItem::TypeAnnotation(ty, span));
                }
                TokenKind::Minus => {
                    let minus_span = self.next().unwrap().span;
                    match self.peek_kind() {
                        Some(TokenKind::IntLiteral(_)) => {
                            let tok = self.next().unwrap();
                            let v = if let TokenKind::IntLiteral(v) = tok.kind { v } else { unreachable!() };
                            let combined = alloc::format!("-{}", v);
                            items.push(PrefixItem::Literal(Literal::Int(combined), minus_span.join(tok.span).unwrap_or(minus_span)));
                        }
                        Some(TokenKind::FloatLiteral(v)) => {
                            let tok = self.next().unwrap();
                            let combined = alloc::format!("-{}", v);
                            items.push(PrefixItem::Literal(Literal::Float(combined), minus_span.join(tok.span).unwrap_or(minus_span)));
                        }
                        _ => {
                            items.push(PrefixItem::Symbol(Symbol::Ident(
                                Ident {
                                    name: "-".to_string(),
                                    span: minus_span,
                                },
                                Vec::new(),
                                false,
                            )));
                        }
                    }
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
                    if self.consume_if(&TokenKind::RParen) {
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
                    let is_mut = self.consume_if(&TokenKind::KwMut);
                    let (name, span) = self.expect_ident()?;
                    self.consume_if(&TokenKind::Equals);
                    items.push(PrefixItem::Symbol(Symbol::Let {
                        name: Ident { name, span },
                        mutable: is_mut,
                    }));
                }
                TokenKind::KwSet => {
                    let set_tok = self.next().unwrap();
                    let (name, span) = self.expect_ident()?;
                    self.consume_if(&TokenKind::Equals);
                    items.push(PrefixItem::Symbol(Symbol::Set {
                        name: Ident { name, span: set_tok.span.join(span).unwrap_or(span) },
                    }));
                }
                TokenKind::DirIntrinsic => {
                    let kw_span = self.next().unwrap().span;
                    let (name, _name_span) = match self.peek_kind() {
                        Some(TokenKind::StringLiteral(_)) => {
                            let tok = self.next().unwrap();
                            if let TokenKind::StringLiteral(s) = tok.kind {
                                (s, tok.span)
                            } else { unreachable!() }
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
                    if self.check(&TokenKind::LAngle) {
                        self.consume_if(&TokenKind::LAngle);
                        loop {
                            if self.check(&TokenKind::RAngle) {
                                break;
                            }
                            type_args.push(self.parse_type_expr()?);
                            if !self.consume_if(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokenKind::RAngle)?;
                    }

                    let lp = if self.check(&TokenKind::LParen) {
                        self.next().unwrap().span
                    } else {
                        let sp = self.peek_span().unwrap_or(kw_span);
                        self.diagnostics.push(Diagnostic::error(
                            "expected '(' after intrinsic name/res",
                            sp,
                        ));
                        return None;
                    };

                    let (args, args_span, _) = if self.consume_if(&TokenKind::RParen) {
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
                TokenKind::KwBlock => {
                    let span = self.next().unwrap().span;
                    if self.consume_if(&TokenKind::Colon) {
                        let block = if self.check(&TokenKind::Newline) {
                            self.parse_block_after_colon()?
                        } else {
                            let err_span = self.peek_span().unwrap_or(span);
                            self.diagnostics.push(Diagnostic::error(
                                "block: requires newline after ':' (only whitespace/comment is allowed)",
                                err_span,
                            ));
                            self.parse_single_line_block(err_span)?
                        };
                        let bspan = block.span;
                        items.push(PrefixItem::Block(block, bspan));
                        break;
                    } else {
                        let block = self.parse_single_line_block(span)?;
                        let bspan = block.span;
                        items.push(PrefixItem::Block(block, bspan));
                        let continue_outer = match self.peek_kind() {
                            Some(TokenKind::KwBlock) => true,
                            Some(TokenKind::Ident(name)) if name == "else" => true,
                            _ => false,
                        };
                        if !continue_outer {
                            break;
                        }
                    }
                }
                TokenKind::KwMlstr => {
                    let span = self.next().unwrap().span;
                    if self.consume_if(&TokenKind::Colon) {
                        if let Some(content) = self.parse_mlstr_layout(span) {
                            let cspan = span.join(self.peek_span().unwrap_or(span)).unwrap_or(span);
                            items.push(PrefixItem::Literal(Literal::Str(content), cspan));
                        }
                    }
                    break;
                }
                TokenKind::KwTuple => {
                    let span = self.next().unwrap().span;
                    if self.consume_if(&TokenKind::Colon) {
                        let block = self.parse_block_after_colon()?;
                        let args = self.extract_arg_layout_exprs(block.clone(), span).ok()?;
                        items.push(PrefixItem::Tuple(
                            args,
                            span.join(block.span).unwrap_or(span),
                        ));
                    }
                    break;
                }
                TokenKind::KwMatch => {
                    let m = self.parse_match_expr()?;
                    let sp = m.span;
                    items.push(PrefixItem::Match(m, sp));
                    break;
                }
                TokenKind::At | TokenKind::Ident(_) => {
                    let ident_item = self.parse_ident_symbol_item(&items, true, true)?;
                    items.push(ident_item);
                }
                TokenKind::Ampersand => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::AddrOf(span)));
                }
                TokenKind::Star => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::Deref(span)));
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
            if self.consume_if(&TokenKind::Comma) {
                saw_comma = true;
                if self.check(&TokenKind::RParen) {
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
            let rp = if self.consume_if(&TokenKind::RParen) {
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

                    if let Some(last) = items.last() {
                        if let Some((params, params_span)) = Self::try_extract_lambda_params(last) {
                            items.pop();
                            items.push(self.build_lambda_block(params, params_span, block));
                            break;
                        }
                    }

                    if let Some(marker) = Self::tail_block_marker(&items) {
                        if marker == "block" {
                            items.pop();
                        }
                        items.push(PrefixItem::Block(block, span));
                    } else if items
                        .iter()
                        .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                    {
                        let expected = if Self::if_layout_needs_cond(&items) {
                            3
                        } else {
                            2
                        };
                        if expected == 2 {
                            Self::drop_if_optional_cond_marker(&mut items);
                        }
                        match self.extract_if_layout_exprs(block.clone(), expected, colon_span) {
                            Ok(mut args) => {
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
                                        let wrapped = Block {
                                            items: vec![Stmt::Expr(a.clone())],
                                            span: a.span,
                                        };
                                        items.push(PrefixItem::Block(wrapped, a.span));
                                    }
                                }
                            }
                            Err(diag) => {
                                if diag.message == "missing expression(s) in if-layout block" {
                                    match self.extract_if_layout_exprs_lenient(
                                        block.clone(),
                                        expected,
                                        colon_span,
                                    ) {
                                        Ok(mut args) => {
                                            for mut a in args.drain(..) {
                                                if a.items.len() == 1 {
                                                    let only = a.items.remove(0);
                                                    items.push(only);
                                                } else {
                                                    let wrapped = Block {
                                                        items: vec![Stmt::Expr(a.clone())],
                                                        span: a.span,
                                                    };
                                                    items.push(PrefixItem::Block(wrapped, a.span));
                                                }
                                            }
                                        }
                                        Err(diag2) => {
                                            if !(expected == 2
                                                && diag2.message
                                                    == "missing expression(s) in if-layout block")
                                            {
                                                self.diagnostics.push(diag2);
                                            }
                                            items.push(PrefixItem::Block(block, span));
                                        }
                                    }
                                } else {
                                    self.diagnostics.push(diag);
                                    items.push(PrefixItem::Block(block, span));
                                }
                            }
                        }
                    } else if items
                        .iter()
                        .any(|it| matches!(it, PrefixItem::Symbol(Symbol::While(_))))
                    {
                        let expected = if Self::while_layout_needs_cond(&items) {
                            2
                        } else {
                            1
                        };
                        match self.extract_while_layout_exprs(block.clone(), expected, colon_span) {
                            Ok(mut args) => {
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
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
                        match self.extract_arg_layout_exprs(block.clone(), colon_span) {
                            Ok(mut args) => {
                                for mut a in args.drain(..) {
                                    if a.items.len() == 1 {
                                        let only = a.items.remove(0);
                                        items.push(only);
                                    } else {
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
                    self.expect(&TokenKind::RAngle)?;
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
                    if self.consume_if(&TokenKind::RParen) {
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
                    let is_mut = self.consume_if(&TokenKind::KwMut);
                    let (name, span) = self.expect_ident()?;
                    self.consume_if(&TokenKind::Equals);
                    items.push(PrefixItem::Symbol(Symbol::Let {
                        name: Ident { name, span },
                        mutable: is_mut,
                    }));
                }
                TokenKind::KwSet => {
                    let _ = self.next();
                    let (mut name, mut span) = self.expect_ident()?;
                    // Handle field access in set: set v.len = 10
                    while self.consume_if(&TokenKind::Dot) {
                        if let Some((field, fspan)) = self.expect_ident() {
                            name.push('.');
                            name.push_str(&field);
                            span = span.join(fspan).unwrap_or(span);
                        } else {
                            break;
                        }
                    }
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
                TokenKind::At | TokenKind::Ident(_) => {
                    let ident_item = self.parse_ident_symbol_item(&items, true, true)?;
                    items.push(ident_item);
                }
                TokenKind::Ampersand => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::AddrOf(span)));
                }
                TokenKind::Star => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::Deref(span)));
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
        let colon_span = if self.check(&TokenKind::Colon) {
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
                TokenKind::LAngle => {
                    let start = self.next().unwrap().span;
                    let ty = self.parse_type_expr()?;
                    self.expect(&TokenKind::RAngle)?;
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
                    if self.consume_if(&TokenKind::RParen) {
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
                TokenKind::At | TokenKind::Ident(_) => {
                    // match scrutinee 側でも `E::A` / `obj.field` を同じ規則で扱う
                    let ident_item = self.parse_ident_symbol_item(&items, true, true)?;
                    items.push(ident_item);
                }
                TokenKind::KwLet => {
                    let _ = self.next();
                    let is_mut = self.consume_if(&TokenKind::KwMut);
                    let (name, span) = self.expect_ident()?;
                    items.push(PrefixItem::Symbol(Symbol::Let {
                        name: Ident { name, span },
                        mutable: is_mut,
                    }));
                }
                TokenKind::KwSet => {
                    let _ = self.next();
                    let (mut name, mut span) = self.expect_ident()?;
                    // Handle field access in set: set v.len = 10
                    while self.consume_if(&TokenKind::Dot) {
                        if let Some((field, fspan)) = self.expect_ident() {
                            name.push('.');
                            name.push_str(&field);
                            span = span.join(fspan).unwrap_or(span);
                        } else {
                            break;
                        }
                    }
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
                TokenKind::Ampersand => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::AddrOf(span)));
                }
                TokenKind::Star => {
                    let span = self.next().unwrap().span;
                    items.push(PrefixItem::Symbol(Symbol::Deref(span)));
                }
                _ => {
                    let span = self.peek_span().unwrap_or_else(Span::dummy);
                    self.diagnostics.push(Diagnostic::error(
                        "unexpected token in expression",
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
                PrefixItem::Symbol(crate::ast::Symbol::Ident(ident, _, _))
                    if ident.name == "then" || ident.name == "else"
            )
        });
        let has_do = items.iter().any(|item| {
            matches!(
                item,
                PrefixItem::Symbol(crate::ast::Symbol::Ident(ident, _, _)) if ident.name == "do"
            )
        });
        let has_while = items
            .iter()
            .any(|item| matches!(item, PrefixItem::Symbol(Symbol::While(_))));
        let mut i = 0;
        while i < items.len() {
            let remove = match &items[i] {
                PrefixItem::Symbol(crate::ast::Symbol::Ident(ident, _, _)) => {
                    let n = ident.name.as_str();
                    // remove only if not the first item (i != 0)
                    ((n == "then" || n == "else") && i != 0)
                        || (n == "cond" && i != 0 && (has_then_else || (has_do && has_while)))
                        || (n == "do" && i != 0 && has_while)
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

    /// Peek at the role marker without modifying the expression. O(1) operation.
    fn peek_role_from_expr(expr: &PrefixExpr) -> Option<IfRole> {
        // Check first item only - markers always appear at the start
        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = expr.items.first() {
            match id.name.as_str() {
                "cond" => return Some(IfRole::Cond),
                "then" => return Some(IfRole::Then),
                "else" => return Some(IfRole::Else),
                _ => {}
            }
        }
        // Check if marker is inside a Block (for block-style markers like `else:`)
        for item in &expr.items {
            if let PrefixItem::Block(block, _) = item {
                let inner_opt = match block.items.first() {
                    Some(Stmt::Expr(inner)) | Some(Stmt::ExprSemi(inner, _)) => Some(inner),
                    _ => None,
                };
                if let Some(inner) = inner_opt {
                    if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = inner.items.first() {
                        match id.name.as_str() {
                            "cond" => return Some(IfRole::Cond),
                            "then" => return Some(IfRole::Then),
                            "else" => return Some(IfRole::Else),
                            _ => {}
                        }
                    }
                }
            }
        }
        None
    }

    fn take_role_from_expr(expr: &mut PrefixExpr) -> Option<IfRole> {
        // Direct marker case: first item is the marker identifier
        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = expr.items.first() {
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
                    let inner_opt = match stmt {
                        Stmt::Expr(inner) | Stmt::ExprSemi(inner, _) => Some(inner),
                        _ => None,
                    };
                    if let Some(inner) = inner_opt {
                        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = inner.items.first() {
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
                            // Use std::mem::take to avoid clone
                            let inner_expr = core::mem::take(inner);
                            let wrapped_block = Block {
                                items: vec![Stmt::Expr(inner_expr)],
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

    fn take_while_role_from_expr(expr: &mut PrefixExpr) -> Option<WhileRole> {
        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = expr.items.first() {
            if id.name == "cond" {
                expr.items.remove(0);
                return Some(WhileRole::Cond);
            } else if id.name == "do" {
                expr.items.remove(0);
                return Some(WhileRole::Do);
            }
        }
        for i in 0..expr.items.len() {
            if let PrefixItem::Block(block, block_span) = &mut expr.items[i] {
                if let Some(stmt) = block.items.first_mut() {
                    let inner_opt = match stmt {
                        Stmt::Expr(inner) | Stmt::ExprSemi(inner, _) => Some(inner),
                        _ => None,
                    };
                    if let Some(inner) = inner_opt {
                        if let Some(PrefixItem::Symbol(Symbol::Ident(id, _, _))) = inner.items.first() {
                            let role = if id.name == "cond" {
                                WhileRole::Cond
                            } else if id.name == "do" {
                                WhileRole::Do
                            } else {
                                continue;
                            };
                            inner.items.remove(0);
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

    fn block_marker_name<'a>(item: &'a PrefixItem) -> Option<&'a str> {
        if let PrefixItem::Symbol(Symbol::Ident(id, _, _)) = item {
            match id.name.as_str() {
                "block" | "cond" | "then" | "else" | "do" => Some(id.name.as_str()),
                _ => None,
            }
        } else {
            None
        }
    }

    fn sole_non_type_marker(items: &[PrefixItem]) -> Option<&str> {
        let mut marker: Option<&PrefixItem> = None;
        for item in items {
            if matches!(item, PrefixItem::TypeAnnotation(_, _)) {
                continue;
            }
            if marker.is_some() {
                return None;
            }
            marker = Some(item);
        }
        marker.and_then(Self::block_marker_name)
    }

    fn tail_block_marker(items: &[PrefixItem]) -> Option<&str> {
        let mut last_non_type: Option<&PrefixItem> = None;
        for item in items.iter().rev() {
            if matches!(item, PrefixItem::TypeAnnotation(_, _)) {
                continue;
            }
            last_non_type = Some(item);
            break;
        }
        if let Some(item) = last_non_type {
            if let Some("block") = Self::block_marker_name(item) {
                return Some("block");
            }
        }
        Self::sole_non_type_marker(items)
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

    fn drop_if_optional_cond_marker(items: &mut Vec<PrefixItem>) {
        let if_pos = items
            .iter()
            .position(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))));
        let Some(if_pos) = if_pos else {
            return;
        };
        let marker_pos = if_pos + 1;
        if marker_pos >= items.len() {
            return;
        }
        if matches!(
            &items[marker_pos],
            PrefixItem::Symbol(Symbol::Ident(id, _, _)) if id.name == "cond"
        ) {
            items.remove(marker_pos);
        }
    }

    fn while_layout_needs_cond(items: &[PrefixItem]) -> bool {
        let mut tail: Vec<&PrefixItem> = items.iter().collect();
        while let Some(PrefixItem::TypeAnnotation(_, _)) = tail.last().copied() {
            tail.pop();
        }
        match tail.as_slice() {
            [.., PrefixItem::Symbol(Symbol::While(_))] => true,
            _ => false,
        }
    }

    fn extract_if_layout_exprs(
        &mut self,
        block: Block,
        expected: usize,
        header_span: Span,
    ) -> Result<Vec<PrefixExpr>, Diagnostic> {
        let mut branches: Vec<(Option<IfRole>, Vec<Stmt>)> = Vec::new();
        let mut current_branch: Vec<Stmt> = Vec::new();
        let mut current_role: Option<IfRole> = None;

        for stmt in block.items {
            let mut is_marker = false;
            if let Stmt::Expr(e) | Stmt::ExprSemi(e, _) = &stmt {
                let mut e_copy = e.clone();
                let role_opt = Self::take_role_from_expr(&mut e_copy);
                if let Some(role) = role_opt {
                    // It's a marker! Finish previous branch if not empty.
                    if !current_branch.is_empty() || current_role.is_some() {
                        branches.push((current_role, current_branch));
                    }
                    // Start a new branch for this role.
                    current_role = Some(role);
                    current_branch = Vec::new();
                    // If there was something else on the marker line, treat that as
                    // the complete expression for this branch (push immediately).
                    if !e_copy.items.is_empty() {
                        current_branch.push(Stmt::Expr(e_copy));
                        branches.push((current_role, current_branch));
                        // reset for following positional branches
                        current_role = None;
                        current_branch = Vec::new();
                    }
                    is_marker = true;
                }
            }
            if !is_marker {
                current_branch.push(stmt);
            }
        }
        if !current_branch.is_empty() || current_role.is_some() {
            branches.push((current_role, current_branch));
        }

        // Expand branches so that positional groups with multiple statements are
        // split into separate positional branches (one stmt -> one branch).
        let mut expanded: Vec<(Option<IfRole>, Vec<Stmt>)> = Vec::new();
        for (role, stmts) in branches {
            if role.is_some() {
                expanded.push((role, stmts));
            } else {
                for s in stmts {
                    expanded.push((None, vec![s]));
                }
            }
        }

        // Assign to slots: positional first, then fill by role.
        // If NO markers are used, we expect exactly `expected` branches and they map positionally.
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
        let mut last_role_idx: Option<usize> = None;
        for (role, stmts) in expanded {
            // Convert statements into a single PrefixExpr (wrapped in Block if multiple)
            let expr = if stmts.len() == 1 {
                match stmts.into_iter().next().unwrap() {
                    Stmt::Expr(e) | Stmt::ExprSemi(e, _) => e,
                    _ => unreachable!(),
                }
            } else {
                let span = if let (Some(f), Some(l)) = (stmts.first(), stmts.last()) {
                    self.stmt_span(f).join(self.stmt_span(l)).unwrap_or_else(|| self.stmt_span(f))
                } else {
                    header_span
                };
                PrefixExpr {
                    items: vec![PrefixItem::Block(Block { items: stmts, span }, span)],
                    trailing_semis: 0,
                    trailing_semi_span: None,
                    span,
                }
            };

            if let Some(r) = role {
                let idx = match role_to_index(r) {
                    Some(i) => i,
                    None => {
                        return Err(Diagnostic::error("invalid marker in this if-layout form", expr.span));
                    }
                };
                if let Some(prev_idx) = last_role_idx {
                    if idx < prev_idx {
                        return Err(Diagnostic::error("invalid marker order in if-layout block", expr.span));
                    }
                }
                if slots[idx].is_some() {
                    return Err(Diagnostic::error("duplicate marker in if-layout block", expr.span));
                }
                slots[idx] = Some(expr);
                last_role_idx = Some(idx);
            } else {
                while next_unfilled < expected && slots[next_unfilled].is_some() {
                    next_unfilled += 1;
                }
                if next_unfilled >= expected {
                    return Err(Diagnostic::error("too many expressions in if-layout block", expr.span));
                }
                slots[next_unfilled] = Some(expr);
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

    fn extract_if_layout_exprs_lenient(
        &mut self,
        block: Block,
        expected: usize,
        header_span: Span,
    ) -> Result<Vec<PrefixExpr>, Diagnostic> {
        let mut entries: Vec<(Option<IfRole>, PrefixExpr)> = Vec::new();
        let mut pending_role: Option<IfRole> = None;

        for stmt in block.items {
            let mut expr = match stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => e,
                other => {
                    return Err(Diagnostic::error(
                        "only expressions are allowed in if-layout block",
                        self.stmt_span(&other),
                    ));
                }
            };

            if let Some(role) = Self::take_role_from_expr(&mut expr) {
                if expr.items.is_empty() {
                    if pending_role.is_some() {
                        return Err(Diagnostic::error("duplicate marker in if-layout block", expr.span));
                    }
                    pending_role = Some(role);
                } else {
                    entries.push((Some(role), expr));
                }
                continue;
            }

            let role = pending_role.take();
            entries.push((role, expr));
        }

        if pending_role.is_some() {
            return Err(Diagnostic::error(
                "missing expression(s) in if-layout block",
                header_span,
            ));
        }

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
        let mut last_role_idx: Option<usize> = None;
        for (role, expr) in entries {
            if let Some(r) = role {
                let idx = match role_to_index(r) {
                    Some(i) => i,
                    None => {
                        return Err(Diagnostic::error("invalid marker in this if-layout form", expr.span));
                    }
                };
                if let Some(prev_idx) = last_role_idx {
                    if idx < prev_idx {
                        return Err(Diagnostic::error("invalid marker order in if-layout block", expr.span));
                    }
                }
                if slots[idx].is_some() {
                    return Err(Diagnostic::error("duplicate marker in if-layout block", expr.span));
                }
                slots[idx] = Some(expr);
                last_role_idx = Some(idx);
            } else {
                while next_unfilled < expected && slots[next_unfilled].is_some() {
                    next_unfilled += 1;
                }
                if next_unfilled >= expected {
                    return Err(Diagnostic::error("too many expressions in if-layout block", expr.span));
                }
                slots[next_unfilled] = Some(expr);
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

    fn extract_while_layout_exprs(
        &mut self,
        block: Block,
        expected: usize,
        header_span: Span,
    ) -> Result<Vec<PrefixExpr>, Diagnostic> {
        let mut branches: Vec<(Option<WhileRole>, Vec<Stmt>)> = Vec::new();
        let mut current_branch: Vec<Stmt> = Vec::new();
        let mut current_role: Option<WhileRole> = None;

        for stmt in block.items {
            let mut is_marker = false;
            if let Stmt::Expr(e) | Stmt::ExprSemi(e, _) = &stmt {
                let mut e_copy = e.clone();
                let role_opt = Self::take_while_role_from_expr(&mut e_copy);
                if let Some(role) = role_opt {
                    if !current_branch.is_empty() || current_role.is_some() {
                        branches.push((current_role, current_branch));
                    }
                    current_role = Some(role);
                    current_branch = Vec::new();
                    if !e_copy.items.is_empty() {
                        current_branch.push(Stmt::Expr(e_copy));
                        branches.push((current_role, current_branch));
                        current_role = None;
                        current_branch = Vec::new();
                    }
                    is_marker = true;
                }
            }
            if !is_marker {
                current_branch.push(stmt);
            }
        }
        if !current_branch.is_empty() || current_role.is_some() {
            branches.push((current_role, current_branch));
        }

        let mut expanded: Vec<(Option<WhileRole>, Vec<Stmt>)> = Vec::new();
        for (role, stmts) in branches {
            if role.is_some() {
                expanded.push((role, stmts));
            } else {
                for s in stmts {
                    expanded.push((None, vec![s]));
                }
            }
        }

        let mut slots: Vec<Option<PrefixExpr>> = vec![None; expected];

        let role_to_index = |role: WhileRole| -> Option<usize> {
            match (expected, role) {
                (2, WhileRole::Cond) => Some(0),
                (2, WhileRole::Do) => Some(1),
                (1, WhileRole::Do) => Some(0),
                _ => None,
            }
        };

        let mut next_unfilled = 0usize;
        for (role, stmts) in expanded {
            let expr = if stmts.len() == 1 {
                match stmts.into_iter().next().unwrap() {
                    Stmt::Expr(e) | Stmt::ExprSemi(e, _) => e,
                    _ => unreachable!(),
                }
            } else {
                let span = if let (Some(f), Some(l)) = (stmts.first(), stmts.last()) {
                    self.stmt_span(f).join(self.stmt_span(l)).unwrap_or_else(|| self.stmt_span(f))
                } else {
                    header_span
                };
                PrefixExpr {
                    items: vec![PrefixItem::Block(Block { items: stmts, span }, span)],
                    trailing_semis: 0,
                    trailing_semi_span: None,
                    span,
                }
            };

            if let Some(r) = role {
                let idx = match role_to_index(r) {
                    Some(i) => i,
                    None => {
                        return Err(Diagnostic::error(
                            "invalid marker in this while-layout form",
                            expr.span,
                        ));
                    }
                };
                if slots[idx].is_some() {
                    return Err(Diagnostic::error("duplicate marker in while-layout block", expr.span));
                }
                slots[idx] = Some(expr);
            } else {
                while next_unfilled < expected && slots[next_unfilled].is_some() {
                    next_unfilled += 1;
                }
                if next_unfilled >= expected {
                    return Err(Diagnostic::error(
                        "too many expressions in while-layout block",
                        expr.span,
                    ));
                }
                slots[next_unfilled] = Some(expr);
                next_unfilled += 1;
            }
        }

        if slots.iter().any(|s| s.is_none()) {
            return Err(Diagnostic::error(
                "missing expression(s) in while-layout block",
                header_span,
            ));
        }

        Ok(slots.into_iter().map(|s| s.unwrap()).collect())
    }

    fn extract_arg_layout_exprs(
        &mut self,
        block: Block,
        header_span: Span,
    ) -> Result<Vec<PrefixExpr>, Diagnostic> {
        let mut exprs: Vec<PrefixExpr> = Vec::new();
        for stmt in block.items {
            match stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => exprs.push(e),
                other => {
                    let sp = self.stmt_span(&other);
                    return Err(Diagnostic::error(
                        "only expressions are allowed in argument layout",
                        sp,
                    ));
                }
            }
        }
        if exprs.is_empty() {
            return Err(Diagnostic::error(
                "argument layout block must contain expressions",
                header_span,
            ));
        }
        Ok(exprs)
    }

    fn parse_match_arms(&mut self) -> Option<Vec<MatchArm>> {
        if !self.consume_if(&TokenKind::Newline) {
            let sp = self.peek_span().unwrap_or_else(Span::dummy);
            self.diagnostics
                .push(Diagnostic::error("expected newline after match ':'", sp));
        }
        self.expect(&TokenKind::Indent)?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.is_eof() {
            if self.consume_if(&TokenKind::Newline) {
                continue;
            }
            let (vname_tok, vspan_tok) = self.expect_ident()?;
            let mut vname = vname_tok;
            let mut vspan = vspan_tok;
            while self.consume_if(&TokenKind::PathSep) {
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
            self.expect(&TokenKind::Colon)?;
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
        self.expect(&TokenKind::Dedent)?;
        Some(arms)
    }

    fn parse_generic_params(&mut self) -> Vec<TypeParam> {
        let mut params = Vec::new();
        if self.consume_if(&TokenKind::LAngle) {
            loop {
                let mut has_dot = false;
                if self.consume_if(&TokenKind::Dot) {
                    has_dot = true;
                }
                if let Some((name, span)) = self.expect_ident() {
                    if !has_dot {
                        self.diagnostics.push(Diagnostic::error(
                            "type parameter must be written as .T",
                            span,
                        ));
                    }
                    let mut bounds = Vec::new();
                    if self.consume_if(&TokenKind::Colon) {
                        if let Some((bound, _bspan)) = self.parse_path_ident() {
                            bounds.push(bound);
                        } else {
                            let sp = self.peek_span().unwrap_or(span);
                            self.diagnostics
                                .push(Diagnostic::error("expected trait name after ':'", sp));
                        }
                        while self.consume_if(&TokenKind::Ampersand) {
                            if let Some((bound, _bspan)) = self.parse_path_ident() {
                                bounds.push(bound);
                            } else {
                                let sp = self.peek_span().unwrap_or(span);
                                self.diagnostics
                                    .push(Diagnostic::error("expected trait name after '&'", sp));
                                break;
                            }
                        }
                    }
                    params.push(TypeParam {
                        name: Ident { name, span },
                        bounds,
                    });
                } else {
                    break;
                }
                if !self.consume_if(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RAngle);
        }
        params
    }

    fn parse_path_ident(&mut self) -> Option<(String, Span)> {
        let (mut name, mut span) = self.expect_ident()?;
        while self.consume_if(&TokenKind::PathSep) {
            if let Some((seg, sspan)) = self.expect_ident() {
                name.push_str("::");
                name.push_str(&seg);
                span = span.join(sspan).unwrap_or(span);
            } else {
                break;
            }
        }
        Some((name, span))
    }

    fn parse_type_expr(&mut self) -> Option<TypeExpr> {
        self.parse_type_expr_internal()
    }


    fn parse_type_expr_internal(&mut self) -> Option<TypeExpr> {
        match self.peek_kind()? {
            TokenKind::UnitLiteral => {
                self.next();
                Some(TypeExpr::Unit)
            }
            TokenKind::Ident(name) => {
                let _ = self.next();
                let mut ty = match name.as_str() {
                    "i32" => TypeExpr::I32,
                    "u8" => TypeExpr::U8,
                    "f32" => TypeExpr::F32,
                    "bool" => TypeExpr::Bool,
                    "never" => TypeExpr::Never,
                    "str" => TypeExpr::Str,
                    "Box" => {
                        self.expect(&TokenKind::LAngle)?;
                        let inner = self.parse_type_expr()?;
                        self.expect(&TokenKind::RAngle)?;
                        return Some(TypeExpr::Boxed(Box::new(inner)));
                    }
                    _ => TypeExpr::Named(name.clone()),
                };

                if self.consume_if(&TokenKind::LAngle) {
                    let mut args = Vec::new();
                    loop {
                        args.push(self.parse_type_expr()?);
                        if self.consume_if(&TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }
                    self.expect(&TokenKind::RAngle)?;
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
                if self.check(&TokenKind::RParen) {
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
                        if self.consume_if(&TokenKind::Comma) {
                            saw_comma = true;
                            continue;
                        }
                        break;
                    }
                    self.expect(&TokenKind::RParen)?;
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
                let is_mut = self.consume_if(&TokenKind::KwMut);
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

    fn parse_ident_symbol_item(
        &mut self,
        current_items: &[PrefixItem],
        allow_path_sep: bool,
        allow_dot: bool,
    ) -> Option<PrefixItem> {
        let at_span = if self.check(&TokenKind::At) {
            Some(self.next().unwrap().span)
        } else {
            None
        };
        let tok = self.next()?;
        let name = if let TokenKind::Ident(n) = tok.kind {
            n
        } else {
            let sp = at_span.unwrap_or(tok.span);
            self.diagnostics
                .push(Diagnostic::error("expected identifier after '@'", sp));
            return None;
        };

        let mut full = name;
        let mut end_span = tok.span;

        let mut type_args = Vec::new();
        loop {
            if allow_path_sep && self.check(&TokenKind::PathSep) {
                self.next();
                if let Some(TokenKind::Ident(_)) = self.peek_kind() {
                    let tok2 = self.next().unwrap();
                    let n2 = if let TokenKind::Ident(n) = tok2.kind {
                        n
                    } else {
                        unreachable!()
                    };
                    full.push_str("::");
                    full.push_str(&n2);
                    end_span = end_span.join(tok2.span).unwrap_or(tok2.span);
                    continue;
                }
                break;
            }

            if allow_dot && self.check(&TokenKind::Dot) {
                self.next();
                if let Some((field, fspan)) = self.expect_ident() {
                    full.push('.');
                    full.push_str(&field);
                    end_span = end_span.join(fspan).unwrap_or(end_span);
                    continue;
                }
                break;
            }

            // `ident<...>` 形式のみを型引数として解釈し、曖昧な場合は巻き戻す。
            if self.check(&TokenKind::LAngle) {
                let next_span = self.peek_span().unwrap_or(end_span);
                if next_span.file_id == end_span.file_id && next_span.start == end_span.end {
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
                        if self.consume_if(&TokenKind::Comma) {
                            continue;
                        }
                        break;
                    }

                    if ok && self.consume_if(&TokenKind::RAngle) {
                        type_args.extend(temp_args);
                        continue;
                    } else {
                        self.pos = saved;
                        self.diagnostics.truncate(saved_diags);
                    }
                }
            }
            break;
        }

        if Self::is_reserved_layout_word(&full)
            && !Self::is_allowed_layout_marker_usage(current_items, &full)
        {
            self.diagnostics.push(Diagnostic::error(
                alloc::format!("'{}' is a reserved keyword and cannot be used as an identifier", full),
                end_span,
            ));
        }

        Some(PrefixItem::Symbol(Symbol::Ident(
            Ident {
                name: full,
                span: end_span,
            },
            type_args,
            at_span.is_some(),
        )))
    }

    fn is_reserved_layout_word(name: &str) -> bool {
        matches!(name, "cond" | "then" | "else" | "do")
    }

    fn is_allowed_layout_marker_usage(current_items: &[PrefixItem], name: &str) -> bool {
        if current_items.is_empty() {
            return matches!(name, "cond" | "then" | "else" | "do");
        }
        let has_if = current_items
            .iter()
            .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))));
        if has_if && matches!(name, "cond" | "then" | "else") {
            return true;
        }
        let has_while = current_items
            .iter()
            .any(|it| matches!(it, PrefixItem::Symbol(Symbol::While(_))));
        has_while && matches!(name, "cond" | "do")
    }

    fn expect(&mut self, kind: &TokenKind) -> Option<()> {
        if self.consume_if(kind) {
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
        match self.peek_kind() {
            Some(TokenKind::Ident(name)) => {
                let tok = self.next().unwrap();
                if let TokenKind::Ident(n) = tok.kind {
                    if Self::is_reserved_layout_word(&n) {
                        self.diagnostics.push(Diagnostic::error(
                            alloc::format!(
                                "'{}' is a reserved keyword and cannot be used as an identifier",
                                n
                            ),
                            tok.span,
                        ));
                        return None;
                    }
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

    fn consume_if(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.next();
            true
        } else {
            false
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        matches!(self.peek_kind(), Some(k) if token_kind_eq(&k, kind))
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
        self.peek_kind().map_or(true, |k| matches!(k, TokenKind::Eof))
    }

    fn is_end(&self, end: &TokenEnd) -> bool {
        match end {
            TokenEnd::Eof => self.is_eof(),
            TokenEnd::Dedent => {
                self.peek_kind()
                    .map_or(true, |k| matches!(k, TokenKind::Dedent | TokenKind::Eof))
            }
            TokenEnd::Line => {
                if self.is_pipe_continuation() {
                    false
                } else {
                    self.peek_kind().map_or(true, |k| {
                        matches!(
                            k,
                            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
                        )
                    })
                }
            }
        }
    }

    fn is_pipe_continuation(&self) -> bool {
        if self.peek_kind() == Some(TokenKind::Newline) {
            if let Some(tok) = self.tokens.get(self.pos + 1) {
                return matches!(tok.kind, TokenKind::Pipe);
            }
        }
        false
    }

    fn parse_single_line_block(&mut self, span: Span) -> Option<Block> {
        let mut items = Vec::new();
        while !self.is_end(&TokenEnd::Line) {
            while self.consume_if(&TokenKind::Semicolon) {} // Skip empty statements/leading semicolons
            if self.is_end(&TokenEnd::Line) {
                break;
            }

            if matches!(
                self.peek_kind(),
                Some(
                    TokenKind::IntLiteral(_)
                        | TokenKind::FloatLiteral(_)
                        | TokenKind::BoolLiteral(_)
                        | TokenKind::UnitLiteral
                        | TokenKind::StringLiteral(_)
                )
            ) {
                let next_kind = self.peek_kind_at(1);
                let should_stop_after_literal = matches!(
                    next_kind,
                    Some(TokenKind::KwBlock)
                        | Some(TokenKind::KwIf)
                        | Some(TokenKind::KwWhile)
                        | Some(TokenKind::KwMatch)
                        | Some(TokenKind::At)
                        | Some(TokenKind::Ident(_))
                        | Some(TokenKind::LParen)
                );
                if should_stop_after_literal {
                    let tok = self.next().unwrap();
                    let lit = match tok.kind {
                        TokenKind::IntLiteral(v) => Literal::Int(v),
                        TokenKind::FloatLiteral(v) => Literal::Float(v),
                        TokenKind::BoolLiteral(b) => Literal::Bool(b),
                        TokenKind::StringLiteral(s) => Literal::Str(s),
                        TokenKind::UnitLiteral => Literal::Unit,
                        _ => unreachable!(),
                    };
                    items.push(Stmt::Expr(PrefixExpr {
                        items: vec![PrefixItem::Literal(lit, tok.span)],
                        trailing_semis: 0,
                        trailing_semi_span: None,
                        span: tok.span,
                    }));
                    break;
                }
            }

            if let Some(expr) = self.parse_prefix_expr() {
                if expr.trailing_semis > 0 {
                    items.push(Stmt::ExprSemi(expr.clone(), expr.trailing_semi_span));
                } else {
                    items.push(Stmt::Expr(expr));
                }
            } else {
                break;
            }
        }
        let end_span = self.peek_span().unwrap_or(span);
        Some(Block {
            items,
            span: span.join(end_span).unwrap_or(span),
        })
    }

    fn parse_mlstr_layout(&mut self, _span: Span) -> Option<String> {
        let mut text = String::new();
        let mut first = true;
        
        // Skip optional newline before block
        while self.consume_if(&TokenKind::Newline) {}
        
        self.expect(&TokenKind::Indent)?;
        
        while !self.is_end(&TokenEnd::Dedent) {
            let next_tok = self.next();
            match next_tok {
                Some(Token {
                    kind: TokenKind::MlstrLine(line),
                    ..
                }) => {
                    if !first {
                        text.push('\n');
                    }
                    text.push_str(&line);
                    first = false;
                    self.consume_if(&TokenKind::Newline);
                }
                Some(Token {
                    kind: TokenKind::Newline,
                    ..
                }) => {
                    // empty line or extra newline, ignore
                }
                Some(Token {
                    kind: TokenKind::Dedent,
                    ..
                }) | Some(Token {
                    kind: TokenKind::Eof,
                    ..
                }) => {
                    // should be handled by is_end, but just in case
                    break;
                }
                Some(_) => {
                    // unexpected token in mlstr layout, but let's just skip it
                    // and continue until we find a Dedent or EOF
                }
                None => break,
            }
        }
        self.consume_if(&TokenKind::Dedent);
        Some(text)
    }

    fn item_span(&self, item: &PrefixItem) -> Span {
        match item {
            PrefixItem::Literal(_, sp) => *sp,
            PrefixItem::Symbol(Symbol::Ident(id, _, _)) => id.span,
            PrefixItem::Symbol(Symbol::Let { name, .. }) => name.span,
            PrefixItem::Symbol(Symbol::Set { name }) => name.span,
            PrefixItem::Symbol(Symbol::If(sp)) => *sp,
            PrefixItem::Symbol(Symbol::While(sp)) => *sp,
            PrefixItem::Symbol(Symbol::AddrOf(sp)) => *sp,
            PrefixItem::Symbol(Symbol::Deref(sp)) => *sp,
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
                Directive::IfProfile { span, .. } => *span,
                Directive::IndentWidth { span, .. } => *span,
                Directive::Extern { span, .. } => *span,
                Directive::Include { span, .. } => *span,
                Directive::Prelude { span, .. } => *span,
                Directive::NoPrelude { span } => *span,
            },
            Stmt::FnDef(f) => f.name.span,
            Stmt::FnAlias(a) => a.name.span,
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
        (MlstrLine(_), MlstrLine(_)) => true,
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
        "u8" => Some(TypeExpr::U8),
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
