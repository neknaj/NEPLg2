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
    let ctx %LayoutContext MockTextMeasurer layout_context (layout_constraints 0 0 24 10) (gui_scale_factor_new 1 1) (mock_text_measurer_new 8 16 12) gui_capabilities_text_grid
    match layout_measure_text_width &ctx (text_run_id_new 1) (font_id_new 1) 5:
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
    let ctx %LayoutContext MockTextMeasurer layout_context (layout_constraints 0 0 24 10) (gui_scale_factor_new 1 1) (mock_text_measurer_new 8 16 12) gui_capabilities_text_grid
    match layout_measure_text_height &ctx (text_run_id_new 1) (font_id_new 1) 5:
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
    let ctx %LayoutContext MockTextMeasurer layout_context (layout_constraints 0 0 (sub 0 1) 10) (gui_scale_factor_new 1 1) (mock_text_measurer_new 8 16 12) gui_capabilities_text_grid
    match layout_measure_text_width &ctx (text_run_id_new 1) (font_id_new 1) 5:
        Result::Ok width:
            width
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    0
                _:
                    1
```
