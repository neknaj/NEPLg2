# alloc/gui tree

このファイルは retained `ViewTree` / `LayoutTree` が platform handle を持たず、`Result` と `Option` による bounded contract で扱えることを固定します。

## view_tree_tracks_focusable_widget_order

[目的/もくてき]:
- label は focus target ではなく、button だけが focus traversal に入ることを確認します。
- focus target は callback ではなく `WidgetId` として返ることを固定します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "title" (layout_hint_fixed 8 1)
    let button_node %WidgetDescriptor widget_button (button_config (widget_id 7) "Run" (action_id 2)) (layout_hint_fixed 6 1)
    let tree %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) button_node
    let count_check assert_eq_i32 1 view_tree_focusable_count &tree
    let first_check match view_tree_first_focusable_id &tree:
        Option::Some id:
            assert_eq_i32 7 widget_id_value id
        Option::None:
            assert false
    let checks checks_push (checks_push checks_new count_check) first_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## view_tree_capacity_overflow_is_result_error

[目的/もくてき]:
- bounded tree の capacity overflow が panic や silent no-op ではなく `GuiError::ResourceExhausted` になることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let root %WidgetDescriptor widget_label (widget_id 1) "root" (layout_hint_fixed 8 1)
    let c1 %WidgetDescriptor widget_button (button_config (widget_id 2) "A" (action_id 2)) (layout_hint_fixed 4 1)
    let c2 %WidgetDescriptor widget_button (button_config (widget_id 3) "B" (action_id 3)) (layout_hint_fixed 4 1)
    let c3 %WidgetDescriptor widget_button (button_config (widget_id 4) "C" (action_id 4)) (layout_hint_fixed 4 1)
    let tree1 %ViewTree unwrap_ok view_tree_add_child (view_tree_single root) c1
    let tree2 %ViewTree unwrap_ok view_tree_add_child tree1 c2
    match view_tree_add_child tree2 c3:
        Result::Ok _tree:
            1
        Result::Err error:
            match error:
                GuiError::ResourceExhausted:
                    0
                _:
                    2
```

## layout_tree_tracks_layout_node_children

[目的/もくてき]:
- `LayoutTree` が placement result を `Option` child として保持できることを確認します。

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let root %LayoutNode layout_node (widget_id 1) (gui_rect_new 0 0 10 2)
    let child %LayoutNode layout_node (widget_id 2) (gui_rect_new 0 2 10 2)
    match layout_tree_add_child (layout_tree_single root) child:
        Result::Ok tree:
            layout_tree_child_count &tree
        Result::Err _error:
            9
```
