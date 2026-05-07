use super::model::ResourceI32RelationOp;

pub(super) fn relation_negation(op: ResourceI32RelationOp) -> ResourceI32RelationOp {
    match op {
        ResourceI32RelationOp::Eq => ResourceI32RelationOp::Ne,
        ResourceI32RelationOp::Ne => ResourceI32RelationOp::Eq,
        ResourceI32RelationOp::Lt => ResourceI32RelationOp::Ge,
        ResourceI32RelationOp::Le => ResourceI32RelationOp::Gt,
        ResourceI32RelationOp::Gt => ResourceI32RelationOp::Le,
        ResourceI32RelationOp::Ge => ResourceI32RelationOp::Lt,
    }
}

pub(super) fn relation_reverse(op: ResourceI32RelationOp) -> ResourceI32RelationOp {
    match op {
        ResourceI32RelationOp::Eq => ResourceI32RelationOp::Eq,
        ResourceI32RelationOp::Ne => ResourceI32RelationOp::Ne,
        ResourceI32RelationOp::Lt => ResourceI32RelationOp::Gt,
        ResourceI32RelationOp::Le => ResourceI32RelationOp::Ge,
        ResourceI32RelationOp::Gt => ResourceI32RelationOp::Lt,
        ResourceI32RelationOp::Ge => ResourceI32RelationOp::Le,
    }
}

pub(super) fn relation_implication(
    known: ResourceI32RelationOp,
    query: ResourceI32RelationOp,
) -> Option<bool> {
    use ResourceI32RelationOp::{Eq, Ge, Gt, Le, Lt, Ne};
    match (known, query) {
        (left, right) if left == right => Some(true),
        (Eq, Ne) | (Ne, Eq) => Some(false),
        (Eq, Le | Ge) => Some(true),
        (Lt, Le) | (Gt, Ge) => Some(true),
        (Lt, Eq | Gt | Ge) => Some(false),
        (Le, Gt) => Some(false),
        (Gt, Eq | Lt | Le) => Some(false),
        (Ge, Lt) => Some(false),
        _ => None,
    }
}

pub(super) fn relation_holds(left: i32, op: ResourceI32RelationOp, right: i32) -> bool {
    match op {
        ResourceI32RelationOp::Eq => left == right,
        ResourceI32RelationOp::Ne => left != right,
        ResourceI32RelationOp::Lt => left < right,
        ResourceI32RelationOp::Le => left <= right,
        ResourceI32RelationOp::Gt => left > right,
        ResourceI32RelationOp::Ge => left >= right,
    }
}
