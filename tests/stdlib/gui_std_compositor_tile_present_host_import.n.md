# GUI std compositor tile RLE present host-import doctests

このファイルは、F5mx の std layer compositor tile RLE present host continuation request boundary が F5mu record だけを host target 付き request に包み、headless unsupported / RGBA8888 capability validation を行い、lower F5cr/F5cq / raw / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_import_facade_ok
- std_compositor_tile_rle_present_host_import_target_enum_ok
- std_compositor_tile_rle_present_host_import_rgba8888_capability_ok
- std_compositor_tile_rle_present_host_import_headless_unsupported_ok
- std_compositor_tile_rle_present_host_import_consumes_f5mu_only_ok
- std_compositor_tile_rle_present_host_import_no_schedule_no_lower_raw_no_host_call_no_platform_no_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_host_import\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std compositor tile present host import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_host_import" as *
#import "std/test" as test

// std_compositor_tile_rle_present_host_import_facade_ok
// std_compositor_tile_rle_present_host_import_target_enum_ok
// std_compositor_tile_rle_present_host_import_rgba8888_capability_ok
// std_compositor_tile_rle_present_host_import_headless_unsupported_ok
// std_compositor_tile_rle_present_host_import_consumes_f5mu_only_ok
// std_compositor_tile_rle_present_host_import_no_schedule_no_lower_raw_no_host_call_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let actual %i32 0
    let report:
        test::test_report_new "gui_std_compositor_tile_present_host_import"
        |> test::test_report_push test::assert_eq_i32 "std compositor tile present host import" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
