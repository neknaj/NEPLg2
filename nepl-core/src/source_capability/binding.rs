use alloc::collections::BTreeMap;
use alloc::string::String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SourceCapabilityBindingKind {
    TopLevelCallable,
    ImplMethod,
    LocalValue,
}

pub(super) fn bind_symbol_kind(
    bindings: &mut BTreeMap<String, SourceCapabilityBindingKind>,
    name: &str,
    kind: SourceCapabilityBindingKind,
) {
    match kind {
        SourceCapabilityBindingKind::LocalValue | SourceCapabilityBindingKind::TopLevelCallable => {
            bindings.insert(String::from(name), kind);
        }
        SourceCapabilityBindingKind::ImplMethod => {
            bindings.entry(String::from(name)).or_insert(kind);
        }
    }
}
