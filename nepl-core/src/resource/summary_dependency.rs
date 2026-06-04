extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use super::model::{EffectOp, ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp};
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
    direct_call_dependencies: Vec<Vec<usize>>,
    direct_call_dependents: Vec<Vec<usize>>,
    direct_call_initial_order: Vec<usize>,
    raw_alias_dependencies: Vec<Vec<usize>>,
    raw_alias_dependents: Vec<Vec<usize>>,
    raw_alias_initial_order: Vec<usize>,
    raw_init_dependencies: Vec<Vec<usize>>,
    raw_init_dependents: Vec<Vec<usize>>,
    raw_init_initial_order: Vec<usize>,
    owner_dependents: Vec<Vec<usize>>,
    owner_initial_order: Vec<usize>,
    direct_raw_initialization_summary_ops: Vec<bool>,
    direct_owner_summary_ops: Vec<bool>,
}

#[derive(Default)]
struct ResourceFunctionOpInventory {
    summary_dependency_names: BTreeSet<String>,
    direct_call_dependency_names: BTreeSet<String>,
    function_value_candidate_names: BTreeSet<String>,
    has_indirect_call: bool,
    has_direct_raw_initialization_summary_op: bool,
    has_direct_owner_summary_op: bool,
}

impl ResourceSummaryDependencyGraph {
    /// `ResourceModule` から summary kind 非依存の依存関係 view を構築する。
    ///
    /// `dependencies` は caller から callee、`dependents` は callee から caller への
    /// 逆辺である。`initial_order` は callee 側を先に評価しやすい順序で、各
    /// `SummaryWorklist` が同じ初期条件から開始できるように保持する。
    pub(super) fn build(module: &ResourceModule) -> Self {
        let function_indices = build_function_indices(module);
        let inventories = build_function_op_inventories(module);
        let dependencies = summary_dependency_names_to_indices(
            module.functions.len(),
            &function_indices,
            &inventories,
        );
        let dependents =
            invert_function_summary_dependencies(module.functions.len(), &dependencies);
        let initial_order = summary_order_from_dependencies(module.functions.len(), &dependencies);
        let direct_call_dependencies = direct_call_dependency_names_to_indices(
            module.functions.len(),
            &function_indices,
            &inventories,
        );
        let direct_call_dependents =
            invert_function_summary_dependencies(module.functions.len(), &direct_call_dependencies);
        let direct_call_initial_order =
            summary_order_from_dependencies(module.functions.len(), &direct_call_dependencies);
        let function_value_call_dependencies = function_value_call_dependency_names_to_indices(
            module.functions.len(),
            &function_indices,
            &inventories,
        );
        let raw_alias_dependencies = function_value_call_dependencies.clone();
        let raw_alias_dependents =
            invert_function_summary_dependencies(module.functions.len(), &raw_alias_dependencies);
        let raw_alias_initial_order =
            summary_order_from_dependencies(module.functions.len(), &raw_alias_dependencies);
        let raw_init_dependencies = function_value_call_dependencies;
        let raw_init_dependents =
            invert_function_summary_dependencies(module.functions.len(), &raw_init_dependencies);
        let raw_init_initial_order =
            summary_order_from_dependencies(module.functions.len(), &raw_init_dependencies);
        let owner_dependencies = raw_init_dependencies.clone();
        let owner_dependents =
            invert_function_summary_dependencies(module.functions.len(), &owner_dependencies);
        let owner_initial_order =
            summary_order_from_dependencies(module.functions.len(), &owner_dependencies);
        let direct_raw_initialization_summary_ops = inventories
            .iter()
            .map(|inventory| inventory.has_direct_raw_initialization_summary_op)
            .collect();
        let direct_owner_summary_ops = inventories
            .iter()
            .map(|inventory| inventory.has_direct_owner_summary_op)
            .collect();
        Self {
            dependencies,
            dependents,
            initial_order,
            direct_call_dependencies,
            direct_call_dependents,
            direct_call_initial_order,
            raw_alias_dependencies,
            raw_alias_dependents,
            raw_alias_initial_order,
            raw_init_dependencies,
            raw_init_dependents,
            raw_init_initial_order,
            owner_dependents,
            owner_initial_order,
            direct_raw_initialization_summary_ops,
            direct_owner_summary_ops,
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

    /// direct call だけで読まれる summary 用の依存辺を返す。
    ///
    /// i32 scalar summary は現在 direct call の output / arg 境界でだけ適用される。
    /// function value を生成するだけの helper や indirect call の候補はこの summary を
    /// 消費しないため、固定点探索と dependency closure hash へ入れない。
    pub(super) fn direct_call_dependencies(&self) -> &[Vec<usize>] {
        &self.direct_call_dependencies
    }

    /// direct-call summary 専用依存辺の逆辺を返す。
    pub(super) fn direct_call_dependents(&self) -> &[Vec<usize>] {
        &self.direct_call_dependents
    }

    /// direct-call summary 専用依存辺から作った初期評価順序を返す。
    pub(super) fn direct_call_initial_order(&self) -> &[usize] {
        &self.direct_call_initial_order
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

    /// owner return summary 専用依存辺の逆辺を返す。
    ///
    /// owner summary は direct call と、同じ関数内で indirect call に流れ得る
    /// function value からだけ callee summary を読む。単に function value を作るだけの
    /// facade は owner summary の固定点探索には不要なので、raw-init / raw-alias と
    /// 同じ依存辺 view から作った逆辺で full summary dependency より探索範囲を狭める。
    pub(super) fn owner_dependents(&self) -> &[Vec<usize>] {
        &self.owner_dependents
    }

    /// owner return summary 専用依存辺から作った初期評価順序を返す。
    pub(super) fn owner_initial_order(&self) -> &[usize] {
        &self.owner_initial_order
    }

    /// 関数内に raw initialization summary の起点になる operation があるかを返す。
    ///
    /// raw-init relevance は signature / raw alias summary / callee 伝播とは別に、
    /// 関数自身が raw memory、collection slot、indirect call などの summary 起点を
    /// 持つかを見る。依存辺構築時に同じ op tree を既に走査しているため、この結果を
    /// graph に保持し、summary kind ごとの再走査を避ける。
    pub(super) fn has_direct_raw_initialization_summary_op(&self, function_index: usize) -> bool {
        self.direct_raw_initialization_summary_ops
            .get(function_index)
            .copied()
            .unwrap_or(false)
    }

    /// 関数内に owner return summary の起点になる operation があるかを返す。
    ///
    /// owner summary relevance は公開 signature の owner leaf と、raw memory /
    /// raw address / storage origin / indirect call / non-pure call のような直接の
    /// owner proof 起点から決まる。依存辺構築時に同じ op tree を既に走査しているため、
    /// op 由来の relevance を graph に保持して、owner summary stage での再走査を避ける。
    pub(super) fn has_direct_owner_summary_op(&self, function_index: usize) -> bool {
        self.direct_owner_summary_ops
            .get(function_index)
            .copied()
            .unwrap_or(false)
    }
}

pub(super) fn build_function_summary_dependents(module: &ResourceModule) -> Vec<Vec<usize>> {
    let dependencies = build_function_summary_dependencies(module);
    invert_function_summary_dependencies(module.functions.len(), &dependencies)
}

pub(super) fn build_function_summary_dependencies(module: &ResourceModule) -> Vec<Vec<usize>> {
    let function_indices = build_function_indices(module);
    let inventories = build_function_op_inventories(module);
    summary_dependency_names_to_indices(module.functions.len(), &function_indices, &inventories)
}

fn build_function_indices(module: &ResourceModule) -> BTreeMap<&str, usize> {
    let mut function_indices = BTreeMap::new();
    for (index, function) in module.functions.iter().enumerate() {
        function_indices.insert(function.name.as_str(), index);
    }
    function_indices
}

fn build_function_op_inventories(module: &ResourceModule) -> Vec<ResourceFunctionOpInventory> {
    module
        .functions
        .iter()
        .map(collect_function_op_inventory)
        .collect()
}

fn summary_dependency_names_to_indices(
    function_count: usize,
    function_indices: &BTreeMap<&str, usize>,
    inventories: &[ResourceFunctionOpInventory],
) -> Vec<Vec<usize>> {
    let mut dependencies = vec![Vec::new(); function_count];
    for (caller_index, inventory) in inventories.iter().enumerate() {
        push_dependency_indices(
            &mut dependencies[caller_index],
            function_indices,
            inventory
                .summary_dependency_names
                .iter()
                .map(String::as_str),
        );
    }
    dependencies
}

fn function_value_call_dependency_names_to_indices(
    function_count: usize,
    function_indices: &BTreeMap<&str, usize>,
    inventories: &[ResourceFunctionOpInventory],
) -> Vec<Vec<usize>> {
    let mut dependencies = vec![Vec::new(); function_count];
    for (caller_index, inventory) in inventories.iter().enumerate() {
        push_dependency_indices(
            &mut dependencies[caller_index],
            function_indices,
            inventory
                .direct_call_dependency_names
                .iter()
                .map(String::as_str),
        );
        if inventory.has_indirect_call {
            push_dependency_indices(
                &mut dependencies[caller_index],
                function_indices,
                inventory
                    .function_value_candidate_names
                    .iter()
                    .map(String::as_str),
            );
        }
    }
    dependencies
}

fn direct_call_dependency_names_to_indices(
    function_count: usize,
    function_indices: &BTreeMap<&str, usize>,
    inventories: &[ResourceFunctionOpInventory],
) -> Vec<Vec<usize>> {
    let mut dependencies = vec![Vec::new(); function_count];
    for (caller_index, inventory) in inventories.iter().enumerate() {
        push_dependency_indices(
            &mut dependencies[caller_index],
            function_indices,
            inventory
                .direct_call_dependency_names
                .iter()
                .map(String::as_str),
        );
    }
    dependencies
}

fn push_dependency_indices<'a>(
    out: &mut Vec<usize>,
    function_indices: &BTreeMap<&str, usize>,
    names: impl Iterator<Item = &'a str>,
) {
    for dependency in names {
        if let Some(dependency_index) = function_indices.get(dependency) {
            out.push(*dependency_index);
        }
    }
}

fn collect_function_op_inventory(function: &ResourceFunction) -> ResourceFunctionOpInventory {
    let mut inventory = ResourceFunctionOpInventory::default();
    for block in &function.blocks {
        collect_ops_op_inventory(&block.ops, &mut inventory);
    }
    inventory
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

fn collect_ops_op_inventory(ops: &[ResourceOp], inventory: &mut ResourceFunctionOpInventory) {
    for op in ops {
        collect_op_inventory(op, inventory);
    }
}

fn collect_op_inventory(op: &ResourceOp, inventory: &mut ResourceFunctionOpInventory) {
    match op {
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            effect,
            ..
        } => {
            inventory.summary_dependency_names.insert(name.clone());
            if direct_call_reads_i32_scalar_summary(effect) {
                inventory.direct_call_dependency_names.insert(name.clone());
            }
            if call_directly_affects_owner_summary(effect) {
                inventory.has_direct_owner_summary_op = true;
            }
        }
        ResourceOp::Call { effect, .. } => {
            if call_directly_affects_owner_summary(effect) {
                inventory.has_direct_owner_summary_op = true;
            }
        }
        ResourceOp::FunctionValue { identity, .. } => {
            let symbol = identity.symbol().to_string();
            inventory.summary_dependency_names.insert(symbol.clone());
            inventory.function_value_candidate_names.insert(symbol);
        }
        ResourceOp::IndirectCall { .. } => {
            inventory.has_indirect_call = true;
            inventory.has_direct_raw_initialization_summary_op = true;
            inventory.has_direct_owner_summary_op = true;
        }
        ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. } => {
            inventory.has_direct_raw_initialization_summary_op = true;
            inventory.has_direct_owner_summary_op = true;
        }
        ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. } => {
            inventory.has_direct_raw_initialization_summary_op = true;
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_ops_op_inventory(then_ops, inventory);
            collect_ops_op_inventory(else_ops, inventory);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_ops_op_inventory(condition_ops, inventory);
            collect_ops_op_inventory(body_ops, inventory);
        }
        ResourceOp::Match { arms, .. } => {
            for arm in arms {
                collect_ops_op_inventory(&arm.ops, inventory);
            }
        }
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::Construct { .. } => {}
    }
}

