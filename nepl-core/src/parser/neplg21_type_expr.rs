use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::{Effect, TypeExpr};
use crate::diagnostic_codes::{DiagnosticCode, ParserDiagnosticCode};
use crate::lexer::TokenKind;
use crate::qualified_name::member_tail;
use crate::span::Span;

use super::push_type_arity_hint;
use super::Parser;

struct Neplg21ParsedType {
    ty: TypeExpr,
    grouped: bool,
}

impl Parser {
    pub(super) fn is_neplg21_prefix_type_expr_start(&self) -> bool {
        match self.peek_kind() {
            Some(TokenKind::KwFn) => true,
            Some(TokenKind::Ident(ref name))
                if name == "impure" && matches!(self.peek_kind_at(1), Some(TokenKind::KwFn)) =>
            {
                true
            }
            Some(TokenKind::Ident(ref name))
                if !matches!(self.peek_kind_at(1), Some(TokenKind::LAngle))
                    && self.neplg21_type_arity(name) > 0 =>
            {
                true
            }
            _ => false,
        }
    }

    pub(super) fn parse_neplg21_type_expr(&mut self) -> Option<TypeExpr> {
        let start_span = self.peek_span().unwrap_or_else(Span::dummy);
        let ty = self.parse_neplg21_type_expr_internal()?;
        let end_span = self.previous_span().unwrap_or(start_span);
        let span = start_span.join(end_span).unwrap_or(start_span);
        Some(ty.with_span(span))
    }

    pub(super) fn register_type_arity_hint(&mut self, name: &str, arity: usize) {
        push_type_arity_hint(&mut self.type_arity_hints, name.to_string(), arity);
    }

    fn parse_neplg21_type_expr_internal(&mut self) -> Option<TypeExpr> {
        Some(self.parse_neplg21_type_expr_marked()?.ty)
    }

    fn parse_neplg21_type_expr_marked(&mut self) -> Option<Neplg21ParsedType> {
        match self.peek_kind()? {
            TokenKind::UnitLiteral => {
                self.next();
                Some(Neplg21ParsedType {
                    ty: TypeExpr::Unit,
                    grouped: false,
                })
            }
            TokenKind::KwFn => {
                self.next();
                Some(Neplg21ParsedType {
                    ty: self.parse_neplg21_function_type(Effect::Pure)?,
                    grouped: false,
                })
            }
            TokenKind::Ident(ref name)
                if name == "impure" && matches!(self.peek_kind_at(1), Some(TokenKind::KwFn)) =>
            {
                self.next();
                self.next();
                Some(Neplg21ParsedType {
                    ty: self.parse_neplg21_function_type(Effect::Impure)?,
                    grouped: false,
                })
            }
            TokenKind::Ident(_) => {
                let (name, _) = self.parse_path_ident()?;
                let mut ty = match name.as_str() {
                    "i32" => TypeExpr::I32,
                    "u8" => TypeExpr::U8,
                    "f32" => TypeExpr::F32,
                    "bool" => TypeExpr::Bool,
                    "char" => TypeExpr::Char,
                    "never" => TypeExpr::Never,
                    "str" => TypeExpr::Str,
                    "Box" => {
                        let inner = self.parse_neplg21_type_expr()?;
                        return Some(Neplg21ParsedType {
                            ty: TypeExpr::Boxed(Box::new(inner)),
                            grouped: false,
                        });
                    }
                    _ => TypeExpr::Named(name.clone()),
                };

                let arity = self.neplg21_type_arity(&name);
                if arity > 0 {
                    let mut args = Vec::new();
                    for _ in 0..arity {
                        args.push(self.parse_neplg21_type_expr()?);
                    }
                    ty = TypeExpr::Apply(Box::new(ty), args);
                }
                Some(Neplg21ParsedType { ty, grouped: false })
            }
            TokenKind::Dot => {
                let _ = self.next();
                let ty = if let Some(TokenKind::Ident(name)) = self.peek_kind() {
                    let name = name.clone();
                    let _ = self.next();
                    TypeExpr::Label(Some(name))
                } else {
                    TypeExpr::Label(None)
                };
                Some(Neplg21ParsedType { ty, grouped: false })
            }
            TokenKind::LParen => {
                self.next();
                if self.consume_if(&TokenKind::RParen) {
                    Some(Neplg21ParsedType {
                        ty: TypeExpr::Unit,
                        grouped: false,
                    })
                } else {
                    let inner = self.parse_neplg21_type_expr()?;
                    self.expect(&TokenKind::RParen)?;
                    Some(Neplg21ParsedType {
                        ty: inner,
                        grouped: true,
                    })
                }
            }
            TokenKind::Ampersand => {
                let _ = self.next();
                let is_mut = self.consume_if(&TokenKind::KwMut);
                let inner = self.parse_neplg21_type_expr()?;
                Some(Neplg21ParsedType {
                    ty: TypeExpr::Reference(Box::new(inner), is_mut),
                    grouped: false,
                })
            }
            _ => {
                let span = self.peek_span().unwrap_or_else(Span::dummy);
                self.push_error_with_code(
                    DiagnosticCode::Parser(ParserDiagnosticCode::TypeExprInvalid),
                    "invalid NEPLg2.1 prefix type expression",
                    span,
                );
                self.next();
                None
            }
        }
    }

    fn parse_neplg21_function_type(&mut self, effect: Effect) -> Option<TypeExpr> {
        let params = if self.consume_if(&TokenKind::VoidMarker) {
            Vec::new()
        } else {
            vec![self.parse_neplg21_type_expr()?]
        };
        let result = self.parse_neplg21_type_expr_marked()?;
        Some(Self::combine_neplg21_function_type(
            params,
            result.ty,
            effect,
            !result.grouped,
        ))
    }

    fn combine_neplg21_function_type(
        params: Vec<TypeExpr>,
        result: TypeExpr,
        effect: Effect,
        flatten_nested: bool,
    ) -> TypeExpr {
        match result.into_unspanned() {
            TypeExpr::Function {
                params: mut nested_params,
                result: nested_result,
                effect: nested_effect,
            } if flatten_nested
                && effect == nested_effect
                && !params.is_empty()
                && !nested_params.is_empty() =>
            {
                let mut all_params = params;
                all_params.append(&mut nested_params);
                TypeExpr::Function {
                    params: all_params,
                    result: nested_result,
                    effect,
                }
            }
            other => TypeExpr::Function {
                params,
                result: Box::new(other),
                effect,
            },
        }
    }

    pub(super) fn neplg21_type_arity(&self, name: &str) -> usize {
        let name_tail = member_tail(name);
        for (known, arity) in self.type_arity_hints.iter().rev() {
            if known == name || known == name_tail {
                return *arity;
            }
        }
        0
    }
}
