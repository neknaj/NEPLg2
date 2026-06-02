# alloc/gui layout

このファイルは GUI / TUI 共通 layout contract が platform API に依存せず、`TextMeasurer` 注入と `Result` による失敗通知で動くことを固定します。

## layout_text_width_uses_injected_text_measurer_and_clamps

[目的/もくてき]:
- layout が browser / terminal / OS font API を直接呼ばず、context に注入された `MockTextMeasurer` だけで測定することを確認します。
- 測定 width が max width を超えないことを固定します。

neplg2:test
ret: 24
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let constraints %LayoutConstraints layout_constraints 0 0 24 10
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let run_id %TextRunId text_run_id_new 1
    let font_id %FontId font_id_new 1
    match layout_measure_text_width &ctx run_id font_id 5:
        Result::Ok width:
            width
        Result::Err _width_error:
            1
```

## layout_text_height_uses_injected_text_measurer_and_clamps

[目的/もくてき]:
- 測定 height が max height を超えないことを固定します。

neplg2:test
ret: 10
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let constraints %LayoutConstraints layout_constraints 0 0 24 10
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let run_id %TextRunId text_run_id_new 1
    let font_id %FontId font_id_new 1
    match layout_measure_text_height &ctx run_id font_id 5:
        Result::Ok height:
            height
        Result::Err _height_error:
            1
```

## layout_invalid_constraints_are_result_error

[目的/もくてき]:
- invalid geometry が panic や silent no-op ではなく `GuiError::InvalidGeometry` として返ることを固定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let negative_max_width %i32 sub 0 1
    let constraints %LayoutConstraints layout_constraints 0 0 negative_max_width 10
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let run_id %TextRunId text_run_id_new 1
    let font_id %FontId font_id_new 1
    match layout_measure_text_width &ctx run_id font_id 5:
        Result::Ok width:
            width
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    0
                _:
                    1
```

## layout_view_tree_arena_linear_places_nested_nodes

[目的/もくてき]:
- allocator-backed `ViewTreeArena` を `LayoutTreeArena` へ変換し、parent index / depth を保ったまま layout phase に接続できることを確認します。
- 初期 linear layout contract として、arena order で y 方向に積まれることを固定します。

neplg2:test
ret: 0
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
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let child_id %WidgetId widget_id 2
    let child_action %ActionId action_id 2
    let child_config %ButtonConfig button_config child_id "Open" child_action
    let child_hint %LayoutHint layout_hint_fixed 8 1
    let child %WidgetDescriptor widget_button child_config child_hint
    let nested_id %WidgetId widget_id 3
    let nested_action %ActionId action_id 3
    let nested_config %ButtonConfig button_config nested_id "Save" nested_action
    let nested_hint %LayoutHint layout_hint_fixed 8 1
    let nested %WidgetDescriptor widget_button nested_config nested_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view1 %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 child
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view1 1 nested
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let run_id %TextRunId text_run_id_new 1
    let font_id %FontId font_id_new 1
    match layout_view_tree_arena_linear &ctx &view run_id font_id 2:
        Result::Ok layout:
            let count_ok %bool eq layout_tree_arena_len &layout 3
            let nested_check %bool:
                match layout_tree_arena_get_node &layout 2:
                    Option::Some node:
                        let depth %i32 layout_tree_arena_node_depth &node
                        let placed %LayoutNode layout_tree_arena_node_layout &node
                        let bounds %GuiRect layout_node_bounds &placed
                        let y %i32 gui_rect_y &bounds
                        and:
                            eq depth 2
                            eq y 2
                    Option::None:
                        false
            layout_tree_arena_free layout
            view_tree_arena_free view
            if and count_ok nested_check:
                then 0
                else 9
        Result::Err _error:
            view_tree_arena_free view
            1
```

## layout_view_tree_arena_linear_invalid_constraints_are_result_error

[目的/もくてき]:
- arena layout 中の測定失敗が panic や silent no-op ではなく `GuiError::InvalidGeometry` として返ることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 1
    let root_hint %LayoutHint layout_hint_fixed 8 1
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let view %ViewTreeArena unwrap_ok view_tree_arena_single root
    let negative_max_width %i32 sub 0 1
    let constraints %LayoutConstraints layout_constraints 0 0 negative_max_width 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let run_id %TextRunId text_run_id_new 1
    let font_id %FontId font_id_new 1
    match layout_view_tree_arena_linear &ctx &view run_id font_id 2:
        Result::Ok layout:
            layout_tree_arena_free layout
            view_tree_arena_free view
            1
        Result::Err error:
            view_tree_arena_free view
            match error:
                GuiError::InvalidGeometry:
                    0
                _:
                    2
