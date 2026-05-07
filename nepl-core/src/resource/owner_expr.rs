use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceExprKind};
use super::owner_check::ResourceOwnerCheckEngine;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn check_expr(
        &mut self,
        raw_aliases: &mut RawCellAddressAliases,
        kind: ResourceExprKind,
        output: &Place,
    ) {
        match kind {
            ResourceExprKind::LiteralI32(value) => raw_aliases.set_i32_value(output, value),
            ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct => {}
            ResourceExprKind::Literal
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Loop
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop => raw_aliases.clear(output),
            ResourceExprKind::Borrow => {}
        }
    }
}
