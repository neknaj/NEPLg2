use alloc::string::String;
use alloc::vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::TypeCtx;

use super::super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::super::model::{
    Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator,
};
use super::super::summary_dependency::ResourceSummaryDependencyGraph;
use super::*;

/// collection slot helper は raw memory op を直接持たない場合でも、
/// call 境界で raw initialization summary と同じ initialized-cell facts を運ぶ。
/// relevance pruning がこれを落とすと、helper 経由の slot 初期化証明が消える。
#[test]
fn collection_slot_ops_are_raw_initialization_summary_triggers() {
    let types = TypeCtx::new();
    let unit = types.unit();
    let slot = Place::local(String::from("slot"), unit);
    let function = ResourceFunction {
        name: String::from("slot_helper"),
        origin_name: String::from("slot_helper"),
        type_params: vec![],
        params: vec![],
        result: unit,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![ResourceOp::CollectionSlotLifecycle {
                target: slot,
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: unit },
                span: Span::dummy(),
            }],
            terminator: ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    };

    let module = ResourceModule {
        functions: vec![function],
        entry: None,
        string_literals: vec![],
    };
    let dependency_graph = ResourceSummaryDependencyGraph::build(&module);

    assert!(
        dependency_graph.has_direct_raw_initialization_summary_op(0),
        "collection slot marker だけを持つ helper も raw initialization summary worklist の seed である必要がある"
    );
}

/// variant-param summary の前提判定は、collector が実際に読む
/// top-level Branch の output と return value の一致だけを見る。
/// 一致しない Branch を候補に含めると、variant-param facts を作れない block でも
/// ResourceCheckEngine の prefix replay が起動し、性能劣化だけが残る。
#[test]
fn variant_param_summary_scan_skips_without_return_branch_output() {
    let types = TypeCtx::new();
    let unit = types.unit();
    let return_value = Place::local(String::from("ret"), unit);
    let other_value = Place::local(String::from("other"), unit);
    let condition = Place::local(String::from("cond"), unit);

    assert!(!ops_have_top_level_branch_output_for_return(
        &[],
        &return_value
    ));

    let ops = vec![ResourceOp::Branch {
        output: other_value.clone(),
        condition,
        condition_fact: None,
        then_ops: vec![],
        then_value: other_value.clone(),
        else_ops: vec![],
        else_value: other_value,
        span: Span::dummy(),
    }];

    assert!(!ops_have_top_level_branch_output_for_return(
        &ops,
        &return_value
    ));
}

/// return value を直接作る top-level Branch は、variant-param summary が
/// 分岐ごとの param-cell facts を回収するための入口である。
/// この入口を保つことで、性能最適化が既存の variant 証明能力を削らないことを確認する。
#[test]
fn variant_param_summary_scan_detects_return_branch_output() {
    let types = TypeCtx::new();
    let unit = types.unit();
    let return_value = Place::local(String::from("ret"), unit);
    let condition = Place::local(String::from("cond"), unit);
    let then_value = Place::local(String::from("then"), unit);
    let else_value = Place::local(String::from("else"), unit);
    let ops = vec![ResourceOp::Branch {
        output: return_value.clone(),
        condition,
        condition_fact: None,
        then_ops: vec![],
        then_value,
        else_ops: vec![],
        else_value,
        span: Span::dummy(),
    }];

    assert!(ops_have_top_level_branch_output_for_return(
        &ops,
        &return_value
    ));
}