```

## layout_view_tree_arena_stack_places_siblings_under_same_parent

[目的/もくてき]:
- stack layout が global arena order ではなく、同じ parent を持つ sibling だけを previous sibling の size と spacing で積むことを確認します。
- `WidgetId` と arena index を混同しないよう、id は arena index と異なる値にします。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let first_id %WidgetId widget_id 10
    let first_hint %LayoutHint layout_hint_fixed 4 2
    let first %WidgetDescriptor widget_label first_id "first" first_hint
    let second_id %WidgetId widget_id 20
    let second_hint %LayoutHint layout_hint_fixed 4 2
    let second %WidgetDescriptor widget_label second_id "second" second_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view1 %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 first
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view1 0 second
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let policy %StackLayoutPolicy stack_layout_vertical 1
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            let check %bool:
                match layout_tree_arena_get_node &layout 2:
                    Option::Some node:
                        let parent_index %i32 layout_tree_arena_node_parent_index &node
                        let depth %i32 layout_tree_arena_node_depth &node
                        let placed %LayoutNode layout_tree_arena_node_layout &node
                        let bounds %GuiRect layout_node_bounds &placed
                        let y %i32 gui_rect_y &bounds
                        and:
                            and eq parent_index 0 eq depth 1
                            eq y 3
                    Option::None:
                        false
            layout_tree_arena_free layout
            view_tree_arena_free view
            if check:
                then 0
                else 9
        Result::Err _error:
            view_tree_arena_free view
            1
```

## layout_view_tree_arena_stack_resets_offset_for_nested_parent

[目的/もくてき]:
- nested child が root sibling の cursor ではなく、自分の parent の local stack offset から配置されることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let first_id %WidgetId widget_id 10
    let first_hint %LayoutHint layout_hint_fixed 4 2
    let first %WidgetDescriptor widget_label first_id "first" first_hint
    let second_id %WidgetId widget_id 20
    let second_hint %LayoutHint layout_hint_fixed 4 2
    let second %WidgetDescriptor widget_label second_id "second" second_hint
    let nested_id %WidgetId widget_id 30
    let nested_hint %LayoutHint layout_hint_fixed 3 1
    let nested %WidgetDescriptor widget_label nested_id "nested" nested_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view1 %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 first
    let view2 %ViewTreeArena unwrap_ok view_tree_arena_add_child view1 0 second
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view2 1 nested
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let policy %StackLayoutPolicy stack_layout_vertical 1
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            let check %bool:
                match layout_tree_arena_get_node &layout 3:
                    Option::Some node:
                        let parent_index %i32 layout_tree_arena_node_parent_index &node
                        let depth %i32 layout_tree_arena_node_depth &node
                        let placed %LayoutNode layout_tree_arena_node_layout &node
                        let bounds %GuiRect layout_node_bounds &placed
                        let y %i32 gui_rect_y &bounds
                        and:
                            and eq parent_index 1 eq depth 2
                            eq y 0
                    Option::None:
                        false
            layout_tree_arena_free layout
            view_tree_arena_free view
            if check:
                then 0
                else 9
        Result::Err _error:
            view_tree_arena_free view
            1
```

## layout_view_tree_arena_stack_invalid_policy_is_result_error

[目的/もくてき]:
- negative spacing が panic や silent no-op ではなく `GuiError::InvalidGeometry` として返ることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let view %ViewTreeArena unwrap_ok view_tree_arena_single root
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let negative_spacing %i32 sub 0 1
    let policy %StackLayoutPolicy stack_layout_vertical negative_spacing
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            layout_tree_arena_free layout
            view_tree_arena_free view
            1
        Result::Err error:
            view_tree_arena_free view
            match error:
                GuiError::InvalidGeometry:
                    0
                _:
                    2
```

## layout_view_tree_arena_stack_vertical_center_aligns_cross_axis

[目的/もくてき]:
- vertical stack の cross-axis alignment が parent width と child width から x を deterministic に計算することを確認します。
- parent width 20、child width 4 の `Center` は x 8 になることを固定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let child_id %WidgetId widget_id 10
    let child_hint %LayoutHint layout_hint_fixed 4 2
    let child %WidgetDescriptor widget_label child_id "child" child_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 child
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let alignment %StackCrossAlignment StackCrossAlignment::Center
    let policy %StackLayoutPolicy stack_layout_vertical_aligned 0 alignment
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            let check %bool:
                match layout_tree_arena_get_node &layout 1:
                    Option::Some node:
                        let placed %LayoutNode layout_tree_arena_node_layout &node
                        let bounds %GuiRect layout_node_bounds &placed
                        let x %i32 gui_rect_x &bounds
                        eq x 8
                    Option::None:
                        false
            layout_tree_arena_free layout
            view_tree_arena_free view
            if check:
                then 0
                else 9
        Result::Err _error:
            view_tree_arena_free view
            1
