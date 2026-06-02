extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::model::{ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp};
use super::summary_worklist_order::summary_order_from_dependencies;

/// Resource summary の固定点計算で共有する関数依存グラフ。
///
/// Resource static check では raw alias、i32 scalar、raw initialization、
/// collection slot、final initialized check が同じ `ResourceModule` の呼び出し関係を
/// 何度も参照する。各 summary kind が個別に依存関係、逆辺、初期 worklist 順序を
/// 作り直すと、証明内容は変わらないまま初回 compile の固定費だけが増える。
///
/// この構造体は 1 回の Resource static check 内でだけ使う compile-local view である。
/// 永続 cache key には入れず、既存の body hash / source capability policy /
/// typed boundary による stale hit 防止を維持したまま、同じグラフ構築を共有する。
pub(super) struct ResourceSummaryDependencyGraph {
    dependencies: Vec<Vec<usize>>,
    dependents: Vec<Vec<usize>>,
    initial_order: Vec<usize>,
    raw_alias_dependencies: Vec<Vec<usize>>,
    raw_alias_dependents: Vec<Vec<usize>>,
    raw_alias_initial_order: Vec<usize>,
    raw_init_dependencies: Vec<Vec<usize>>,
    raw_init_dependents: Vec<Vec<usize>>,
    raw_init_initial_order: Vec<usize>,
}

impl ResourceSummaryDependencyGraph {
    /// `ResourceModule` から summary kind 非依存の依存関係 view を構築する。
    ///
    /// `dependencies` は caller から callee、`dependents` は callee から caller への
    /// 逆辺である。`initial_order` は callee 側を先に評価しやすい順序で、各
    /// `SummaryWorklist` が同じ初期条件から開始できるように保持する。
    pub(super) fn build(module: &ResourceModule) -> Self {
        let dependencies = build_function_summary_dependencies(module);
        let dependents =
            invert_function_summary_dependencies(module.functions.len(), &dependencies);
        let initial_order = summary_order_from_dependencies(module.functions.len(), &dependencies);
        let raw_alias_dependencies = build_function_value_call_summary_dependencies(module);
        let raw_alias_dependents =
            invert_function_summary_dependencies(module.functions.len(), &raw_alias_dependencies);
        let raw_alias_initial_order =
            summary_order_from_dependencies(module.functions.len(), &raw_alias_dependencies);
        let raw_init_dependencies = build_raw_init_summary_dependencies(module);
        let raw_init_dependents =
            invert_function_summary_dependencies(module.functions.len(), &raw_init_dependencies);
        let raw_init_initial_order =
            summary_order_from_dependencies(module.functions.len(), &raw_init_dependencies);
        Self {
            dependencies,
            dependents,
            initial_order,
            raw_alias_dependencies,
            raw_alias_dependents,
            raw_alias_initial_order,
            raw_init_dependencies,
            raw_init_dependents,
            raw_init_initial_order,
        }
    }

    /// caller index から、その関数が直接参照する callee index の一覧を返す。
    pub(super) fn dependencies(&self) -> &[Vec<usize>] {
        &self.dependencies
    }

    /// callee index から、その関数の summary 更新で再投入が必要な caller index を返す。
    pub(super) fn dependents(&self) -> &[Vec<usize>] {
        &self.dependents
    }

    /// 固定点計算を開始するときの関数順序を返す。
    ///
    /// この順序は正しさの前提ではないが、callee の summary を先に安定させることで
    /// 不要な再投入を減らすための性能上の入力である。
    pub(super) fn initial_order(&self) -> &[usize] {
        &self.initial_order
    }

    /// raw-address alias summary が実際に読む callee summary 用の依存辺を返す。
    ///
    /// raw-address alias summary は direct call と、同じ関数内で indirect call へ流れ得る
    /// function value だけから callee summary を読む。単に function value を作るだけの
    /// facade や constructor helper は raw alias summary を消費しないため、shared graph
    /// から分離した view で固定点探索と dependency closure hash の両方を小さく保つ。
    pub(super) fn raw_alias_dependencies(&self) -> &[Vec<usize>] {
        &self.raw_alias_dependencies
    }

