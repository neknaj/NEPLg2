use crate::ast::{Literal, PrefixExpr};
use crate::intrinsic_kinds::FieldAccessorKind;
use crate::runtime_helpers::helper_base_name;
use crate::source_capability::owner_aggregate::OwnerAggregateEvidenceContext;
use crate::source_capability::scope::SourceCapabilityScope;
use crate::source_map::CompilerMemoryField;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) enum CompilerMemoryFieldEvidence {
    RepresentationField(CompilerMemoryField),
}

impl CompilerMemoryFieldEvidence {
    pub(in crate::source_capability) const fn field(self) -> CompilerMemoryField {
        match self {
            CompilerMemoryFieldEvidence::RepresentationField(field) => field,
        }
    }
}

pub(in crate::source_capability) fn compiler_memory_field_symbol_evidence(
    symbol: &str,
    selector: Option<&str>,
    scope: &SourceCapabilityScope,
    context: &OwnerAggregateEvidenceContext,
) -> Option<CompilerMemoryFieldEvidence> {
    if source_symbol_shadowed(symbol, scope) {
        return None;
    }
    let accessor = context.core_field_accessor_kind(symbol)?;
    compiler_memory_field_evidence_from_accessor(accessor, selector)
}

pub(in crate::source_capability) fn compiler_memory_field_intrinsic_evidence(
    symbol: &str,
    args: &[PrefixExpr],
) -> Option<CompilerMemoryFieldEvidence> {
    let accessor = FieldAccessorKind::from_intrinsic_name(helper_base_name(symbol))?;
    compiler_memory_field_evidence_from_accessor(accessor, prefix_arg_string_literal(args, 1))
}

fn compiler_memory_field_evidence_from_accessor(
    accessor: FieldAccessorKind,
    selector: Option<&str>,
) -> Option<CompilerMemoryFieldEvidence> {
    match accessor {
        FieldAccessorKind::Get | FieldAccessorKind::GetRef => selector
            .and_then(CompilerMemoryField::from_name)
            .map(CompilerMemoryFieldEvidence::RepresentationField),
        FieldAccessorKind::Put => None,
    }
}

fn prefix_arg_string_literal(args: &[PrefixExpr], index: usize) -> Option<&str> {
    let arg = args.get(index)?;
    match arg.items.as_slice() {
        [crate::ast::PrefixItem::Literal(Literal::Str(value), _)] => Some(value.as_str()),
        _ => None,
    }
}

fn source_symbol_shadowed(symbol: &str, scope: &SourceCapabilityScope) -> bool {
    scope.shadow_kind_symbol_or_qualifier(symbol).is_some()
}