```

## layout_view_tree_arena_stack_vertical_stretch_uses_parent_cross_size

[目的/もくてき]:
- `Stretch` alignment が vertical stack の child width を parent width にそろえることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let child_id %WidgetId widget_id 10
    let child_hint %LayoutHint layout_hint_fixed 4 2
    let child %WidgetDescriptor widget_label child_id "child" child_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 child
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let alignment %StackCrossAlignment StackCrossAlignment::Stretch
    let policy %StackLayoutPolicy stack_layout_vertical_aligned 0 alignment
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            let check %bool:
                match layout_tree_arena_get_node &layout 1:
                    Option::Some node:
                        let placed %LayoutNode layout_tree_arena_node_layout &node
                        let bounds %GuiRect layout_node_bounds &placed
                        let width %i32 gui_rect_width &bounds
                        eq width 20
                    Option::None:
                        false
            layout_tree_arena_free layout
            view_tree_arena_free view
            if check:
                then 0
                else 9
        Result::Err _error:
            view_tree_arena_free view
            1
```

## layout_view_tree_arena_stack_horizontal_end_aligns_cross_axis

[目的/もくてき]:
- horizontal stack の cross-axis alignment が y / height 側へ正しく適用されることを確認します。
- parent height 10、child height 2 の `End` は y 8 になることを固定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 10
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let child_id %WidgetId widget_id 10
    let child_hint %LayoutHint layout_hint_fixed 4 2
    let child %WidgetDescriptor widget_label child_id "child" child_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 child
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let alignment %StackCrossAlignment StackCrossAlignment::End
    let policy %StackLayoutPolicy stack_layout_horizontal_aligned 0 alignment
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            let check %bool:
                match layout_tree_arena_get_node &layout 1:
                    Option::Some node:
                        let placed %LayoutNode layout_tree_arena_node_layout &node
                        let bounds %GuiRect layout_node_bounds &placed
                        let y %i32 gui_rect_y &bounds
                        eq y 8
                    Option::None:
                        false
            layout_tree_arena_free layout
            view_tree_arena_free view
            if check:
                then 0
                else 9
        Result::Err _error:
            view_tree_arena_free view
            1
```

## layout_view_tree_arena_stack_rejects_overflow_when_policy_requires_it

[目的/もくてき]:
- `StackOverflowPolicy::Reject` が parent bounds を超える child 配置を `GuiError::InvalidGeometry` として返すことを確認します。
- layout 途中で失敗しても `LayoutTreeArena` owner は layout module 側で解放され、caller は borrowed `ViewTreeArena` だけを解放します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn void i32 \void:
    let root_id %WidgetId widget_id 100
    let root_hint %LayoutHint layout_hint_fixed 20 3
    let root %WidgetDescriptor widget_label root_id "root" root_hint
    let first_id %WidgetId widget_id 10
    let first_hint %LayoutHint layout_hint_fixed 4 2
    let first %WidgetDescriptor widget_label first_id "first" first_hint
    let second_id %WidgetId widget_id 20
    let second_hint %LayoutHint layout_hint_fixed 4 2
    let second %WidgetDescriptor widget_label second_id "second" second_hint
    let view0 %ViewTreeArena unwrap_ok view_tree_arena_single root
    let view1 %ViewTreeArena unwrap_ok view_tree_arena_add_child view0 0 first
    let view %ViewTreeArena unwrap_ok view_tree_arena_add_child view1 0 second
    let constraints %LayoutConstraints layout_constraints 0 0 100 100
    let scale %GuiScaleFactor gui_scale_factor_new 1 1
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let ctx %LayoutContext MockTextMeasurer layout_context constraints scale measurer gui_capabilities_text_grid
    let alignment %StackCrossAlignment StackCrossAlignment::Start
    let overflow %StackOverflowPolicy StackOverflowPolicy::Reject
    let policy %StackLayoutPolicy stack_layout_policy_full StackAxis::Vertical 0 alignment overflow
    match layout_view_tree_arena_stack &ctx &view policy:
        Result::Ok layout:
            layout_tree_arena_free layout
            view_tree_arena_free view
            1
        Result::Err error:
            view_tree_arena_free view
            match error:
                GuiError::InvalidGeometry:
                    0
                _:
                    2
```
