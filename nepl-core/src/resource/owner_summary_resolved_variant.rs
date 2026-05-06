extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeKind};

use super::model::{Place, ResourceExprKind, ResourceFunction, ResourceMatchPattern, ResourceOp};
use super::summary::OwnerResolvedParameterVariant;

pub(super) fn collect_resolved_parameter_variants_from_return(
    out: &mut Vec<OwnerResolvedParameterVariant>,
    function: &ResourceFunction,
    types: &TypeCtx,
    ops: &[ResourceOp],
    return_value: &Place,
) {
    let mut aliases = TransparentValueAliases::default();
    for op in ops {
        match op {
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                if output == return_value {
                    let Some(parameter_index) =
                        parameter_index_for_scrutinee(function, &aliases, scrutinee)
                    else {
                        aliases.clear_place(output);
                        continue;
                    };
                    let mut returning_variants = Vec::new();
                    for arm in arms {
                        if matches!(types.get(types.resolve_id(arm.value.ty)), TypeKind::Never) {
                            continue;
                        }
                        let Some(variant) = match_pattern_variant_name(&arm.pattern) else {
                            continue;
                        };
                        push_unique_resolved_variant_name(&mut returning_variants, variant);
                    }
                    if returning_variants.len() == 1 {
                        push_unique_resolved_parameter_variant(
                            out,
                            OwnerResolvedParameterVariant {
                                parameter_index,
                                variant: returning_variants.remove(0),
                            },
                        );
                    }
                }
                aliases.clear_place(output);
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                aliases.set_alias(output, source);
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: Some(initializer),
                ..
            } => {
                aliases.set_alias(place, initializer);
            }
            ResourceOp::Assign { target, value, .. } => {
                aliases.set_alias(target, value);
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: None,
                ..
            }
            | ResourceOp::Borrow { output: place, .. }
            | ResourceOp::FunctionValue { output: place, .. }
            | ResourceOp::Call { output: place, .. }
            | ResourceOp::IndirectCall { output: place, .. }
            | ResourceOp::RawMemory { output: place, .. }
            | ResourceOp::RawAddressAlias { target: place, .. }
            | ResourceOp::RawAddressView { target: place, .. }
            | ResourceOp::Construct { output: place, .. }
            | ResourceOp::Branch { output: place, .. } => {
                aliases.clear_place(place);
            }
            ResourceOp::Expr { output, kind, .. } => {
                if expr_kind_replaces_transparent_value_alias(*kind) {
                    aliases.clear_place(output);
                }
            }
            ResourceOp::Drop { place, .. } => {
                aliases.clear_place(place);
                aliases.clear_aliases_sourced_from(place);
            }
            ResourceOp::EndScope { locals, result, .. } => {
                for local in locals {
                    aliases.clear_place(local);
                    aliases.clear_aliases_sourced_from(local);
                }
                if let Some(result) = result {
                    aliases.clear_aliases_sourced_from(result);
                }
            }
            ResourceOp::Loop { .. } | ResourceOp::CallEffect { .. } => {}
        }
    }
}

fn expr_kind_replaces_transparent_value_alias(kind: ResourceExprKind) -> bool {
    matches!(
        kind,
        ResourceExprKind::Literal
            | ResourceExprKind::LiteralI32(_)
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Loop
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop
            | ResourceExprKind::Borrow
    )
}

#[derive(Debug, Default)]
struct TransparentValueAliases {
    entries: Vec<TransparentValueAlias>,
}

#[derive(Debug)]
struct TransparentValueAlias {
    target: Place,
    source: Place,
}

impl TransparentValueAliases {
    fn set_alias(&mut self, target: &Place, source: &Place) {
        self.clear_place(target);
        if !target.projections.is_empty() || !source.projections.is_empty() {
            return;
        }
        self.entries.push(TransparentValueAlias {
            target: target.clone(),
            source: source.clone(),
        });
    }

    fn clear_place(&mut self, place: &Place) {
        self.entries.retain(|entry| entry.target != *place);
    }

    fn clear_aliases_sourced_from(&mut self, place: &Place) {
        self.entries.retain(|entry| entry.source != *place);
    }

    fn source_for(&self, target: &Place) -> Option<&Place> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.target == *target)
            .map(|entry| &entry.source)
    }
}

fn parameter_index_for_scrutinee(
    function: &ResourceFunction,
    aliases: &TransparentValueAliases,
    place: &Place,
) -> Option<usize> {
    let mut current = place;
    for _ in 0..=aliases.entries.len() {
        if let Some(index) = parameter_index_for_place(function, current) {
            return Some(index);
        }
        let source = aliases.source_for(current)?;
        current = source;
    }
    None
}

fn parameter_index_for_place(function: &ResourceFunction, place: &Place) -> Option<usize> {
    if !place.projections.is_empty() {
        return None;
    }
    function
        .params
        .iter()
        .position(|param| param.place == *place)
}

fn match_pattern_variant_name(pattern: &ResourceMatchPattern) -> Option<String> {
    let ResourceMatchPattern::Variant(variant) = pattern else {
        return None;
    };
    Some(String::from(variant.rsplit("::").next().unwrap_or(variant)))
}

fn push_unique_resolved_variant_name(out: &mut Vec<String>, variant: String) {
    if !out.iter().any(|existing| existing == &variant) {
        out.push(variant);
    }
}

fn push_unique_resolved_parameter_variant(
    out: &mut Vec<OwnerResolvedParameterVariant>,
    entry: OwnerResolvedParameterVariant,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
