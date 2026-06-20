# GUI core alpha mask command

このファイルは、F5bk の core alpha mask command が no_alloc の opaque resource handle と command payload だけを提供し、mask storage や backend fallback を core に持ち込まない契約を固定する。

source policy coverage labels:

- core_alpha_mask_rect_command_handle_ok
- core_alpha_mask_rect_command_accessors_ok
- core_alpha_mask_rect_command_variant_ok
- core_alpha_mask_rect_command_source_over_only_contract
- core_alpha_mask_rect_command_no_alloc_no_platform_no_fallback

## alpha mask rect command smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_core_alpha_mask_rect_command\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/geometry" as *
#import "core/gui/render_command" as *
#import "core/test" as *
#import "std/test" as test

// core_alpha_mask_rect_command_handle_ok
// core_alpha_mask_rect_command_accessors_ok
// core_alpha_mask_rect_command_variant_ok
// core_alpha_mask_rect_command_source_over_only_contract
// core_alpha_mask_rect_command_no_alloc_no_platform_no_fallback

fn alpha_mask_variant_code %fn RenderCommand i32 \command:
    match command:
        RenderCommand::AlphaMaskRect payload:
            let read_id %AlphaMaskId alpha_mask_rect_command_mask_id &payload
            alpha_mask_id_raw &read_id
        _:
            -1

fn run_case %fn void i32 \void:
    let zero %u8 cast 0
    let full %u8 cast 255
    let half %u8 cast 128
    let color %Rgba8888 rgba8888_new full zero full half
    let paint %GuiPaint gui_paint_solid color
    let mask %AlphaMaskId alpha_mask_id_new 77
    let rect %GuiRect gui_rect_new 4 5 128 32
    let payload %AlphaMaskRectCommand AlphaMaskRectCommand mask rect paint
    let read_id %AlphaMaskId alpha_mask_rect_command_mask_id &payload
    let read_rect %GuiRect alpha_mask_rect_command_rect &payload
    let read_paint %GuiPaint alpha_mask_rect_command_paint &payload
    let read_color %Rgba8888 gui_paint_color &read_paint
    assert_eq_i32 77 alpha_mask_id_raw &read_id
    assert_eq_i32 4 gui_rect_x &read_rect
    assert_eq_i32 5 gui_rect_y &read_rect
    assert_eq_i32 128 gui_rect_width &read_rect
    assert_eq_i32 32 gui_rect_height &read_rect
    assert_eq_i32 128 cast rgba8888_a &read_color
    assert_eq_i32 77 alpha_mask_variant_code render_command_alpha_mask_rect mask rect paint
    0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_core_alpha_mask_rect_command"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