fn direct_call_reads_i32_scalar_summary(effect: &EffectOp) -> bool {
    // The i32 scalar summary propagator intentionally does not replay callee scalar facts across
    // raw-memory helper calls. Keeping those calls in the i32 dependency graph makes the fixed
    // point walk functions whose summaries can never be consumed at that call boundary.
    !matches!(
        effect,
        EffectOp::InternalAlloc { .. } | EffectOp::UnsafeMemory { .. }
    )
}

fn call_directly_affects_owner_summary(effect: &EffectOp) -> bool {
    !effect.is_proof_pure()
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
        Place, PlaceRoot, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceTerminator,
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
    fn direct_call_dependency_graph_keeps_direct_calls() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops("caller", vec![call("callee")]),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.direct_call_dependencies(), &[vec![1], vec![]]);
        assert_eq!(graph.direct_call_dependents(), &[vec![], vec![0]]);
    }

    #[test]
    fn direct_call_dependency_graph_ignores_calls_that_cannot_read_i32_scalar_summaries() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops(
                    "caller",
                    vec![call_with_effect(
                        "alloc_raw",
                        EffectOp::InternalAlloc {
                            operation: RawMemoryOp::Alloc,
                        },
                    )],
                ),
                function_with_ops("alloc_raw", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert_eq!(graph.dependencies(), &[vec![1], vec![]]);
        assert_eq!(graph.direct_call_dependencies(), &[vec![], vec![]]);
    }

    #[test]
    fn owner_summary_inventory_treats_pure_user_calls_as_proof_pure() {
        let module = ResourceModule {
            functions: vec![
                function_with_ops(
                    "pure_caller",
                    vec![call_with_effect(
                        "callee",
                        EffectOp::UserCall {
                            name: "callee".into(),
                            effect: Effect::Pure,
                        },
                    )],
                ),
                function_with_ops(
                    "impure_caller",
                    vec![call_with_effect(
                        "callee",
                        EffectOp::UserCall {
                            name: "callee".into(),
                            effect: Effect::Impure,
                        },
                    )],
                ),
                function_with_ops("callee", vec![]),
            ],
            entry: None,
            string_literals: vec![],
        };

        let graph = ResourceSummaryDependencyGraph::build(&module);

        assert!(!graph.has_direct_owner_summary_op(0));
        assert!(graph.has_direct_owner_summary_op(1));
        assert!(!graph.has_direct_owner_summary_op(2));
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
        assert_eq!(graph.owner_dependents(), &[vec![], vec![0]]);
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
        assert_eq!(graph.owner_dependents(), &[vec![], vec![]]);
    }

    #[test]
    fn direct_call_dependency_graph_ignores_function_value_without_call() {
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
        assert_eq!(graph.direct_call_dependencies(), &[vec![], vec![]]);
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
        assert_eq!(graph.owner_dependents(), &[vec![], vec![0]]);
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
    fn direct_call_dependency_graph_ignores_indirect_function_value_candidates() {
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
        assert_eq!(graph.direct_call_dependencies(), &[vec![], vec![]]);
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
        call_with_effect(name, super::super::model::EffectOp::Pure)
    }

    fn call_with_effect(name: &str, effect: EffectOp) -> ResourceOp {
        ResourceOp::Call {
            output: place("call_out"),
            target: ResourceCallTarget::User {
                name: name.to_string(),
                type_args: vec![],
            },
            args: vec![],
            effect,
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
