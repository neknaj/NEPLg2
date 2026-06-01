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
