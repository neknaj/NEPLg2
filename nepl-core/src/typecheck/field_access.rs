use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::layout::composite_field_offset_bytes;
use crate::resource_primitives::compiler_memory_type_of_type;
use crate::source_map::CompilerMemoryType;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::copy_capability::target_contains_owner_backed_aggregate;
use super::diagnostics::type_error;
use super::model::RestrictedStructConstructor;
use super::{BlockChecker, FieldIdx};

impl<'a> BlockChecker<'a> {
    pub(super) fn restricted_struct_field_access_error(
        &mut self,
        base_ty: TypeId,
        field_ty: TypeId,
        span: Span,
    ) -> Option<(TypeDiagnosticCode, &'static str)> {
        if let Some(restricted) = self.restricted_struct_constructor_for_field_access(base_ty) {
            if !self.restricted_struct_field_access_allowed(restricted, span) {
                return Some(restricted_struct_field_access_error(restricted));
            }
        }

        if let Some(restricted) = self.restricted_struct_constructor_for_field_access(field_ty) {
            if !self.restricted_owner_field_projection_allowed(restricted, span) {
                return Some(restricted_struct_field_access_error(restricted));
            }
        }

        if self.owner_backed_aggregate_field_projection_restricted(field_ty, span) {
            return Some((
                TypeDiagnosticCode::OwnerAggregateFieldAccessRestricted,
                "owner-backed aggregate fields are restricted to compiler memory boundary",
            ));
        }

        None
    }

    fn owner_backed_aggregate_field_projection_restricted(
        &self,
        field_ty: TypeId,
        span: Span,
    ) -> bool {
        !self.owner_aggregate_field_boundary_allowed(span)
            && target_contains_owner_backed_aggregate(self.ctx, self.structs, field_ty)
    }

    fn restricted_struct_field_access_allowed(
        &self,
        restricted: RestrictedStructConstructor,
        span: Span,
    ) -> bool {
        if self.raw_memory_structural_boundary_allowed(span) {
            return true;
        }
        let Some(source_map) = self.source_map else {
            return false;
        };
        match restricted {
            RestrictedStructConstructor::OwnerToken => source_map
                .compiler_memory_type_definition_allowed(
                    span.file_id,
                    CompilerMemoryType::OwnerToken,
                ),
            RestrictedStructConstructor::RawPointer => source_map
                .compiler_memory_type_definition_allowed(
                    span.file_id,
                    CompilerMemoryType::RawPointer,
                ),
        }
    }

    fn restricted_owner_field_projection_allowed(
        &self,
        restricted: RestrictedStructConstructor,
        span: Span,
    ) -> bool {
        if restricted == RestrictedStructConstructor::OwnerToken
            && self.owner_aggregate_field_boundary_allowed(span)
        {
            return true;
        }
        self.restricted_struct_field_access_allowed(restricted, span)
    }

    fn restricted_struct_constructor_for_field_access(
        &self,
        base_ty: TypeId,
    ) -> Option<RestrictedStructConstructor> {
        match compiler_memory_type_of_type(self.ctx, base_ty) {
            Some(CompilerMemoryType::RawPointer) => Some(RestrictedStructConstructor::RawPointer),
            Some(CompilerMemoryType::OwnerToken) => Some(RestrictedStructConstructor::OwnerToken),
            None => None,
        }
    }

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
                checker.diagnostics.push(type_error(
                    TypeDiagnosticCode::FieldInvalidAccess,
                    message,
                    span,
                ));
            }
            None
        }

        let resolved_ty = self.ctx.resolve(base_ty);
        if let Some(restricted) = self.restricted_struct_constructor_for_field_access(resolved_ty) {
            if !self.restricted_struct_field_access_allowed(restricted, span) {
                if emit_diagnostics {
                    let (code, message) = restricted_struct_field_access_error(restricted);
                    self.diagnostics.push(type_error(code, message, span));
                }
                return None;
            }
        }

        let access = match self.ctx.get(resolved_ty) {
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
        };

        if let Some((field_ty, _offset)) = access {
            if let Some((code, message)) =
                self.restricted_struct_field_access_error(resolved_ty, field_ty, span)
            {
                if emit_diagnostics {
                    self.diagnostics.push(type_error(code, message, span));
                }
                return None;
            }
        }

        access
    }
}

fn restricted_struct_field_access_error(
    restricted: RestrictedStructConstructor,
) -> (TypeDiagnosticCode, &'static str) {
    match restricted {
        RestrictedStructConstructor::OwnerToken => (
            TypeDiagnosticCode::OwnerTokenFieldAccessRestricted,
            "owner token fields are restricted to compiler memory boundary",
        ),
        RestrictedStructConstructor::RawPointer => (
            TypeDiagnosticCode::RawPointerFieldAccessRestricted,
            "raw pointer fields are restricted to compiler memory boundary",
        ),
    }
}
