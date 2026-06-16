# GUI render2d SourceOver alpha mask

このファイルは、F5bq の RGBA8888 SourceOver alpha-mask 合成が render2d 共通の純粋 helper として、font/glyf や platform backend に依存せず動作することを固定する。

source policy coverage labels:

- render2d_source_over_alpha_mask_formula_ok
- render2d_source_over_alpha_mask_floor_rounding_ok
- render2d_source_over_alpha_mask_zero_mask_ok
- render2d_source_over_alpha_mask_full_mask_ok
- render2d_source_over_alpha_mask_partial_alpha_ok
- render2d_source_over_alpha_mask_low_alpha_unpremultiply_ok
- render2d_source_over_alpha_mask_typed_error_ok
- render2d_source_over_alpha_mask_no_platform_no_fallback

## source over alpha mask cases

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_source_over_alpha_mask\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"1\" actual=\"1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_source_over_alpha_mask_formula_ok
// render2d_source_over_alpha_mask_floor_rounding_ok
// render2d_source_over_alpha_mask_zero_mask_ok
// render2d_source_over_alpha_mask_full_mask_ok
// render2d_source_over_alpha_mask_partial_alpha_ok
// render2d_source_over_alpha_mask_low_alpha_unpremultiply_ok
// render2d_source_over_alpha_mask_typed_error_ok
// render2d_source_over_alpha_mask_no_platform_no_fallback

fn kind_is_invalid_max %fn GuiRgba8888SourceOverAlphaMaskErrorKind bool \kind:
    match kind:
        GuiRgba8888SourceOverAlphaMaskErrorKind::InvalidMaskAlphaMax:
            true
        _:
            false

fn color_eq %fn &Rgba8888 fn i32 fn i32 fn i32 fn i32 bool \color\r\g\b\a:
    let rr %i32 cast rgba8888_r color
    let gg %i32 cast rgba8888_g color
    let bb %i32 cast rgba8888_b color
    let aa %i32 cast rgba8888_a color
    let rg %bool and eq rr r eq gg g
    let ba %bool and eq bb b eq aa a
    and rg ba

fn zero_mask_case %fn void bool \void:
    let sr %u8 cast 200
    let sg %u8 cast 10
    let sb %u8 cast 10
    let sa %u8 cast 180
    let dr %u8 cast 10
    let dg %u8 cast 20
    let db %u8 cast 30
    let da %u8 cast 40
    let s %Rgba8888 rgba8888_new sr sg sb sa
    let d %Rgba8888 rgba8888_new dr dg db da
    match gui_rgba8888_source_over_alpha_mask &s 0 4 &d:
        Result::Err _:
            false
        Result::Ok out:
            color_eq &out 10 20 30 40

fn full_mask_case %fn void bool \void:
    let sr %u8 cast 100
    let sg %u8 cast 50
    let sb %u8 cast 25
    let sa %u8 cast 255
    let zero %u8 cast 0
    let s %Rgba8888 rgba8888_new sr sg sb sa
    let d %Rgba8888 rgba8888_new zero zero zero zero
    match gui_rgba8888_source_over_alpha_mask &s 4 4 &d:
        Result::Err _:
            false
        Result::Ok out:
            color_eq &out 100 50 25 255

fn partial_alpha_case %fn void bool \void:
    let sr %u8 cast 200
    let sg %u8 cast 0
    let sb %u8 cast 0
    let sa %u8 cast 128
    let dr %u8 cast 0
    let dg %u8 cast 0
    let db %u8 cast 200
    let da %u8 cast 255
    let s %Rgba8888 rgba8888_new sr sg sb sa
    let d %Rgba8888 rgba8888_new dr dg db da
    match gui_rgba8888_source_over_alpha_mask &s 1 2 &d:
        Result::Err _:
            false
        Result::Ok out:
            color_eq &out 50 0 149 255

fn low_alpha_case %fn void bool \void:
    let white %u8 cast 255
    let alpha %u8 cast 1
    let s %Rgba8888 rgba8888_new white white white alpha
    let d %Rgba8888 rgba8888_new white white white alpha
    match gui_rgba8888_source_over_alpha_mask &s 1 1 &d:
        Result::Err _:
            false
        Result::Ok out:
            color_eq &out 255 255 255 1

fn invalid_case %fn void bool \void:
    let sr %u8 cast 1
    let sg %u8 cast 2
    let sb %u8 cast 3
    let sa %u8 cast 4
    let zero %u8 cast 0
    let s %Rgba8888 rgba8888_new sr sg sb sa
    let d %Rgba8888 rgba8888_new zero zero zero zero
    match gui_rgba8888_source_over_alpha_mask &s 1 0 &d:
        Result::Err kind:
            kind_is_invalid_max kind
        Result::Ok _:
            false

fn source_over_case %fn void i32 \void:
    let zero_ok %bool zero_mask_case
    let full_ok %bool full_mask_case
    let partial_ok %bool partial_alpha_case
    let low_alpha_ok %bool low_alpha_case
    let invalid_ok %bool invalid_case
    let first %bool and zero_ok full_ok
    let second %bool and partial_ok low_alpha_ok
    if and first and second invalid_ok 1 0

fn main %impure fn void i32 \void:
    let actual %i32 source_over_case
    let report:
        test::test_report_new "gui_render2d_source_over_alpha_mask"
        |> test::test_report_push test::assert_eq_i32 "return value" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