    /// raw-address alias summary 専用依存辺の逆辺を返す。
    pub(super) fn raw_alias_dependents(&self) -> &[Vec<usize>] {
        &self.raw_alias_dependents
    }

    /// raw-address alias summary 専用依存辺から作った初期評価順序を返す。
    pub(super) fn raw_alias_initial_order(&self) -> &[usize] {
        &self.raw_alias_initial_order
    }

    /// raw initialization summary が実際に読む callee summary 用の依存辺を返す。
    ///
    /// raw initialization summary は direct call と、同じ関数内で indirect call へ流れ得る
    /// function value だけから callee summary を読む。単に関数値を作るだけの helper は
    /// raw-init facts を消費しないため、shared graph から分離した view で固定点探索と
    /// dependency closure hash の両方を小さく保つ。
    pub(super) fn raw_init_dependencies(&self) -> &[Vec<usize>] {
        &self.raw_init_dependencies
    }

    /// raw initialization summary 専用依存辺の逆辺を返す。
    pub(super) fn raw_init_dependents(&self) -> &[Vec<usize>] {
        &self.raw_init_dependents
    }

    /// raw initialization summary 専用依存辺から作った初期評価順序を返す。
    pub(super) fn raw_init_initial_order(&self) -> &[usize] {
        &self.raw_init_initial_order
    }
}

pub(super) fn build_function_summary_dependents(module: &ResourceModule) -> Vec<Vec<usize>> {
    let dependencies = build_function_summary_dependencies(module);
    invert_function_summary_dependencies(module.functions.len(), &dependencies)
}

pub(super) fn build_function_summary_dependencies(module: &ResourceModule) -> Vec<Vec<usize>> {
    let mut function_indices = BTreeMap::new();
    for (index, function) in module.functions.iter().enumerate() {
        function_indices.insert(function.name.as_str(), index);
    }

    let mut dependencies = vec![Vec::new(); module.functions.len()];
    for (caller_index, function) in module.functions.iter().enumerate() {
        let mut dependency_names = BTreeSet::new();
        collect_function_summary_dependencies(function, &mut dependency_names);
        for dependency in dependency_names {
            if let Some(dependency_index) = function_indices.get(dependency.as_str()) {
                dependencies[caller_index].push(*dependency_index);
            }
        }
    }
    dependencies
}

fn build_raw_init_summary_dependencies(module: &ResourceModule) -> Vec<Vec<usize>> {
    build_function_value_call_summary_dependencies(module)
}

fn build_function_value_call_summary_dependencies(module: &ResourceModule) -> Vec<Vec<usize>> {
    let mut function_indices = BTreeMap::new();
    for (index, function) in module.functions.iter().enumerate() {
        function_indices.insert(function.name.as_str(), index);
    }

    let mut dependencies = vec![Vec::new(); module.functions.len()];
    for (caller_index, function) in module.functions.iter().enumerate() {
        let mut dependency_names = BTreeSet::new();
        collect_function_value_call_summary_dependencies(function, &mut dependency_names);
        for dependency in dependency_names {
            if let Some(dependency_index) = function_indices.get(dependency.as_str()) {
                dependencies[caller_index].push(*dependency_index);
            }
        }
    }
    dependencies
}

fn invert_function_summary_dependencies(
    function_count: usize,
    dependencies: &[Vec<usize>],
) -> Vec<Vec<usize>> {
    let mut dependents = vec![Vec::new(); function_count];
    for (caller_index, function_dependencies) in dependencies.iter().enumerate() {
        for dependency_index in function_dependencies {
            if let Some(function_dependents) = dependents.get_mut(*dependency_index) {
                function_dependents.push(caller_index);
            }
        }
    }
    dependents
}

