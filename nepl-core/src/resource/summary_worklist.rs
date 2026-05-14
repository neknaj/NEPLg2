extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use super::model::ResourceModule;
use super::summary_dependency::{
    build_function_summary_dependencies, build_function_summary_dependents,
};

pub(super) struct SummaryWorklist {
    dependents: Vec<Vec<usize>>,
    pending: VecDeque<usize>,
    queued: Vec<bool>,
    max_recomputations: usize,
    recomputations: usize,
}

impl SummaryWorklist {
    pub(super) fn new(module: &ResourceModule) -> Self {
        let mut pending = VecDeque::new();
        let mut queued = vec![false; module.functions.len()];
        for index in initial_summary_order(module) {
            pending.push_back(index);
            queued[index] = true;
        }
        let max_recomputations = module
            .functions
            .len()
            .saturating_mul(module.functions.len().saturating_add(1));
        Self {
            dependents: build_function_summary_dependents(module),
            pending,
            queued,
            max_recomputations,
            recomputations: 0,
        }
    }

    pub(super) fn pop(&mut self) -> Option<usize> {
        if self.recomputations >= self.max_recomputations {
            return None;
        }
        let index = self.pending.pop_front()?;
        self.queued[index] = false;
        self.recomputations += 1;
        Some(index)
    }

    pub(super) fn notify_changed(&mut self, function_index: usize) {
        for dependent in &self.dependents[function_index] {
            if !self.queued[*dependent] {
                self.pending.push_back(*dependent);
                self.queued[*dependent] = true;
            }
        }
    }

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    pub(super) fn recomputations(&self) -> usize {
        self.recomputations
    }
}

fn initial_summary_order(module: &ResourceModule) -> Vec<usize> {
    let dependencies = build_function_summary_dependencies(module);
    let mut marks = vec![SummaryOrderMark::Unvisited; module.functions.len()];
    let mut out = Vec::new();
    for index in 0..module.functions.len() {
        push_summary_order(index, &dependencies, &mut marks, &mut out);
    }
    out
}

fn push_summary_order(
    index: usize,
    dependencies: &[Vec<usize>],
    marks: &mut [SummaryOrderMark],
    out: &mut Vec<usize>,
) {
    match marks[index] {
        SummaryOrderMark::Done | SummaryOrderMark::Visiting => return,
        SummaryOrderMark::Unvisited => {}
    }
    marks[index] = SummaryOrderMark::Visiting;
    for dependency in &dependencies[index] {
        push_summary_order(*dependency, dependencies, marks, out);
    }
    marks[index] = SummaryOrderMark::Done;
    out.push(index);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SummaryOrderMark {
    Unvisited,
    Visiting,
    Done,
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::TypeId;

    use super::*;
    use crate::resource::model::{
        EffectOp, Place, PlaceRoot, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceOp,
        ResourceTerminator,
    };

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

    fn function_with_ops(
        name: &str,
        ops: Vec<ResourceOp>,
    ) -> super::super::model::ResourceFunction {
        super::super::model::ResourceFunction {
            name: name.to_string(),
            origin_name: name.to_string(),
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
}
