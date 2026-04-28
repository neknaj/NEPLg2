use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirExpr, HirExprKind, HirMatchArm, HirMatchPattern};
use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::types::{TypeId, TypeKind};

use super::provenance::{
    aggregate_field_raw_alias_at, field_move_path_from_addr, func_ref_name, i32_const_from_value,
    is_field_get_name, is_region_ptr_at_name, raw_addr_alias_from_value,
    raw_memory_place_key_from_region_token,
};
use super::raw_place::{
    combine_raw_memory_offsets, format_raw_memory_place_key_parts, parse_raw_memory_place_key,
    raw_place_key_has_unknown_offset,
};
use super::summary::{
    extend_unique_raw_memory_effects, FunctionRawAliasSummary, RawMemoryEffectSummary,
    ValueAliasSummary,
};
use super::summary_build::{block_raw_alias_summary, expression_raw_alias_summary};
use super::MoveCheckContext;
pub(super) fn singleton_function_alias(alias: String) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    aliases.insert(alias);
    aliases
}

pub(super) fn function_value_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    match &value.kind {
        HirExprKind::FnValue(name) => singleton_function_alias(name.clone()),
        HirExprKind::Var(name) => ctx.function_value_aliases(name),
        HirExprKind::Call { .. } => {
            let aliases = function_value_aliases_from_field_projection(value, ctx, tctx);
            if aliases.is_empty() {
                function_call_raw_alias_summary(value, ctx, tctx)
                    .map(|summary| summary.function_value_aliases)
                    .unwrap_or_default()
            } else {
                aliases
            }
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "load" && args.len() == 1 => {
            function_value_aliases_from_field_load(value, ctx, tctx)
        }
        _ => BTreeSet::new(),
    }
}

pub(super) fn value_alias_summary_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: raw_addr_alias_from_value(value, ctx, tctx),
        aggregate_field_raw_aliases: aggregate_field_raw_aliases_from_value(value, ctx, tctx),
        aggregate_field_function_aliases: aggregate_field_function_aliases_from_value(
            value, ctx, tctx,
        ),
        enum_payload_raw_aliases: enum_payload_raw_aliases_from_value(value, ctx, tctx),
        enum_payload_aggregate_field_raw_aliases:
            enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx),
        enum_payload_aggregate_field_function_aliases:
            enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx),
        enum_payload_function_aliases: enum_payload_function_aliases_from_value(value, ctx, tctx),
        function_value_aliases: function_value_aliases_from_value(value, ctx, tctx),
    }
}

pub(super) fn expression_function_value_aliases(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let aliases = function_value_aliases_from_value(value, ctx, tctx);
    if aliases.is_empty() {
        expression_raw_alias_summary(value, ctx, tctx).function_value_aliases
    } else {
        aliases
    }
}

pub(super) fn function_param_raw_alias_key(index: usize) -> String {
    alloc::format!("$param:{}", index)
}

pub(super) fn function_param_field_raw_alias_key(index: usize, offset: usize) -> String {
    alloc::format!("$param_field:{}:{}", index, offset)
}

pub(super) fn function_param_field_function_alias_key(index: usize, offset: usize) -> String {
    alloc::format!("$fnparam_field:{}:{}", index, offset)
}

pub(super) fn function_param_enum_payload_raw_alias_key(index: usize, variant: &str) -> String {
    alloc::format!("$param_enum_payload:{}:{}", index, variant)
}

pub(super) fn function_param_enum_payload_field_raw_alias_key(
    index: usize,
    variant: &str,
    offset: usize,
) -> String {
    alloc::format!("$param_enum_payload_field:{}:{}:{}", index, offset, variant)
}

pub(super) fn function_param_enum_payload_field_function_alias_key(
    index: usize,
    variant: &str,
    offset: usize,
) -> String {
    alloc::format!(
        "$fnparam_enum_payload_field:{}:{}:{}",
        index,
        offset,
        variant
    )
}

pub(super) fn function_param_enum_payload_function_alias_key(
    index: usize,
    variant: &str,
) -> String {
    alloc::format!("$fnparam_enum_payload:{}:{}", index, variant)
}

pub(super) fn function_param_function_alias_key(index: usize) -> String {
    alloc::format!("$fnparam:{}", index)
}

pub(super) fn is_function_param_function_alias_key(alias: &str) -> bool {
    alias
        .strip_prefix("$fnparam:")
        .and_then(|index_text| index_text.parse::<usize>().ok())
        .is_some()
}

pub(super) fn is_function_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    matches!(tctx.get_ref(tctx.resolve_id(ty)), TypeKind::Function { .. })
}

pub(super) fn aggregate_field_placeholder_aliases(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    mut make_key: impl FnMut(usize) -> String,
) -> BTreeMap<usize, String> {
    fn collect(
        tctx: &crate::types::TypeCtx,
        ty: TypeId,
        base_offset: usize,
        out: &mut BTreeMap<usize, String>,
        visiting: &mut BTreeSet<TypeId>,
        make_key: &mut dyn FnMut(usize) -> String,
    ) {
        let resolved = tctx.resolve_named_type_id(ty);
        if !visiting.insert(resolved) {
            return;
        }
        for field in aggregate_fields_with_offsets(tctx, ty) {
            let offset = base_offset.saturating_add(field.offset);
            out.insert(offset, make_key(offset));
            collect(tctx, field.ty, offset, out, visiting, make_key);
        }
        visiting.remove(&resolved);
    }

    let mut out = BTreeMap::new();
    collect(tctx, ty, 0, &mut out, &mut BTreeSet::new(), &mut make_key);
    out
}

