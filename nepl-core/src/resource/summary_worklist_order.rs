extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::model::ResourceModule;
use super::summary_dependency::build_function_summary_dependencies;

pub(super) fn initial_summary_order(module: &ResourceModule) -> Vec<usize> {
    let dependencies = build_function_summary_dependencies(module);
    summary_order_from_dependencies(module.functions.len(), &dependencies)
}

pub(super) fn summary_order_from_dependencies(
    function_count: usize,
    dependencies: &[Vec<usize>],
) -> Vec<usize> {
    let mut marks = vec![SummaryOrderMark::Unvisited; function_count];
    let mut out = Vec::new();
    for index in 0..function_count {
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
