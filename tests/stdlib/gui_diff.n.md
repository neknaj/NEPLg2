# alloc/gui diff

このファイルは retained `ViewTree` の差分が platform handle ではなく `WidgetDescriptor` の data と `GuiInvalidation` enum で表されることを固定します。

## view_tree_diff_detects_widget_content_change

[目的/もくてき]:
- root id が同じでも label text が[変/か]われば root slot が changed になることを確認します。
- invalidation は string や silent bool ではなく `GuiInvalidation::Widget id` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let root_id %WidgetId widget_id 1
    let hint %LayoutHint layout_hint_fixed 4 1
    let old_root %WidgetDescriptor widget_label root_id "old" hint
    let new_root %WidgetDescriptor widget_label root_id "new" hint
    let old_tree %ViewTree view_tree_single old_root
    let new_tree %ViewTree view_tree_single new_root
    let diff %ViewTreeDiff view_tree_diff &old_tree &new_tree
    let changed_check assert view_tree_diff_root_changed &diff
    let invalidation %GuiInvalidation view_tree_invalidation &old_tree &new_tree
    let invalidation_check match gui_invalidation_widget_id invalidation:
        Option::Some id:
            assert_eq_i32 1 widget_id_value id
        Option::None:
            assert false
    let checks1 checks_push checks_new changed_check
    let checks checks_push checks1 invalidation_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## view_tree_diff_shape_change_invalidates_tree

[目的/もくてき]:
- child 追加のような tree shape 変更は、個別 widget ではなく tree invalidation になることを確認します。

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let root_id %WidgetId widget_id 1
    let child_id %WidgetId widget_id 2
    let child_action %ActionId action_id 3
    let hint %LayoutHint layout_hint_fixed 4 1
    let child_config %ButtonConfig button_config child_id "Run" child_action
    let root %WidgetDescriptor widget_label root_id "root" hint
    let child %WidgetDescriptor widget_button child_config hint
    let old_tree %ViewTree view_tree_single root
    let new_tree %ViewTree unwrap_ok view_tree_add_child old_tree child
    match view_tree_invalidation &old_tree &new_tree:
        GuiInvalidation::Tree:
            2
        _:
            9
```

## view_tree_diff_child_content_change_returns_child_id

[目的/もくてき]:
- child slot の button action が[変/か]わった場合、child の `WidgetId` を invalidation 対象にすることを確認します。

neplg2:test
ret: 2
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let root_id %WidgetId widget_id 1
    let child_id %WidgetId widget_id 2
    let old_action %ActionId action_id 3
    let new_action %ActionId action_id 4
    let hint %LayoutHint layout_hint_fixed 4 1
    let old_config %ButtonConfig button_config child_id "Run" old_action
    let new_config %ButtonConfig button_config child_id "Run" new_action
    let root %WidgetDescriptor widget_label root_id "root" hint
    let child_old %WidgetDescriptor widget_button old_config hint
    let child_new %WidgetDescriptor widget_button new_config hint
    let old_root_tree %ViewTree view_tree_single root
    let new_root_tree %ViewTree view_tree_single root
    let old_tree %ViewTree unwrap_ok view_tree_add_child old_root_tree child_old
    let new_tree %ViewTree unwrap_ok view_tree_add_child new_root_tree child_new
    let invalidation %GuiInvalidation view_tree_invalidation &old_tree &new_tree
    match gui_invalidation_widget_id invalidation:
        Option::Some id:
            widget_id_value id
        Option::None:
            9
```
