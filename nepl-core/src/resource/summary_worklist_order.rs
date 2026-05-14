extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::model::ResourceModule;
use super::summary_dependency::build_function_summary_dependencies;

pub(super) fn initial_summary_order(module: &ResourceModule) -> Vec<usize> {
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