fn collect_function_summary_dependencies(function: &ResourceFunction, out: &mut BTreeSet<String>) {
    for block in &function.blocks {
        collect_ops_summary_dependencies(&block.ops, out);
    }
}

fn collect_function_value_call_summary_dependencies(
    function: &ResourceFunction,
    out: &mut BTreeSet<String>,
) {
    let mut function_values = BTreeSet::new();
    let mut has_indirect_call = false;
    for block in &function.blocks {
        collect_ops_function_value_call_summary_dependencies(
            &block.ops,
            out,
            &mut function_values,
            &mut has_indirect_call,
        );
    }
    if has_indirect_call {
        out.extend(function_values);
    }
}

fn collect_ops_summary_dependencies(ops: &[ResourceOp], out: &mut BTreeSet<String>) {
    for op in ops {
        collect_op_summary_dependencies(op, out);
    }
}

fn collect_ops_function_value_call_summary_dependencies(
    ops: &[ResourceOp],
    out: &mut BTreeSet<String>,
    function_values: &mut BTreeSet<String>,
    has_indirect_call: &mut bool,
) {
    for op in ops {
        collect_op_function_value_call_summary_dependencies(
            op,
            out,
            function_values,
            has_indirect_call,
        );
    }
}

fn collect_op_summary_dependencies(op: &ResourceOp, out: &mut BTreeSet<String>) {
    match op {
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            ..
        } => {
            out.insert(name.clone());
        }
        ResourceOp::FunctionValue { identity, .. } => {
            out.insert(identity.symbol().to_string());
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_ops_summary_dependencies(then_ops, out);
            collect_ops_summary_dependencies(else_ops, out);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_ops_summary_dependencies(condition_ops, out);
            collect_ops_summary_dependencies(body_ops, out);
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                collect_ops_summary_dependencies(&arm.ops, out);
            }
        }
        ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. }
        | ResourceOp::Construct { .. } => {}
    }
}

