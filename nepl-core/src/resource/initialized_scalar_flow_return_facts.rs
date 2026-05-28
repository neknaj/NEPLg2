extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::i32_scalar_return_facts::{
    I32ScalarParameterCondition, I32ScalarReturnAlias, I32ScalarReturnCondition,
    I32ScalarReturnConstant, I32ScalarReturnFacts, I32ScalarReturnOffset, I32ScalarReturnRelation,
};
use super::initialized_scalar_flow::I32ScalarConcreteVariants;
use super::model::{Place, PlaceProjection};
use super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;

pub(super) trait I32ScalarReturnProjectedFact: Clone + Eq {
    fn return_projection(&self) -> &[PlaceProjection];
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnAlias {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnOffset {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnConstant {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

impl I32ScalarReturnProjectedFact for I32ScalarReturnCondition {
    fn return_projection(&self) -> &[PlaceProjection] {
        &self.return_projection
    }
}

pub(super) fn i32_scalar_return_fact_projections(
    types: &TypeCtx,
    value: &Place,
    facts: &I32ScalarReturnFacts,
    concrete_variants: &I32ScalarConcreteVariants,
    leaf_cache: &mut I32LeafProjectionCache,
) -> Vec<Vec<PlaceProjection>> {
    let mut projections = Vec::new();
    for leaf in leaf_cache.leaf_places_for_conditions(types, value) {
        if !concrete_variants.projection_is_possible(types, value, &leaf.suffix) {
            continue;
        }
        push_unique_i32_scalar_return_projection(&mut projections, &leaf.suffix);
    }
    concrete_variants.push_variant_projection_paths(value, &mut projections);
    for alias in &facts.aliases {
        push_unique_i32_scalar_return_projection(&mut projections, &alias.return_projection);
    }
    for offset in &facts.offsets {
        push_unique_i32_scalar_return_projection(&mut projections, &offset.return_projection);
    }
    for relation in &facts.relations {
        push_unique_i32_scalar_return_projection(
            &mut projections,
            &relation.left_return_projection,
        );
        push_unique_i32_scalar_return_projection(
            &mut projections,
            &relation.right_return_projection,
        );
    }
    for constant in &facts.constants {
        push_unique_i32_scalar_return_projection(&mut projections, &constant.return_projection);
    }
    for condition in &facts.return_conditions {
        push_unique_i32_scalar_return_projection(&mut projections, &condition.return_projection);
    }
    projections
}

pub(super) fn push_unique_i32_scalar_return_projection(
    projections: &mut Vec<Vec<PlaceProjection>>,
    projection: &[PlaceProjection],
) {
    if !projections
        .iter()
        .any(|existing| existing.as_slice() == projection)
    {
        projections.push(projection.to_vec());
    }
}

pub(super) fn merge_i32_scalar_return_fact_paths<T>(
    paths: Vec<Vec<T>>,
    projection_paths: &[Vec<Vec<PlaceProjection>>],
) -> Vec<T>
where
    T: I32ScalarReturnProjectedFact,
{
    if paths.len() == 1 {
        let mut out = Vec::new();
        for fact in paths.into_iter().next().unwrap_or_default() {
            push_unique_i32_scalar_return_fact(&mut out, fact);
        }
        return out;
    }
    let mut candidates = Vec::new();
    for path in &paths {
        for fact in path {
            push_unique_i32_scalar_return_fact(&mut candidates, fact.clone());
        }
    }
    candidates
        .into_iter()
        .filter(|fact| {
            paths
                .iter()
                .zip(projection_paths)
                .all(|(path, projections)| {
                    path.iter().any(|path_fact| path_fact == fact)
                        || projections.iter().any(|projection| {
                            return_projections_target_sibling_variant(
                                fact.return_projection(),
                                projection,
                            )
                        })
                })
        })
        .collect()
}

pub(super) fn merge_i32_scalar_return_relation_paths(
    paths: Vec<Vec<I32ScalarReturnRelation>>,
    projection_paths: &[Vec<Vec<PlaceProjection>>],
) -> Vec<I32ScalarReturnRelation> {
    if paths.len() == 1 {
        let mut out = Vec::new();
        for relation in paths.into_iter().next().unwrap_or_default() {
            push_unique_i32_scalar_return_relation(&mut out, relation);
        }
        return out;
    }
    let mut candidates = Vec::new();
    for path in &paths {
        for relation in path {
            push_unique_i32_scalar_return_relation(&mut candidates, relation.clone());
        }
    }
    candidates
        .into_iter()
        .filter(|relation| {
            paths
                .iter()
                .zip(projection_paths)
                .all(|(path, projections)| {
                    path.iter().any(|path_relation| path_relation == relation)
                        || relation_projection_targets_sibling_variant(relation, projections)
                })
        })
        .collect()
}

fn relation_projection_targets_sibling_variant(
    relation: &I32ScalarReturnRelation,
    projections: &[Vec<PlaceProjection>],
) -> bool {
    projections.iter().any(|projection| {
        return_projections_target_sibling_variant(&relation.left_return_projection, projection)
            || return_projections_target_sibling_variant(
                &relation.right_return_projection,
                projection,
            )
    })
}

fn push_unique_i32_scalar_return_relation(
    relations: &mut Vec<I32ScalarReturnRelation>,
    relation: I32ScalarReturnRelation,
) {
    if !relations.iter().any(|existing| existing == &relation) {
        relations.push(relation);
    }
}

fn push_unique_i32_scalar_return_fact<T>(facts: &mut Vec<T>, fact: T)
where
    T: I32ScalarReturnProjectedFact,
{
    if !facts.iter().any(|existing| existing == &fact) {
        facts.push(fact);
    }
}

pub(super) fn merge_i32_scalar_parameter_condition_paths(
    paths: Vec<Vec<I32ScalarParameterCondition>>,
) -> Vec<I32ScalarParameterCondition> {
    if paths.len() == 1 {
        let mut out = Vec::new();
        for fact in paths.into_iter().next().unwrap_or_default() {
            push_unique_i32_scalar_parameter_condition(&mut out, fact);
        }
        return out;
    }
    let mut out = Vec::new();
    if let Some(first) = paths.first() {
        for fact in first {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.iter().any(|existing| existing == fact))
            {
                push_unique_i32_scalar_parameter_condition(&mut out, fact.clone());
            }
        }
    }
    out
}

fn push_unique_i32_scalar_parameter_condition(
    facts: &mut Vec<I32ScalarParameterCondition>,
    fact: I32ScalarParameterCondition,
) {
    if !facts.iter().any(|existing| existing == &fact) {
        facts.push(fact);
    }
}

fn return_projections_target_sibling_variant(
    left_projection: &[PlaceProjection],
    right_projection: &[PlaceProjection],
) -> bool {
    left_projection
        .iter()
        .zip(right_projection)
        .enumerate()
        .any(|(index, (left, right))| {
            matches!(
                (left, right),
                (
                    PlaceProjection::EnumPayload { variant: left_variant },
                    PlaceProjection::EnumPayload {
                        variant: right_variant
                    },
                ) if left_variant != right_variant
                    && place_projection_prefixes_match(
                        &left_projection[..index],
                        &right_projection[..index],
                    )
            )
        })
}

fn place_projection_prefixes_match(left: &[PlaceProjection], right: &[PlaceProjection]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(left, right)| left == right)
}
