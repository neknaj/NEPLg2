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

fn main %fn unit i32 \unit:
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

fn main %fn unit i32 \unit:
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

fn main %fn unit i32 \unit:
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

fn main %fn unit i32 \unit:
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

fn main %fn unit i32 \unit:
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
