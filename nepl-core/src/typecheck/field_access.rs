use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::layout::composite_field_offset_bytes;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::{BlockChecker, FieldIdx};

impl<'a> BlockChecker<'a> {
    pub(super) fn resolve_field_access(
        &mut self,
        base_ty: TypeId,
        idx: FieldIdx,
        span: Span,
    ) -> Option<(TypeId, usize)> {
        self.resolve_field_access_with_mode(base_ty, idx, span, true)
    }

    pub(super) fn resolve_field_access_with_mode(
        &mut self,
        base_ty: TypeId,
        idx: FieldIdx,
        span: Span,
        emit_diagnostics: bool,
    ) -> Option<(TypeId, usize)> {
        fn invalid_field(
            checker: &mut BlockChecker<'_>,
            emit_diagnostics: bool,
            span: Span,
            message: String,
        ) -> Option<(TypeId, usize)> {
            if emit_diagnostics {
                checker
                    .diagnostics
                    .push(
                        Diagnostic::error(message, span).with_code(DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::FieldInvalidAccess,
                        )),
                    );
            }
            None
        }

        let resolved_ty = self.ctx.resolve(base_ty);
        match self.ctx.get(resolved_ty) {
            TypeKind::Struct {
                fields,
                field_names,
                ..
            } => match idx {
                FieldIdx::Index(i) => {
                    if i < fields.len() {
                        Some((
                            fields[i],
                            composite_field_offset_bytes(self.ctx, &fields, i),
                        ))
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("struct index out of bounds: {}", i),
                        )
                    }
                }
                FieldIdx::Name(name) => {
                    if let Some(i) = field_names.iter().position(|n| *n == name) {
                        Some((
                            fields[i],
                            composite_field_offset_bytes(self.ctx, &fields, i),
                        ))
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("struct has no field {}", name),
                        )
                    }
                }
            },
            TypeKind::Tuple { items } => match idx {
                FieldIdx::Index(i) => {
                    if i < items.len() {
                        Some((items[i], composite_field_offset_bytes(self.ctx, &items, i)))
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("tuple index out of bounds: {}", i),
                        )
                    }
                }
                FieldIdx::Name(name) => {
                    if let Ok(i) = name.parse::<usize>() {
                        if i < items.len() {
                            Some((items[i], composite_field_offset_bytes(self.ctx, &items, i)))
                        } else {
                            invalid_field(
                                self,
                                emit_diagnostics,
                                span,
                                format!("tuple index out of bounds: {}", i),
                            )
                        }
                    } else {
                        invalid_field(
                            self,
                            emit_diagnostics,
                            span,
                            format!("invalid tuple field access: {}", name),
                        )
                    }
                }
            },
            TypeKind::Apply { base, args } => {
                let base_ty = self.ctx.resolve(base);
                match self.ctx.get(base_ty) {
                    TypeKind::Struct {
                        type_params,
                        fields,
                        field_names,
                        ..
                    } => {
                        let mut mapping = BTreeMap::new();
                        for (tp, arg) in type_params.iter().zip(args.iter()) {
                            mapping.insert(*tp, *arg);
                        }
                        let substituted_fields = fields
                            .iter()
                            .map(|f| self.ctx.substitute(*f, &mapping))
                            .collect::<Vec<_>>();
                        match idx {
                            FieldIdx::Index(i) => {
                                if i < substituted_fields.len() {
                                    Some((
                                        substituted_fields[i],
                                        composite_field_offset_bytes(
                                            self.ctx,
                                            &substituted_fields,
                                            i,
                                        ),
                                    ))
                                } else {
                                    invalid_field(
                                        self,
                                        emit_diagnostics,
                                        span,
                                        format!("generic struct index out of bounds: {}", i),
                                    )
                                }
                            }
                            FieldIdx::Name(name) => {
                                if let Some(i) = field_names.iter().position(|n| *n == name) {
                                    Some((
                                        substituted_fields[i],
                                        composite_field_offset_bytes(
                                            self.ctx,
                                            &substituted_fields,
                                            i,
                                        ),
                                    ))
                                } else {
                                    invalid_field(
                                        self,
                                        emit_diagnostics,
                                        span,
                                        format!("generic struct has no field {}", name),
                                    )
                                }
                            }
                        }
                    }
                    TypeKind::Named(type_name) => {
                        if let Some(info) = self.structs.get(&type_name) {
                            let type_params = info.type_params.clone();
                            let fields = info.fields.clone();
                            let field_names = info.field_names.clone();
                            let mut mapping = BTreeMap::new();
                            for (tp, arg) in type_params.iter().zip(args.iter()) {
                                mapping.insert(*tp, *arg);
                            }
                            let substituted_fields = fields
                                .iter()
                                .map(|f| self.ctx.substitute(*f, &mapping))
                                .collect::<Vec<_>>();
                            match idx {
                                FieldIdx::Index(i) => {
                                    if i < substituted_fields.len() {
                                        Some((
                                            substituted_fields[i],
                                            composite_field_offset_bytes(
                                                self.ctx,
                                                &substituted_fields,
                                                i,
                                            ),
                                        ))
                                    } else {
                                        invalid_field(
                                            self,
                                            emit_diagnostics,
                                            span,
                                            format!("generic struct index out of bounds: {}", i),
                                        )
                                    }
                                }
                                FieldIdx::Name(name) => {
                                    if let Some(i) = field_names.iter().position(|n| *n == name) {
                                        Some((
                                            substituted_fields[i],
                                            composite_field_offset_bytes(
                                                self.ctx,
                                                &substituted_fields,
                                                i,
                                            ),
                                        ))
                                    } else {
                                        invalid_field(
                                            self,
                                            emit_diagnostics,
                                            span,
                                            format!("generic struct has no field {}", name),
                                        )
                                    }
                                }
                            }
                        } else {
                            invalid_field(
                                self,
                                emit_diagnostics,
                                span,
                                "cannot access field on this type".to_string(),
                            )
                        }
                    }
                    _ => invalid_field(
                        self,
                        emit_diagnostics,
                        span,
                        "cannot access field on this type".to_string(),
                    ),
                }
            }
            TypeKind::Named(type_name) => {
                if let Some(info) = self.structs.get(&type_name) {
                    let fields = info.fields.clone();
                    let field_names = info.field_names.clone();
                    match idx {
                        FieldIdx::Index(i) => {
                            if i < fields.len() {
                                Some((
                                    fields[i],
                                    composite_field_offset_bytes(self.ctx, &fields, i),
                                ))
                            } else {
                                invalid_field(
                                    self,
                                    emit_diagnostics,
                                    span,
                                    format!("struct index out of bounds: {}", i),
                                )
                            }
                        }
                        FieldIdx::Name(name) => {
                            if let Some(i) = field_names.iter().position(|n| *n == name) {
                                Some((
                                    fields[i],
                                    composite_field_offset_bytes(self.ctx, &fields, i),
                                ))
                            } else {
                                invalid_field(
                                    self,
                                    emit_diagnostics,
                                    span,
                                    format!("struct has no field {}", name),
                                )
                            }
                        }
                    }
                } else {
                    invalid_field(
                        self,
                        emit_diagnostics,
                        span,
                        "cannot access field on this type".to_string(),
                    )
                }
            }
            _ => invalid_field(
                self,
                emit_diagnostics,
                span,
                "cannot access field on non-composite type".to_string(),
            ),
        }
    }
}
