# alloc/gui accessibility semantic tree

このファイルは `alloc/gui/accessibility` が visual tree / draw command とは別の semantic tree として使えることを固定します。

## accessibility_tree_keeps_semantics_separate_from_draw_commands

[目的/もくてき]:
- accessibility tree が node id、role、label、state、action の data contract だけで構築できることを確認します。
- platform handle、std/gui host、draw command を使わずに semantic snapshot を検査します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/accessibility" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let root_id %SemanticNodeId semantic_node_id 1
    let run_id %SemanticNodeId semantic_node_id 2
    let title_id %SemanticNodeId semantic_node_id 3
    let run_state %SemanticState semantic_state true false false
    let root %SemanticNode semantic_node root_id SemanticRole::Window "window" semantic_state_default SemanticAction::None
    let run_button %SemanticNode semantic_node run_id SemanticRole::Button "Run" run_state SemanticAction::Activate
    let title %SemanticNode semantic_node title_id SemanticRole::Label "Title" semantic_state_default SemanticAction::None
    let tree0 %AccessibilityTree accessibility_tree_single root
    let tree1_result %Result AccessibilityTree AccessibilityTreeError accessibility_tree_add_child tree0 run_button
    let checks match tree1_result:
        Result::Ok tree1:
            match accessibility_tree_add_child tree1 title:
                Result::Ok tree2:
                    let check0 assert_eq_i32 2 accessibility_tree_child_count &tree2
                    let check1 match accessibility_tree_first_child &tree2:
                        Option::Some node:
                            assert semantic_node_is_button &node
                        Option::None:
                            assert false
                    let check2 assert_eq_i32 1 semantic_node_id_raw &semantic_node_id 1
                    checks_new
                    |> checks_push check0
                    |> checks_push check1
                    |> checks_push check2
                Result::Err _error:
                    checks_push checks_new assert false
        Result::Err _error:
            checks_push checks_new assert false
    let shown checks_print_report checks
    checks_exit_code shown
```

## accessibility_tree_capacity_failure_is_result

[目的/もくてき]:
- bounded tree の capacity overflow が silent no-op や panic ではなく `Result::Err` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/accessibility" as *
#import "core/result" as *
#import "std/test" as *

fn small_node %fn i32 SemanticNode \id:
    let node_id %SemanticNodeId semantic_node_id id
    semantic_node node_id SemanticRole::Label "node" semantic_state_default SemanticAction::None

fn main %impure fn void i32 \void:
    let tree0 %AccessibilityTree accessibility_tree_single small_node 1
    let tree1 %AccessibilityTree unwrap_ok accessibility_tree_add_child tree0 small_node 2
    let tree2 %AccessibilityTree unwrap_ok accessibility_tree_add_child tree1 small_node 3
    let check match accessibility_tree_add_child tree2 small_node 4:
        Result::Ok _tree:
            assert false
        Result::Err error:
            match error:
                AccessibilityTreeError::CapacityExceeded:
                    assert true
    let checks checks_push checks_new check
    let shown checks_print_report checks
    checks_exit_code shown
```
