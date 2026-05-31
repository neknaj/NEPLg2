use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::resource::model::{
    EffectOp, Place, PlaceRoot, ResourceBlock, ResourceBlockId, ResourceCallTarget,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use crate::span::Span;
use crate::types::TypeId;

use super::super::summary_dependency::ResourceSummaryDependencyGraph;
use super::super::summary_worklist_order::initial_summary_order;
use super::*;

#[test]
fn initial_summary_order_places_callees_before_callers() {
    let module = ResourceModule {
        functions: vec![
            function_with_ops("caller", vec![call("callee")]),
            function_with_ops("callee", vec![call("leaf")]),
            function_with_ops("leaf", vec![]),
            function_with_ops("recursive", vec![call("recursive")]),
        ],
        entry: None,
        string_literals: vec![],
    };

    let order = initial_summary_order(&module);

    assert_before(&order, 2, 1);
    assert_before(&order, 1, 0);
    assert_eq!(order.len(), 4);
    assert_eq!(order.iter().filter(|index| **index == 3).count(), 1);
}

#[test]
fn filtered_summary_worklist_initially_queues_only_relevant_functions() {
    let module = ResourceModule {
        functions: vec![
            function_with_ops("caller", vec![call("callee")]),
            function_with_ops("callee", vec![call("leaf")]),
            function_with_ops("leaf", vec![]),
            function_with_ops("unrelated", vec![]),
        ],
        entry: None,
        string_literals: vec![],
    };
    let mut worklist = SummaryWorklist::new_filtered(&module, vec![true, false, true, false]);

    assert_eq!(drain_worklist(&mut worklist), vec![2, 0]);
}

#[test]
fn dependency_graph_worklist_matches_default_order_and_dependents() {
    let module = ResourceModule {
        functions: vec![
            function_with_ops("caller", vec![call("callee")]),
            function_with_ops("callee", vec![call("leaf")]),
            function_with_ops("leaf", vec![]),
            function_with_ops("unrelated", vec![]),
        ],
        entry: None,
        string_literals: vec![],
    };
    let graph = ResourceSummaryDependencyGraph::build(&module);
    let mut default_worklist =
        SummaryWorklist::new_filtered(&module, vec![true, false, true, false]);
    let mut graph_worklist = SummaryWorklist::new_filtered_with_dependency_graph(
        &module,
        vec![true, false, true, false],
        &graph,
    );

    assert_eq!(
        drain_worklist(&mut graph_worklist),
        drain_worklist(&mut default_worklist)
    );
}

#[test]
fn filtered_summary_worklist_notify_changed_skips_irrelevant_dependents() {
    let module = ResourceModule {
        functions: vec![
            function_with_ops("relevant_caller", vec![call("callee")]),
            function_with_ops("callee", vec![]),
            function_with_ops("irrelevant_caller", vec![call("callee")]),
        ],
        entry: None,
        string_literals: vec![],
    };
    let mut worklist = SummaryWorklist::new_filtered(&module, vec![true, true, false]);

    assert_eq!(worklist.pop(), Some(1));
    assert_eq!(worklist.pop(), Some(0));
    worklist.notify_changed(1);

    assert_eq!(drain_worklist(&mut worklist), vec![0]);
}

#[test]
fn unrecomputed_initial_skips_keep_only_entries_that_never_reentered_worklist() {
    let module = ResourceModule {
        functions: vec![
            function_with_ops("caller", vec![call("callee")]),
            function_with_ops("callee", vec![]),
            function_with_ops("independent", vec![]),
        ],
        entry: None,
        string_literals: vec![],
    };
    let mut worklist = SummaryWorklist::new_filtered_with_initial_skips(
        &module,
        vec![true, true, true],
        vec![true, false, true],
    );

    assert_eq!(worklist.pop(), Some(1));
    worklist.notify_changed(1);
    assert_eq!(worklist.pop(), Some(0));

    assert_eq!(
        worklist.unrecomputed_initial_skips(&[true, false, true]),
        vec![false, false, true]
    );
}

fn assert_before(order: &[usize], left: usize, right: usize) {
    let left_pos = order.iter().position(|index| *index == left).unwrap();
    let right_pos = order.iter().position(|index| *index == right).unwrap();
    assert!(left_pos < right_pos);
}

fn drain_worklist(worklist: &mut SummaryWorklist) -> Vec<usize> {
    let mut out = Vec::new();
    while let Some(index) = worklist.pop() {
        out.push(index);
    }
    out
}

fn function_with_ops(name: &str, ops: Vec<ResourceOp>) -> ResourceFunction {
    ResourceFunction {
        name: name.to_string(),
        origin_name: name.to_string(),
        type_params: vec![],
        params: vec![],
        result: TypeId(0),
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops,
            terminator: ResourceTerminator::Return {
                value: None,
                span: Span::dummy(),
            },
            span: Span::dummy(),
        }],
        span: Span::dummy(),
    }
}

fn call(name: &str) -> ResourceOp {
    ResourceOp::Call {
        output: place("call_out"),
        target: ResourceCallTarget::User {
            name: name.to_string(),
            type_args: vec![],
        },
        args: vec![],
        effect: EffectOp::Pure,
        span: Span::dummy(),
    }
}

fn place(name: &str) -> Place {
    Place {
        root: PlaceRoot::Local(name.to_string()),
        projections: vec![],
        ty: TypeId(0),
    }
}
