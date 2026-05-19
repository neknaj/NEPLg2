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

fn assert_before(order: &[usize], left: usize, right: usize) {
    let left_pos = order.iter().position(|index| *index == left).unwrap();
    let right_pos = order.iter().position(|index| *index == right).unwrap();
    assert!(left_pos < right_pos);
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
