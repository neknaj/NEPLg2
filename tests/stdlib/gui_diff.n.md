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

fn main %impure fn void i32 \void:
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

fn main %fn void i32 \void:
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

fn main %fn void i32 \void:
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

## view_tree_arena_invalidation_returns_nested_widget_id

[目的/もくてき]:
- allocator-backed `ViewTreeArena` で nested widget の内容だけが変わった場合、tree 全体ではなくその `WidgetId` を invalidation 対象にすることを確認します。
- arena owner を消費せず borrow-only で diff できることを確認します。

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let parent_id %WidgetId widget_id 2
    let nested_id %WidgetId widget_id 3
    let parent_action %ActionId action_id 20
    let old_nested_action %ActionId action_id 30
    let new_nested_action %ActionId action_id 31
    let root_hint %LayoutHint layout_hint_fixed 10 4
    let button_hint %LayoutHint layout_hint_fixed 4 2
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let parent_config %ButtonConfig button_config parent_id "Parent" parent_action
    let parent %WidgetDescriptor widget_button parent_config button_hint
    let old_nested_config %ButtonConfig button_config nested_id "Nested" old_nested_action
    let new_nested_config %ButtonConfig button_config nested_id "Nested" new_nested_action
    let old_nested %WidgetDescriptor widget_button old_nested_config button_hint
    let new_nested %WidgetDescriptor widget_button new_nested_config button_hint
    let old0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let old1 %ViewTreeArena unwrap_ok view_tree_arena_add_child old0 0 parent
    let old2 %ViewTreeArena unwrap_ok view_tree_arena_add_child old1 1 old_nested
    let new0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let new1 %ViewTreeArena unwrap_ok view_tree_arena_add_child new0 0 parent
    let new2 %ViewTreeArena unwrap_ok view_tree_arena_add_child new1 1 new_nested
    let diff %ViewTreeArenaDiff view_tree_arena_diff &old2 &new2
    let diff_any %bool view_tree_arena_diff_any &diff
    let changed_count %i32 view_tree_arena_diff_changed_widget_count &diff
    let diff_id %i32:
        match view_tree_arena_diff_changed_widget_id &diff:
            Option::Some id:
                widget_id_value id
            Option::None:
                0
    let invalidation %GuiInvalidation view_tree_arena_invalidation_from_diff &diff
    let invalidation_id %i32:
        match gui_invalidation_widget_id invalidation:
            Option::Some id:
                widget_id_value id
            Option::None:
                0
    view_tree_arena_free old2
    view_tree_arena_free new2
    let count_ok %bool eq changed_count 1
    let diff_id_ok %bool eq diff_id 3
    let invalidation_id_ok %bool eq invalidation_id 3
    let all_ok %bool and:
        and diff_any count_ok
        and diff_id_ok invalidation_id_ok
    if all_ok:
        then 3
        else 9
```

## view_tree_arena_invalidation_tree_for_shape_or_multiple_changes

[目的/もくてき]:
- arena node 数が同じでも parent index / depth が変わる場合は `GuiInvalidation::Tree` になることを確認します。
- content change が複数ある場合、単一 widget id では表せないため `GuiInvalidation::Tree` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *
#import "std/test" as *

fn invalidation_is_tree %fn GuiInvalidation bool \value:
    match value:
        GuiInvalidation::Tree:
            true
        _:
            false

fn main %impure fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let child_id %WidgetId widget_id 2
    let nested_id %WidgetId widget_id 3
    let child_action %ActionId action_id 20
    let nested_action %ActionId action_id 30
    let changed_child_action %ActionId action_id 21
    let changed_nested_action %ActionId action_id 31
    let root_hint %LayoutHint layout_hint_fixed 10 4
    let button_hint %LayoutHint layout_hint_fixed 4 2
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let child_config %ButtonConfig button_config child_id "Child" child_action
    let child %WidgetDescriptor widget_button child_config button_hint
    let nested_config %ButtonConfig button_config nested_id "Nested" nested_action
    let nested %WidgetDescriptor widget_button nested_config button_hint
    let old0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let old1 %ViewTreeArena unwrap_ok view_tree_arena_add_child old0 0 child
    let old2 %ViewTreeArena unwrap_ok view_tree_arena_add_child old1 1 nested
    let reparent0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let reparent1 %ViewTreeArena unwrap_ok view_tree_arena_add_child reparent0 0 child
    let reparent2 %ViewTreeArena unwrap_ok view_tree_arena_add_child reparent1 0 nested
    let changed_child_config %ButtonConfig button_config child_id "Child" changed_child_action
    let changed_child %WidgetDescriptor widget_button changed_child_config button_hint
    let changed_nested_config %ButtonConfig button_config nested_id "Nested" changed_nested_action
    let changed_nested %WidgetDescriptor widget_button changed_nested_config button_hint
    let changed0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let changed1 %ViewTreeArena unwrap_ok view_tree_arena_add_child changed0 0 changed_child
    let changed2 %ViewTreeArena unwrap_ok view_tree_arena_add_child changed1 1 changed_nested
    let shape_invalidation %GuiInvalidation view_tree_arena_invalidation &old2 &reparent2
    let multi_invalidation %GuiInvalidation view_tree_arena_invalidation &old2 &changed2
    let shape_check assert invalidation_is_tree shape_invalidation
    let multi_check assert invalidation_is_tree multi_invalidation
    view_tree_arena_free old2
    view_tree_arena_free reparent2
    view_tree_arena_free changed2
    let checks1 checks_push checks_new shape_check
    let checks checks_push checks1 multi_check
    let shown checks_print_report checks
    checks_exit_code shown
```