fn collect_op_function_value_call_summary_dependencies(
    op: &ResourceOp,
    out: &mut BTreeSet<String>,
    function_values: &mut BTreeSet<String>,
    has_indirect_call: &mut bool,
) {
    match op {
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            ..
        } => {
            out.insert(name.clone());
        }
        ResourceOp::FunctionValue { identity, .. } => {
            function_values.insert(identity.symbol().to_string());
        }
        ResourceOp::IndirectCall { .. } => {
            *has_indirect_call = true;
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_ops_function_value_call_summary_dependencies(
                then_ops,
                out,
                function_values,
                has_indirect_call,
            );
            collect_ops_function_value_call_summary_dependencies(
                else_ops,
                out,
                function_values,
                has_indirect_call,
            );
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_ops_function_value_call_summary_dependencies(
                condition_ops,
                out,
                function_values,
                has_indirect_call,
            );
            collect_ops_function_value_call_summary_dependencies(
                body_ops,
                out,
                function_values,
                has_indirect_call,
            );
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                collect_ops_function_value_call_summary_dependencies(
                    &arm.ops,
                    out,
                    function_values,
                    has_indirect_call,
                );
            }
        }
        ResourceOp::Call { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. }
        | ResourceOp::Construct { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::ast::Effect;
    use crate::function_identity::FunctionValueIdentity;
    use crate::span::Span;
    use crate::types::TypeId;

    use super::*;
    use crate::resource::model::{
        Place, PlaceRoot, ResourceBlock, ResourceBlockId, ResourceTerminator,
    };

    #[test]
    fn summary_dependents_cover_nested_calls_function_values_and_self_recursion() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops(
                    "caller",
                    vec![
                        call("callee"),
                        ResourceOp::Branch {
                            output: place("branch_out"),
                            condition: place("cond"),
                            condition_fact: None,
                            then_ops: vec![function_value("callback")],
                            then_value: place("then_value"),
                            else_ops: vec![],
                            else_value: place("else_value"),
                            span: Span::dummy(),
                        },
                    ],
                ),
                function_with_ops("callee", vec![]),
                function_with_ops("callback", vec![]),
                function_with_ops("recursive", vec![call("recursive")]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let dependents = build_function_summary_dependents(&module);

        assert_eq!(dependents[0], Vec::<usize>::new());
        assert_eq!(dependents[1], vec![0]);
        assert_eq!(dependents[2], vec![0]);
        assert_eq!(dependents[3], vec![3]);
    }

    #[test]
    fn dependency_graph_reuses_dependency_dependent_and_order_views() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("caller", vec![call("callee")]),
                function_with_ops("callee", vec![call("leaf")]),
                function_with_ops("leaf", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.dependencies(), &[vec![1], vec![2], vec![]]);
        assert_eq!(graph.dependents(), &[vec![], vec![0], vec![1]]);
        assert_eq!(graph.initial_order(), &[2, 1, 0]);
    }

    #[test]
    fn raw_init_dependency_graph_keeps_direct_calls() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("caller", vec![call("callee")]),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.raw_init_dependencies(), &[vec![1], vec![]]);
        assert_eq!(graph.raw_init_dependents(), &[vec![], vec![0]]);
    }

    #[test]
    fn raw_alias_dependency_graph_keeps_direct_calls() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("caller", vec![call("callee")]),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.raw_alias_dependencies(), &[vec![1], vec![]]);
        assert_eq!(graph.raw_alias_dependents(), &[vec![], vec![0]]);
    }

    #[test]
    fn raw_init_dependency_graph_ignores_function_value_without_indirect_call() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("factory", vec![function_value("callee")]),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.dependencies(), &[vec![1], vec![]]);
        assert_eq!(graph.raw_init_dependencies(), &[vec![], vec![]]);
    }

    #[test]
    fn raw_alias_dependency_graph_ignores_function_value_without_indirect_call() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("factory", vec![function_value("callee")]),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.dependencies(), &[vec![1], vec![]]);
        assert_eq!(graph.raw_alias_dependencies(), &[vec![], vec![]]);
    }

    #[test]
    fn raw_init_dependency_graph_keeps_function_value_candidates_when_indirect_call_exists() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops(
                    "caller",
                    vec![function_value("callee"), indirect_call("callback")],
                ),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.raw_init_dependencies(), &[vec![1], vec![]]);
    }

    #[test]
    fn raw_alias_dependency_graph_keeps_function_value_candidates_when_indirect_call_exists() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops(
                    "caller",
                    vec![function_value("callee"), indirect_call("callback")],
                ),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.raw_alias_dependencies(), &[vec![1], vec![]]);
    }

    #[test]
    fn raw_init_dependency_graph_does_not_turn_unknown_indirect_call_into_all_edges() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("caller", vec![indirect_call("callback")]),
                function_with_ops("unrelated", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.raw_init_dependencies(), &[vec![], vec![]]);
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
            effect: super::super::model::EffectOp::Pure,
            span: Span::dummy(),
        }
    }

    fn function_value(name: &str) -> ResourceOp {
        ResourceOp::FunctionValue {
            output: place("function_value"),
            name: name.to_string(),
            identity: FunctionValueIdentity::new(
                name.to_string(),
                None,
                TypeId(0),
                Effect::Pure,
                vec![],
            ),
            value_kind: super::super::model::ResourceFunctionValueKind::Plain,
            effect: super::super::model::EffectOp::Pure,
            span: Span::dummy(),
        }
    }

    fn indirect_call(callee: &str) -> ResourceOp {
        ResourceOp::IndirectCall {
            output: place("indirect_out"),
            callee: place(callee),
            params: vec![],
            result: TypeId(0),
            args: vec![],
            effect: super::super::model::EffectOp::Pure,
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
