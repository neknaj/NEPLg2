# GUI std compositor tile RLE present virtual-drain doctests

このファイルは、F5mv の std layer compositor tile RLE present host-command virtual drain boundary が F5mu record だけを受け取り、metadata consistency と run offset continuity を検査し、lower cursor / host import / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_virtual_drain_facade_ok
- std_compositor_tile_rle_present_virtual_drain_phase_enum_ok
- std_compositor_tile_rle_present_virtual_drain_error_enum_ok
- std_compositor_tile_rle_present_virtual_drain_f5mu_record_only_ok
- std_compositor_tile_rle_present_virtual_drain_descriptor_accessor_ok
- std_compositor_tile_rle_present_virtual_drain_metadata_consistency_ok
- std_compositor_tile_rle_present_virtual_drain_run_offset_continuity_ok
- std_compositor_tile_rle_present_virtual_drain_no_lower_cursor_host_platform_fallback

## module import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present_virtual_drain\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std compositor tile present virtual drain import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_virtual_drain" as *
#import "std/test" as test

// std_compositor_tile_rle_present_virtual_drain_facade_ok
// std_compositor_tile_rle_present_virtual_drain_phase_enum_ok
// std_compositor_tile_rle_present_virtual_drain_error_enum_ok
// std_compositor_tile_rle_present_virtual_drain_f5mu_record_only_ok
// std_compositor_tile_rle_present_virtual_drain_descriptor_accessor_ok
// std_compositor_tile_rle_present_virtual_drain_metadata_consistency_ok
// std_compositor_tile_rle_present_virtual_drain_run_offset_continuity_ok
// std_compositor_tile_rle_present_virtual_drain_no_lower_cursor_host_platform_fallback

fn phase_code %fn GuiRgba8888CompositorTileRlePresentVirtualDrainPhase i32 \phase:
    match phase:
        GuiRgba8888CompositorTileRlePresentVirtualDrainPhase::WaitingBegin:
            0
        GuiRgba8888CompositorTileRlePresentVirtualDrainPhase::InFrame:
            1
        GuiRgba8888CompositorTileRlePresentVirtualDrainPhase::Ended:
            2

fn main %impure fn void i32 \void:
    let drain %GuiRgba8888CompositorTileRlePresentVirtualDrain gui_rgba8888_compositor_tile_rle_present_virtual_drain_empty
    let phase %GuiRgba8888CompositorTileRlePresentVirtualDrainPhase gui_rgba8888_compositor_tile_rle_present_virtual_drain_phase &drain
    let report:
        test::test_report_new "gui_std_compositor_tile_present_virtual_drain"
        |> test::test_report_push test::assert_eq_i32 "std compositor tile present virtual drain import" 0 phase_code phase
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