pub(super) fn aggregate_field_function_placeholder_aliases(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    mut make_key: impl FnMut(usize) -> String,
) -> BTreeMap<usize, BTreeSet<String>> {
    fn collect(
        tctx: &crate::types::TypeCtx,
        ty: TypeId,
        base_offset: usize,
        out: &mut BTreeMap<usize, BTreeSet<String>>,
        visiting: &mut BTreeSet<TypeId>,
        make_key: &mut dyn FnMut(usize) -> String,
    ) {
        let resolved = tctx.resolve_named_type_id(ty);
        if !visiting.insert(resolved) {
            return;
        }
        for field in aggregate_fields_with_offsets(tctx, ty) {
            let offset = base_offset.saturating_add(field.offset);
            if is_function_type(tctx, field.ty) {
                out.insert(offset, singleton_function_alias(make_key(offset)));
            }
            collect(tctx, field.ty, offset, out, visiting, make_key);
        }
        visiting.remove(&resolved);
    }

    let mut out = BTreeMap::new();
    collect(tctx, ty, 0, &mut out, &mut BTreeSet::new(), &mut make_key);
    out
}

pub(super) fn enum_variants_for_type(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
) -> Vec<(String, Option<TypeId>)> {
    match tctx.get_ref(tctx.resolve_named_type_id(ty)) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .map(|variant| (variant.name.clone(), variant.payload))
            .collect(),
        TypeKind::Apply { base, args } => {
            let base = tctx.resolve_named_type_id(*base);
            match tctx.get_ref(base) {
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let mapping = crate::layout::extend_type_mapping(
                        tctx,
                        &BTreeMap::new(),
                        type_params,
                        args,
                    );
                    variants
                        .iter()
                        .map(|variant| {
                            (
                                variant.name.clone(),
                                variant.payload.map(|payload| {
                                    crate::layout::mapped_type_id(tctx, payload, &mapping)
                                }),
                            )
                        })
                        .collect()
                }
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

pub(super) fn enum_payload_raw_alias_from_value(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx
            .enum_payload_raw_alias(name, variant)
            .map(ToString::to_string),
        _ => {
            let aliases = enum_payload_raw_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant).cloned()
        }
    }
}

pub(super) fn enum_payload_aggregate_field_raw_aliases_from_expr(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.enum_payload_aggregate_field_raw_aliases(name, variant),
        _ => {
            let aliases = enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant)
                .cloned()
                .unwrap_or_default()
        }
    }
}

pub(super) fn enum_payload_aggregate_field_function_aliases_from_expr(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.enum_payload_aggregate_field_function_aliases(name, variant),
        _ => {
            let aliases =
                enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant)
                .cloned()
                .unwrap_or_default()
        }
    }
}

pub(super) fn enum_payload_function_aliases_from_expr(
    value: &HirExpr,
    variant: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.enum_payload_function_aliases_for_variant(name, variant),
        _ => {
            let aliases = enum_payload_function_aliases_from_value(value, ctx, tctx);
            variant_alias(&aliases, variant)
                .cloned()
                .unwrap_or_default()
        }
    }
}

