extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_relation_op::{relation_implication, relation_reverse};
use super::model::{Place, ResourceI32RelationOp};

impl RawCellAddressAliases {
    pub(super) fn i32_strict_upper_bound_candidates(&self, place: &Place) -> Vec<Place> {
        let aliases = self.scalar_aliases_for(place);
        let mut out = Vec::new();
        for fact in self.i32_relations.relations_touching_aliases(&aliases) {
            if aliases.iter().any(|alias| alias == &fact.left)
                && relation_implication(fact.op, ResourceI32RelationOp::Lt) == Some(true)
            {
                push_unique_place(&mut out, self.canonicalize_scalar(&fact.right));
            } else if aliases.iter().any(|alias| alias == &fact.right)
                && relation_implication(relation_reverse(fact.op), ResourceI32RelationOp::Lt)
                    == Some(true)
            {
                push_unique_place(&mut out, self.canonicalize_scalar(&fact.left));
            }
        }
        out
    }
}

fn push_unique_place(out: &mut Vec<Place>, place: Place) {
    if !out.iter().any(|existing| existing == &place) {
        out.push(place);
    }
}
