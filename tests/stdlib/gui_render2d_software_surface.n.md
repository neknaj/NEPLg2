# GUI render2d software surface

このファイルは、F5bo の RGBA8888 software surface owner が render2d 共通の pixel memory boundary として動作し、platform や font 実装に依存しないことを固定する。

source policy coverage labels:

- render2d_software_surface_facade_ok
- render2d_software_surface_owner_region_token_ok
- render2d_software_surface_create_validation_ok
- render2d_software_surface_allocation_failure_mapping_ok
- render2d_software_surface_read_write_roundtrip_ok
- render2d_software_surface_write_failure_owner_recovery_ok
- render2d_software_surface_free_ok
- render2d_software_surface_no_platform_no_font_no_fallback

## software surface shape validation

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_software_surface_shape\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"1\" actual=\"1\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "core/cast" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_software_surface_facade_ok
// render2d_software_surface_create_validation_ok
// render2d_software_surface_no_platform_no_font_no_fallback

fn kind_is_invalid_geometry %fn GuiRgba8888SoftwareSurfaceErrorKind bool \kind:
    match kind:
        GuiRgba8888SoftwareSurfaceErrorKind::InvalidGeometry:
            true
        _:
            false

fn kind_is_pixel_count_overflow %fn GuiRgba8888SoftwareSurfaceErrorKind bool \kind:
    match kind:
        GuiRgba8888SoftwareSurfaceErrorKind::PixelCountOverflow:
            true
        _:
            false

fn kind_is_stride_overflow %fn GuiRgba8888SoftwareSurfaceErrorKind bool \kind:
    match kind:
        GuiRgba8888SoftwareSurfaceErrorKind::StrideOverflow:
            true
        _:
            false

fn kind_is_byte_length_overflow %fn GuiRgba8888SoftwareSurfaceErrorKind bool \kind:
    match kind:
        GuiRgba8888SoftwareSurfaceErrorKind::ByteLengthOverflow:
            true
        _:
            false

fn shape_case %fn void i32 \void:
    let valid_ok %bool match gui_rgba8888_software_surface_shape 2 3:
        Result::Err _:
            false
        Result::Ok shape:
            and eq gui_rgba8888_software_surface_shape_stride_bytes &shape 8 eq gui_rgba8888_software_surface_shape_byte_len &shape 24
    let invalid_ok %bool match gui_rgba8888_software_surface_shape 0 3:
        Result::Err kind:
            kind_is_invalid_geometry kind
        Result::Ok _:
            false
    let pixel_overflow_ok %bool match gui_rgba8888_software_surface_shape 1073741824 2:
        Result::Err kind:
            kind_is_pixel_count_overflow kind
        Result::Ok _:
            false
    let stride_overflow_ok %bool match gui_rgba8888_software_surface_shape 536870912 1:
        Result::Err kind:
            kind_is_stride_overflow kind
        Result::Ok _:
            false
    let byte_length_overflow_ok %bool match gui_rgba8888_software_surface_shape 536870911 2:
        Result::Err kind:
            kind_is_byte_length_overflow kind
        Result::Ok _:
            false
    let first %bool and valid_ok invalid_ok
    let second %bool and pixel_overflow_ok stride_overflow_ok
    if and first and second byte_length_overflow_ok 1 0

fn main %impure fn void i32 \void:
    let actual %i32 shape_case
    let report:
        test::test_report_new "gui_render2d_software_surface_shape"
        |> test::test_report_push test::assert_eq_i32 "return value" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## software surface owner read write

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_software_surface_owner\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"1\" actual=\"1\" message=\"\"\n"
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

// render2d_software_surface_owner_region_token_ok
// render2d_software_surface_allocation_failure_mapping_ok
// render2d_software_surface_read_write_roundtrip_ok
// render2d_software_surface_write_failure_owner_recovery_ok
// render2d_software_surface_free_ok
// render2d_software_surface_no_platform_no_font_no_fallback

fn kind_is_out_of_memory %fn GuiRgba8888SoftwareSurfaceErrorKind bool \kind:
    match kind:
        GuiRgba8888SoftwareSurfaceErrorKind::OutOfMemory:
            true
        _:
            false

fn kind_is_index_out_of_bounds %fn GuiRgba8888SoftwareSurfaceErrorKind bool \kind:
    match kind:
        GuiRgba8888SoftwareSurfaceErrorKind::IndexOutOfBounds:
            true
        _:
            false

fn allocation_failure_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 536870911 1:
        Result::Err e:
            kind_is_out_of_memory gui_rgba8888_software_surface_create_error_kind &e
        Result::Ok surface:
            match gui_rgba8888_software_surface_free surface:
                Result::Ok _:
                    false
                Result::Err _:
                    false

fn read_write_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            false
        Result::Ok surface0:
            let r %u8 cast 10
            let g %u8 cast 20
            let b %u8 cast 30
            let a %u8 cast 40
            let color %Rgba8888 rgba8888_new r g b a
            match gui_rgba8888_software_surface_write_pixel surface0 1 1 color:
                Result::Err e:
                    match gui_rgba8888_software_surface_write_error_free e:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Ok surface1:
                    let ok %bool match gui_rgba8888_software_surface_read_pixel &surface1 1 1:
                        Result::Err _:
                            false
                        Result::Ok got:
                            let got_r %i32 cast rgba8888_r &got
                            let got_g %i32 cast rgba8888_g &got
                            let got_b %i32 cast rgba8888_b &got
                            let got_a %i32 cast rgba8888_a &got
                            let channel_rg %bool and eq got_r 10 eq got_g 20
                            let channel_ba %bool and eq got_b 30 eq got_a 40
                            and channel_rg channel_ba
                    match gui_rgba8888_software_surface_free surface1:
                        Result::Err _:
                            false
                        Result::Ok _:
                            ok

fn write_failure_recovery_case %fn void bool \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            false
        Result::Ok surface0:
            let zero %u8 cast 0
            let color %Rgba8888 rgba8888_new zero zero zero zero
            match gui_rgba8888_software_surface_write_pixel surface0 1 0 color:
                Result::Ok surface1:
                    match gui_rgba8888_software_surface_free surface1:
                        Result::Ok _:
                            false
                        Result::Err _:
                            false
                Result::Err e:
                    let kind_ok %bool kind_is_index_out_of_bounds gui_rgba8888_software_surface_write_error_kind &e
                    let recovered %GuiRgba8888SoftwareSurfaceOwner gui_rgba8888_software_surface_write_error_surface e
                    match gui_rgba8888_software_surface_free recovered:
                        Result::Err _:
                            false
                        Result::Ok _:
                            kind_ok

fn owner_case %fn void i32 \void:
    let alloc_ok %bool allocation_failure_case
    let rw_ok %bool read_write_case
    let recovery_ok %bool write_failure_recovery_case
    if and alloc_ok and rw_ok recovery_ok 1 0

fn main %impure fn void i32 \void:
    let actual %i32 owner_case
    let report:
        test::test_report_new "gui_render2d_software_surface_owner"
        |> test::test_report_push test::assert_eq_i32 "return value" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