pub(super) fn instantiate_raw_alias_base(
    base: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    if let Some(index_text) = base.strip_prefix("$param:") {
        let index = index_text.parse::<usize>().ok()?;
        return args
            .get(index)
            .and_then(|arg| raw_addr_alias_from_value(arg, ctx, tctx));
    }
    if let Some(rest) = base.strip_prefix("$param_field:") {
        let (index_text, offset_text) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        return args
            .get(index)
            .and_then(|arg| aggregate_field_raw_alias_at(arg, offset, ctx, tctx));
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload:") {
        let (index_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        return args
            .get(index)
            .and_then(|arg| enum_payload_raw_alias_from_value(arg, variant, ctx, tctx));
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload_field:") {
        let (index_text, rest) = rest.split_once(':')?;
        let (offset_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        return args.get(index).and_then(|arg| {
            enum_payload_aggregate_field_raw_aliases_from_expr(arg, variant, ctx, tctx)
                .get(&offset)
                .cloned()
        });
    }
    Some(base.to_string())
}

pub(super) fn instantiate_raw_alias_key(
    key: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let (base, offset) = parse_raw_memory_place_key(key);
    let instantiated = instantiate_raw_alias_base(base.as_str(), args, ctx, tctx)?;
    let (instantiated_base, instantiated_offset) =
        parse_raw_memory_place_key(instantiated.as_str());
    Some(format_raw_memory_place_key_parts(
        instantiated_base.as_str(),
        combine_raw_memory_offsets(instantiated_offset, offset),
    ))
}

pub(super) fn instantiate_raw_alias_base_from_value_summaries(
    base: &str,
    args: &[ValueAliasSummary],
) -> Option<String> {
    if let Some(index_text) = base.strip_prefix("$param:") {
        let index = index_text.parse::<usize>().ok()?;
        return args.get(index)?.raw_addr_alias.clone();
    }
    if let Some(rest) = base.strip_prefix("$param_field:") {
        let (index_text, offset_text) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        return args
            .get(index)?
            .aggregate_field_raw_aliases
            .get(&offset)
            .cloned();
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload:") {
        let (index_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let aliases = &args.get(index)?.enum_payload_raw_aliases;
        return variant_alias(aliases, variant).cloned();
    }
    if let Some(rest) = base.strip_prefix("$param_enum_payload_field:") {
        let (index_text, rest) = rest.split_once(':')?;
        let (offset_text, variant) = rest.split_once(':')?;
        let index = index_text.parse::<usize>().ok()?;
        let offset = offset_text.parse::<usize>().ok()?;
        let aliases = &args.get(index)?.enum_payload_aggregate_field_raw_aliases;
        return variant_alias(aliases, variant)
            .and_then(|field_aliases| field_aliases.get(&offset))
            .cloned();
    }
    Some(base.to_string())
}

pub(super) fn instantiate_raw_alias_key_from_value_summaries(
    key: &str,
    args: &[ValueAliasSummary],
) -> Option<String> {
    let (base, offset) = parse_raw_memory_place_key(key);
    let instantiated = instantiate_raw_alias_base_from_value_summaries(base.as_str(), args)?;
    let (instantiated_base, instantiated_offset) =
        parse_raw_memory_place_key(instantiated.as_str());
    Some(format_raw_memory_place_key_parts(
        instantiated_base.as_str(),
        combine_raw_memory_offsets(instantiated_offset, offset),
    ))
}

pub(super) fn instantiate_function_value_alias_key(
    alias: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    if let Some(index_text) = alias.strip_prefix("$fnparam:") {
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .map(|arg| function_value_aliases_from_value(arg, ctx, tctx))
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_field:") {
        let Some((index_text, offset_text)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .and_then(|arg| {
                aggregate_field_function_aliases_from_value(arg, ctx, tctx).remove(&offset)
            })
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload:") {
        let Some((index_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .map(|arg| enum_payload_function_aliases_from_expr(arg, variant, ctx, tctx))
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload_field:") {
        let Some((index_text, rest)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some((offset_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .and_then(|arg| {
                enum_payload_aggregate_field_function_aliases_from_expr(arg, variant, ctx, tctx)
                    .remove(&offset)
            })
            .unwrap_or_default();
    }
    singleton_function_alias(alias.to_string())
}

pub(super) fn instantiate_function_value_alias_key_from_value_summaries(
    alias: &str,
    args: &[ValueAliasSummary],
) -> BTreeSet<String> {
    if let Some(index_text) = alias.strip_prefix("$fnparam:") {
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .map(|summary| summary.function_value_aliases.clone())
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_field:") {
        let Some((index_text, offset_text)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        return args
            .get(index)
            .and_then(|summary| summary.aggregate_field_function_aliases.get(&offset))
            .cloned()
            .unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload:") {
        let Some((index_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(aliases) = args
            .get(index)
            .map(|summary| &summary.enum_payload_function_aliases)
        else {
            return BTreeSet::new();
        };
        return variant_alias(aliases, variant).cloned().unwrap_or_default();
    }
    if let Some(rest) = alias.strip_prefix("$fnparam_enum_payload_field:") {
        let Some((index_text, rest)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some((offset_text, variant)) = rest.split_once(':') else {
            return BTreeSet::new();
        };
        let Some(index) = index_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(offset) = offset_text.parse::<usize>().ok() else {
            return BTreeSet::new();
        };
        let Some(aliases) = args
            .get(index)
            .map(|summary| &summary.enum_payload_aggregate_field_function_aliases)
        else {
            return BTreeSet::new();
        };
        return variant_alias(aliases, variant)
            .and_then(|field_aliases| field_aliases.get(&offset))
            .cloned()
            .unwrap_or_default();
    }
    singleton_function_alias(alias.to_string())
}

pub(super) fn instantiate_value_alias_summary(
    summary: &ValueAliasSummary,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: summary
            .raw_addr_alias
            .as_ref()
            .and_then(|alias| instantiate_raw_alias_key(alias, args, ctx, tctx)),
        aggregate_field_raw_aliases: summary
            .aggregate_field_raw_aliases
            .iter()
            .filter_map(|(offset, alias)| {
                instantiate_raw_alias_key(alias, args, ctx, tctx).map(|alias| (*offset, alias))
            })
            .collect(),
        aggregate_field_function_aliases: summary
            .aggregate_field_function_aliases
            .iter()
            .filter_map(|(offset, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated
                        .extend(instantiate_function_value_alias_key(alias, args, ctx, tctx));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((*offset, instantiated))
                }
            })
            .collect(),
        enum_payload_raw_aliases: summary
            .enum_payload_raw_aliases
            .iter()
            .filter_map(|(variant, alias)| {
                instantiate_raw_alias_key(alias, args, ctx, tctx)
                    .map(|alias| (variant.clone(), alias))
            })
            .collect(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, alias)| {
                        instantiate_raw_alias_key(alias, args, ctx, tctx)
                            .map(|alias| (*offset, alias))
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, aliases)| {
                        let mut instantiated = BTreeSet::new();
                        for alias in aliases {
                            instantiated.extend(instantiate_function_value_alias_key(
                                alias, args, ctx, tctx,
                            ));
                        }
                        if instantiated.is_empty() {
                            None
                        } else {
                            Some((*offset, instantiated))
                        }
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_function_aliases: summary
            .enum_payload_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated
                        .extend(instantiate_function_value_alias_key(alias, args, ctx, tctx));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        function_value_aliases: {
            let mut aliases = BTreeSet::new();
            for alias in &summary.function_value_aliases {
                aliases.extend(instantiate_function_value_alias_key(alias, args, ctx, tctx));
            }
            aliases
        },
    }
}

pub(super) fn instantiate_value_alias_summary_from_value_summaries(
    summary: &ValueAliasSummary,
    args: &[ValueAliasSummary],
) -> ValueAliasSummary {
    ValueAliasSummary {
        raw_addr_alias: summary
            .raw_addr_alias
            .as_ref()
            .and_then(|alias| instantiate_raw_alias_key_from_value_summaries(alias, args)),
        aggregate_field_raw_aliases: summary
            .aggregate_field_raw_aliases
            .iter()
            .filter_map(|(offset, alias)| {
                instantiate_raw_alias_key_from_value_summaries(alias, args)
                    .map(|alias| (*offset, alias))
            })
            .collect(),
        aggregate_field_function_aliases: summary
            .aggregate_field_function_aliases
            .iter()
            .filter_map(|(offset, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated.extend(instantiate_function_value_alias_key_from_value_summaries(
                        alias, args,
                    ));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((*offset, instantiated))
                }
            })
            .collect(),
        enum_payload_raw_aliases: summary
            .enum_payload_raw_aliases
            .iter()
            .filter_map(|(variant, alias)| {
                instantiate_raw_alias_key_from_value_summaries(alias, args)
                    .map(|alias| (variant.clone(), alias))
            })
            .collect(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, alias)| {
                        instantiate_raw_alias_key_from_value_summaries(alias, args)
                            .map(|alias| (*offset, alias))
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let instantiated = aliases
                    .iter()
                    .filter_map(|(offset, aliases)| {
                        let mut instantiated = BTreeSet::new();
                        for alias in aliases {
                            instantiated.extend(
                                instantiate_function_value_alias_key_from_value_summaries(
                                    alias, args,
                                ),
                            );
                        }
                        if instantiated.is_empty() {
                            None
                        } else {
                            Some((*offset, instantiated))
                        }
                    })
                    .collect::<BTreeMap<_, _>>();
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        enum_payload_function_aliases: summary
            .enum_payload_function_aliases
            .iter()
            .filter_map(|(variant, aliases)| {
                let mut instantiated = BTreeSet::new();
                for alias in aliases {
                    instantiated.extend(instantiate_function_value_alias_key_from_value_summaries(
                        alias, args,
                    ));
                }
                if instantiated.is_empty() {
                    None
                } else {
                    Some((variant.clone(), instantiated))
                }
            })
            .collect(),
        function_value_aliases: {
            let mut aliases = BTreeSet::new();
            for alias in &summary.function_value_aliases {
                aliases.extend(instantiate_function_value_alias_key_from_value_summaries(
                    alias, args,
                ));
            }
            aliases
        },
    }
}

pub(super) fn instantiate_function_raw_alias_summary(
    summary: &FunctionRawAliasSummary,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> FunctionRawAliasSummary {
    let value = ValueAliasSummary {
        raw_addr_alias: summary.raw_addr_alias.clone(),
        aggregate_field_raw_aliases: summary.aggregate_field_raw_aliases.clone(),
        aggregate_field_function_aliases: summary.aggregate_field_function_aliases.clone(),
        enum_payload_raw_aliases: summary.enum_payload_raw_aliases.clone(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .clone(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .clone(),
        enum_payload_function_aliases: summary.enum_payload_function_aliases.clone(),
        function_value_aliases: summary.function_value_aliases.clone(),
    };
    let value = instantiate_value_alias_summary(&value, args, ctx, tctx);
    let mut raw_memory_effects = Vec::new();
    let max_depth = ctx.function_raw_alias_summaries.len().saturating_add(1);
    for effect in &summary.raw_memory_effects {
        extend_unique_raw_memory_effects(
            &mut raw_memory_effects,
            instantiate_raw_memory_effect_summary(effect, args, ctx, tctx, max_depth),
        );
    }
    FunctionRawAliasSummary {
        raw_addr_alias: value.raw_addr_alias,
        aggregate_field_raw_aliases: value.aggregate_field_raw_aliases,
        aggregate_field_function_aliases: value.aggregate_field_function_aliases,
        enum_payload_raw_aliases: value.enum_payload_raw_aliases,
        enum_payload_aggregate_field_raw_aliases: value.enum_payload_aggregate_field_raw_aliases,
        enum_payload_aggregate_field_function_aliases: value
            .enum_payload_aggregate_field_function_aliases,
        enum_payload_function_aliases: value.enum_payload_function_aliases,
        function_value_aliases: value.function_value_aliases,
        raw_memory_effects,
    }
}

pub(super) fn instantiate_raw_memory_effect_summary(
    effect: &RawMemoryEffectSummary,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> Vec<RawMemoryEffectSummary> {
    match effect {
        RawMemoryEffectSummary::Load { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Load { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Store { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Store { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Dealloc { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Dealloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Realloc { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::Realloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::BulkCopy { dst, src, size } => {
            match (
                instantiate_raw_alias_key(dst, args, ctx, tctx),
                instantiate_raw_alias_key(src, args, ctx, tctx),
            ) {
                (Some(dst), Some(src)) => alloc::vec![RawMemoryEffectSummary::BulkCopy {
                    dst,
                    src,
                    size: *size,
                }],
                _ => Vec::new(),
            }
        }
        RawMemoryEffectSummary::ByteWrite { place, size } => {
            instantiate_raw_alias_key(place, args, ctx, tctx)
                .map(|place| RawMemoryEffectSummary::ByteWrite { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::IndirectCall {
            callee,
            args: call_args,
        } => {
            let callees = instantiate_function_value_alias_key(callee, args, ctx, tctx);
            if callees.is_empty() {
                return Vec::new();
            }
            let instantiated_args = call_args
                .iter()
                .map(|arg| instantiate_value_alias_summary(arg, args, ctx, tctx))
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            for callee in callees {
                let effects = instantiate_known_function_raw_memory_effects(
                    callee.as_str(),
                    &instantiated_args,
                    ctx,
                    tctx,
                    remaining_depth.saturating_sub(1),
                );
                if effects.is_empty() && is_function_param_function_alias_key(callee.as_str()) {
                    extend_unique_raw_memory_effects(
                        &mut out,
                        [RawMemoryEffectSummary::IndirectCall {
                            callee,
                            args: instantiated_args.clone(),
                        }],
                    );
                } else {
                    extend_unique_raw_memory_effects(&mut out, effects);
                }
            }
            out
        }
    }
}

pub(super) fn instantiate_function_raw_alias_summary_from_value_summaries(
    summary: &FunctionRawAliasSummary,
    args: &[ValueAliasSummary],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> FunctionRawAliasSummary {
    let value = ValueAliasSummary {
        raw_addr_alias: summary.raw_addr_alias.clone(),
        aggregate_field_raw_aliases: summary.aggregate_field_raw_aliases.clone(),
        aggregate_field_function_aliases: summary.aggregate_field_function_aliases.clone(),
        enum_payload_raw_aliases: summary.enum_payload_raw_aliases.clone(),
        enum_payload_aggregate_field_raw_aliases: summary
            .enum_payload_aggregate_field_raw_aliases
            .clone(),
        enum_payload_aggregate_field_function_aliases: summary
            .enum_payload_aggregate_field_function_aliases
            .clone(),
        enum_payload_function_aliases: summary.enum_payload_function_aliases.clone(),
        function_value_aliases: summary.function_value_aliases.clone(),
    };
    let value = instantiate_value_alias_summary_from_value_summaries(&value, args);
    let mut raw_memory_effects = Vec::new();
    for effect in &summary.raw_memory_effects {
        extend_unique_raw_memory_effects(
            &mut raw_memory_effects,
            instantiate_raw_memory_effect_summary_from_value_summaries(
                effect,
                args,
                ctx,
                tctx,
                remaining_depth,
            ),
        );
    }
    FunctionRawAliasSummary {
        raw_addr_alias: value.raw_addr_alias,
        aggregate_field_raw_aliases: value.aggregate_field_raw_aliases,
        aggregate_field_function_aliases: value.aggregate_field_function_aliases,
        enum_payload_raw_aliases: value.enum_payload_raw_aliases,
        enum_payload_aggregate_field_raw_aliases: value.enum_payload_aggregate_field_raw_aliases,
        enum_payload_aggregate_field_function_aliases: value
            .enum_payload_aggregate_field_function_aliases,
        enum_payload_function_aliases: value.enum_payload_function_aliases,
        function_value_aliases: value.function_value_aliases,
        raw_memory_effects,
    }
}

pub(super) fn instantiate_raw_memory_effect_summary_from_value_summaries(
    effect: &RawMemoryEffectSummary,
    args: &[ValueAliasSummary],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> Vec<RawMemoryEffectSummary> {
    match effect {
        RawMemoryEffectSummary::Load { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Load { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Store { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Store { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Dealloc { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Dealloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::Realloc { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::Realloc { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::BulkCopy { dst, src, size } => {
            match (
                instantiate_raw_alias_key_from_value_summaries(dst, args),
                instantiate_raw_alias_key_from_value_summaries(src, args),
            ) {
                (Some(dst), Some(src)) => alloc::vec![RawMemoryEffectSummary::BulkCopy {
                    dst,
                    src,
                    size: *size,
                }],
                _ => Vec::new(),
            }
        }
        RawMemoryEffectSummary::ByteWrite { place, size } => {
            instantiate_raw_alias_key_from_value_summaries(place, args)
                .map(|place| RawMemoryEffectSummary::ByteWrite { place, size: *size })
                .into_iter()
                .collect()
        }
        RawMemoryEffectSummary::IndirectCall {
            callee,
            args: call_args,
        } => {
            let callees = instantiate_function_value_alias_key_from_value_summaries(callee, args);
            if callees.is_empty() {
                return Vec::new();
            }
            let instantiated_args = call_args
                .iter()
                .map(|arg| instantiate_value_alias_summary_from_value_summaries(arg, args))
                .collect::<Vec<_>>();
            let mut out = Vec::new();
            for callee in callees {
                let effects = instantiate_known_function_raw_memory_effects(
                    callee.as_str(),
                    &instantiated_args,
                    ctx,
                    tctx,
                    remaining_depth.saturating_sub(1),
                );
                if effects.is_empty() && is_function_param_function_alias_key(callee.as_str()) {
                    extend_unique_raw_memory_effects(
                        &mut out,
                        [RawMemoryEffectSummary::IndirectCall {
                            callee,
                            args: instantiated_args.clone(),
                        }],
                    );
                } else {
                    extend_unique_raw_memory_effects(&mut out, effects);
                }
            }
            out
        }
    }
}

pub(super) fn instantiate_known_function_raw_memory_effects(
    callee: &str,
    args: &[ValueAliasSummary],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    remaining_depth: usize,
) -> Vec<RawMemoryEffectSummary> {
    if remaining_depth == 0 {
        return Vec::new();
    }
    let Some(summary) = ctx.function_raw_alias_summaries.get(callee) else {
        return Vec::new();
    };
    instantiate_function_raw_alias_summary_from_value_summaries(
        summary,
        args,
        ctx,
        tctx,
        remaining_depth,
    )
    .raw_memory_effects
}

pub(super) fn raw_alias_summary_needs_call_site_specialization(
    summary: &FunctionRawAliasSummary,
) -> bool {
    summary
        .raw_addr_alias
        .as_deref()
        .is_some_and(raw_place_key_has_unknown_offset)
}

pub(super) fn specialized_function_raw_alias_summary(
    name: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<FunctionRawAliasSummary> {
    if ctx
        .raw_alias_specialization_stack
        .iter()
        .any(|active| active == name)
    {
        return None;
    }
    let func = ctx.function_defs.get(name)?;
    if func.params.len() != args.len() {
        return None;
    }
    let mut call_ctx = ctx.clone_for_alias_summary();
    call_ctx
        .raw_alias_specialization_stack
        .push(name.to_string());
    call_ctx.push_scope();
    for (param, arg) in func.params.iter().zip(args) {
        let value_summary = value_alias_summary_from_value(arg, ctx, tctx);
        let i32_const_alias = i32_const_from_value(arg, ctx, tctx);
        call_ctx.declare_var(param.name.clone());
        call_ctx.set_raw_addr_alias(&param.name, value_summary.raw_addr_alias);
        call_ctx.set_i32_const_alias(&param.name, i32_const_alias);
        call_ctx.set_enum_payload_raw_aliases(&param.name, value_summary.enum_payload_raw_aliases);
        call_ctx.set_aggregate_field_raw_aliases(
            &param.name,
            value_summary.aggregate_field_raw_aliases,
        );
        call_ctx.set_aggregate_field_function_aliases(
            &param.name,
            value_summary.aggregate_field_function_aliases,
        );
        call_ctx.set_enum_payload_aggregate_field_raw_aliases(
            &param.name,
            value_summary.enum_payload_aggregate_field_raw_aliases,
        );
        call_ctx.set_enum_payload_aggregate_field_function_aliases(
            &param.name,
            value_summary.enum_payload_aggregate_field_function_aliases,
        );
        call_ctx.set_enum_payload_function_aliases(
            &param.name,
            value_summary.enum_payload_function_aliases,
        );
        call_ctx.set_function_value_aliases(&param.name, value_summary.function_value_aliases);
    }
    match &func.body {
        crate::hir::HirBody::Block(block) => Some(block_raw_alias_summary(block, &call_ctx, tctx)),
        _ => None,
    }
}

pub(super) fn function_call_raw_alias_summary(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<FunctionRawAliasSummary> {
    let HirExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    let FuncRef::User(name, _, _) = callee else {
        return None;
    };
    let summary = ctx.function_raw_alias_summaries.get(name)?;
    let instantiated = instantiate_function_raw_alias_summary(summary, args, ctx, tctx);
    if raw_alias_summary_needs_call_site_specialization(&instantiated) {
        specialized_function_raw_alias_summary(name, args, ctx, tctx).or(Some(instantiated))
    } else {
        Some(instantiated)
    }
}

pub(super) fn aggregate_field_index_by_name(
    tctx: &crate::types::TypeCtx,
    ty: TypeId,
    field_name: &str,
) -> Option<usize> {
    let ty = tctx.resolve_named_type_id(ty);
    match tctx.get_ref(ty) {
        TypeKind::Struct { field_names, .. } => {
            field_names.iter().position(|name| name == field_name)
        }
        TypeKind::Tuple { items } => field_name
            .parse::<usize>()
            .ok()
            .filter(|index| *index < items.len()),
        TypeKind::Apply { base, .. } => {
            let base = tctx.resolve_named_type_id(*base);
            match tctx.get_ref(base) {
                TypeKind::Struct { field_names, .. } => {
                    field_names.iter().position(|name| name == field_name)
                }
                TypeKind::Tuple { items } => field_name
                    .parse::<usize>()
                    .ok()
                    .filter(|index| *index < items.len()),
                _ => None,
            }
        }
        _ => None,
    }
}

pub(super) fn aggregate_field_layout_from_selector(
    owner_ty: TypeId,
    selector: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<(usize, TypeId)> {
    let index = match &selector.kind {
        HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
        HirExprKind::LiteralStr(id) => {
            let field_name = ctx.string_literal(*id)?;
            aggregate_field_index_by_name(tctx, owner_ty, field_name)
        }
        _ => None,
    }?;
    aggregate_fields_with_offsets(tctx, owner_ty)
        .get(index)
        .map(|field| (field.offset, field.ty))
}

pub(super) fn field_get_projection<'a>(
    expr: &'a HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<(&'a HirExpr, usize, TypeId)> {
    let HirExprKind::Call { callee, args } = &expr.kind else {
        return None;
    };
    if args.len() < 2 {
        return None;
    }
    let name = func_ref_name(callee)?;
    if !is_field_get_name(name) {
        return None;
    }
    let (offset, field_ty) = aggregate_field_layout_from_selector(args[0].ty, &args[1], ctx, tctx)?;
    Some((&args[0], offset, field_ty))
}

pub(super) fn is_result_ok_variant_name(name: &str) -> bool {
    name == "Ok" || name.ends_with("::Ok")
}

pub(super) fn variant_short_name(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

pub(super) fn variant_alias<'a, T>(
    aliases: &'a BTreeMap<String, T>,
    variant: &str,
) -> Option<&'a T> {
    aliases.get(variant).or_else(|| {
        let short = variant_short_name(variant);
        aliases.get(short).or_else(|| {
            aliases.iter().find_map(|(key, value)| {
                if variant_short_name(key.as_str()) == short {
                    Some(value)
                } else {
                    None
                }
            })
        })
    })
}

pub(super) fn pattern_variant_name(arm: &HirMatchArm) -> Option<&str> {
    match &arm.pattern {
        HirMatchPattern::Variant(name) => Some(name.as_str()),
        _ => None,
    }
}

pub(super) fn region_ptr_at_result_ok_raw_alias(
    scrutinee: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let HirExprKind::Call { callee, args } = &scrutinee.kind else {
        return None;
    };
    let name = func_ref_name(callee)?;
    if !is_region_ptr_at_name(name) || args.len() < 2 {
        return None;
    }
    let key = raw_memory_place_key_from_region_token(&args[0], ctx, tctx)?;
    let offset = match &args[1].kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        _ => None,
    };
    let (base, base_offset) = parse_raw_memory_place_key(key.as_str());
    Some(format_raw_memory_place_key_parts(
        base.as_str(),
        combine_raw_memory_offsets(base_offset, offset),
    ))
}

pub(super) fn match_bind_raw_addr_alias(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let variant_name = pattern_variant_name(arm)?;
    if let Some(alias) = enum_payload_raw_alias_from_value(scrutinee, variant_name, ctx, tctx) {
        return Some(alias);
    }
    if is_result_ok_variant_name(variant_name) {
        region_ptr_at_result_ok_raw_alias(scrutinee, ctx, tctx)
    } else {
        None
    }
}

pub(super) fn match_bind_aggregate_field_raw_aliases(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let Some(variant_name) = pattern_variant_name(arm) else {
        return BTreeMap::new();
    };
    enum_payload_aggregate_field_raw_aliases_from_expr(scrutinee, variant_name, ctx, tctx)
}

pub(super) fn match_bind_aggregate_field_function_aliases(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let Some(variant_name) = pattern_variant_name(arm) else {
        return BTreeMap::new();
    };
    enum_payload_aggregate_field_function_aliases_from_expr(scrutinee, variant_name, ctx, tctx)
}

pub(super) fn match_bind_function_value_aliases(
    scrutinee: &HirExpr,
    arm: &HirMatchArm,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let Some(variant_name) = pattern_variant_name(arm) else {
        return BTreeSet::new();
    };
    enum_payload_function_aliases_from_expr(scrutinee, variant_name, ctx, tctx)
}

pub(super) fn enum_payload_raw_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    match &value.kind {
        HirExprKind::EnumConstruct {
            variant,
            payload: Some(payload),
            ..
        } => {
            if let Some(alias) = raw_addr_alias_from_value(payload, ctx, tctx) {
                aliases.insert(variant.clone(), alias);
            }
        }
        _ => {
            if let Some(alias) = region_ptr_at_result_ok_raw_alias(value, ctx, tctx) {
                aliases.insert(String::from("Ok"), alias);
            } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                aliases = summary.enum_payload_raw_aliases;
            }
        }
    }
    aliases
}

pub(super) fn enum_payload_aggregate_field_raw_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, BTreeMap<usize, String>> {
    let mut aliases = BTreeMap::new();
    if let HirExprKind::EnumConstruct {
        variant,
        payload: Some(payload),
        ..
    } = &value.kind
    {
        let aggregate_aliases = aggregate_field_raw_aliases_from_value(payload, ctx, tctx);
        if !aggregate_aliases.is_empty() {
            aliases.insert(variant.clone(), aggregate_aliases);
        }
    } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
        aliases = summary.enum_payload_aggregate_field_raw_aliases;
    }
    aliases
}

pub(super) fn enum_payload_aggregate_field_function_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, BTreeMap<usize, BTreeSet<String>>> {
    let mut aliases = BTreeMap::new();
    if let HirExprKind::EnumConstruct {
        variant,
        payload: Some(payload),
        ..
    } = &value.kind
    {
        let aggregate_aliases = aggregate_field_function_aliases_from_value(payload, ctx, tctx);
        if !aggregate_aliases.is_empty() {
            aliases.insert(variant.clone(), aggregate_aliases);
        }
    } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
        aliases = summary.enum_payload_aggregate_field_function_aliases;
    }
    aliases
}

pub(super) fn enum_payload_function_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut aliases = BTreeMap::new();
    match &value.kind {
        HirExprKind::EnumConstruct {
            variant,
            payload: Some(payload),
            ..
        } => {
            let function_aliases = function_value_aliases_from_value(payload, ctx, tctx);
            if !function_aliases.is_empty() {
                aliases.insert(variant.clone(), function_aliases);
            }
        }
        _ => {
            if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                aliases = summary.enum_payload_function_aliases;
            }
        }
    }
    aliases
}

pub(super) fn aggregate_field_raw_aliases_from_items(
    value_ty: TypeId,
    items: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let layouts = aggregate_fields_with_offsets(tctx, value_ty);
    let mut aliases = BTreeMap::new();
    for (item, layout) in items.iter().zip(layouts.into_iter()) {
        if let Some(alias) = raw_addr_alias_from_value(item, ctx, tctx) {
            aliases.insert(layout.offset, alias);
        }
        for (nested_offset, alias) in aggregate_field_raw_aliases_from_value(item, ctx, tctx) {
            aliases.insert(layout.offset.saturating_add(nested_offset), alias);
        }
    }
    aliases
}

pub(super) fn aggregate_field_raw_aliases_from_projection(
    owner: &HirExpr,
    field_offset: usize,
    field_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let field_size = storage_size_bytes(tctx, field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = field_offset.saturating_add(field_size);
    aggregate_field_raw_aliases_from_value(owner, ctx, tctx)
        .into_iter()
        .filter_map(|(offset, alias)| {
            if field_offset <= offset && offset < field_end {
                Some((offset - field_offset, alias))
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn aggregate_field_raw_aliases_from_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return BTreeMap::new();
    };
    if name != "load" || args.len() != 1 {
        return BTreeMap::new();
    }
    let Some(path) = field_move_path_from_addr(&args[0], value.ty, tctx) else {
        return BTreeMap::new();
    };
    let field_size = storage_size_bytes(tctx, path.field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = path.offset.saturating_add(field_size);
    ctx.aggregate_field_raw_aliases(path.owner.as_str())
        .into_iter()
        .filter_map(|(offset, alias)| {
            if path.offset <= offset && offset < field_end {
                Some((offset - path.offset, alias))
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn aggregate_field_function_aliases_from_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return BTreeMap::new();
    };
    if name != "load" || args.len() != 1 {
        return BTreeMap::new();
    }
    let Some(path) = field_move_path_from_addr(&args[0], value.ty, tctx) else {
        return BTreeMap::new();
    };
    let field_size = storage_size_bytes(tctx, path.field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = path.offset.saturating_add(field_size);
    ctx.aggregate_field_function_aliases(path.owner.as_str())
        .into_iter()
        .filter_map(|(offset, aliases)| {
            if path.offset <= offset && offset < field_end {
                Some((offset - path.offset, aliases))
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn aggregate_field_raw_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, String> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.aggregate_field_raw_aliases(name),
        HirExprKind::StructConstruct { fields, .. } => {
            aggregate_field_raw_aliases_from_items(value.ty, fields, ctx, tctx)
        }
        HirExprKind::TupleConstruct { items } => {
            aggregate_field_raw_aliases_from_items(value.ty, items, ctx, tctx)
        }
        HirExprKind::Call { .. } => {
            if let Some((owner, offset, field_ty)) = field_get_projection(value, ctx, tctx) {
                aggregate_field_raw_aliases_from_projection(owner, offset, field_ty, ctx, tctx)
            } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                summary.aggregate_field_raw_aliases
            } else {
                BTreeMap::new()
            }
        }
        HirExprKind::Intrinsic { .. } => {
            aggregate_field_raw_aliases_from_field_load(value, ctx, tctx)
        }
        _ => BTreeMap::new(),
    }
}

pub(super) fn aggregate_field_function_aliases_from_items(
    value_ty: TypeId,
    items: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let layouts = aggregate_fields_with_offsets(tctx, value_ty);
    let mut aliases = BTreeMap::new();
    for (item, layout) in items.iter().zip(layouts.into_iter()) {
        let item_aliases = function_value_aliases_from_value(item, ctx, tctx);
        if !item_aliases.is_empty() {
            aliases.insert(layout.offset, item_aliases);
        }
        for (nested_offset, nested_aliases) in
            aggregate_field_function_aliases_from_value(item, ctx, tctx)
        {
            aliases
                .entry(layout.offset.saturating_add(nested_offset))
                .or_insert_with(BTreeSet::new)
                .extend(nested_aliases);
        }
    }
    aliases
}

pub(super) fn aggregate_field_function_aliases_from_projection(
    owner: &HirExpr,
    field_offset: usize,
    field_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    let field_size = storage_size_bytes(tctx, field_ty);
    if field_size == 0 {
        return BTreeMap::new();
    }
    let field_end = field_offset.saturating_add(field_size);
    aggregate_field_function_aliases_from_value(owner, ctx, tctx)
        .into_iter()
        .filter_map(|(offset, aliases)| {
            if field_offset <= offset && offset < field_end {
                Some((offset - field_offset, aliases))
            } else {
                None
            }
        })
        .collect()
}

pub(super) fn function_value_aliases_from_field_projection(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let Some((owner, offset, _field_ty)) = field_get_projection(value, ctx, tctx) else {
        return BTreeSet::new();
    };
    aggregate_field_function_aliases_from_value(owner, ctx, tctx)
        .remove(&offset)
        .unwrap_or_default()
}

pub(super) fn function_value_aliases_from_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeSet<String> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return BTreeSet::new();
    };
    if name != "load" || args.len() != 1 {
        return BTreeSet::new();
    }
    let Some(path) = field_move_path_from_addr(&args[0], value.ty, tctx) else {
        return BTreeSet::new();
    };
    ctx.aggregate_field_function_aliases(path.owner.as_str())
        .remove(&path.offset)
        .unwrap_or_default()
}

pub(super) fn aggregate_field_function_aliases_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<usize, BTreeSet<String>> {
    match &value.kind {
        HirExprKind::Var(name) => ctx.aggregate_field_function_aliases(name),
        HirExprKind::StructConstruct { fields, .. } => {
            aggregate_field_function_aliases_from_items(value.ty, fields, ctx, tctx)
        }
        HirExprKind::TupleConstruct { items } => {
            aggregate_field_function_aliases_from_items(value.ty, items, ctx, tctx)
        }
        HirExprKind::Call { .. } => {
            if let Some((owner, offset, field_ty)) = field_get_projection(value, ctx, tctx) {
                aggregate_field_function_aliases_from_projection(owner, offset, field_ty, ctx, tctx)
            } else if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
                summary.aggregate_field_function_aliases
            } else {
                BTreeMap::new()
            }
        }
        HirExprKind::Intrinsic { .. } => {
            aggregate_field_function_aliases_from_field_load(value, ctx, tctx)
        }
        _ => BTreeMap::new(),
    }
}
