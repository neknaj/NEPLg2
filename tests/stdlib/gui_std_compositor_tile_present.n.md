# GUI std compositor tile RLE present bridge doctests

このファイルは、F5mq の std layer compositor tile RLE present-frame bridge が compositor packet owner から lower std present-frame owner へ進み、run cursor / packet record / host / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_frame_facade_ok
- std_compositor_tile_rle_present_frame_prepare_error_kind_runtime_ok
- std_compositor_tile_rle_present_frame_prepare_ok
- std_compositor_tile_rle_present_frame_metadata_validation_ok
- std_compositor_tile_rle_present_frame_owner_recovery_ok
- std_compositor_tile_rle_present_frame_lower_present_recovery_ok
- std_compositor_tile_rle_present_frame_free_delegates_packet_ok
- std_compositor_tile_rle_present_frame_no_cursor_record_host_platform_fallback

## facade and wrapped kind smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_compositor_tile_present\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"present kind\" expected=\"23\" actual=\"23\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present" as *
#import "std/gui/tile_present" as *
#import "std/test" as test

// std_compositor_tile_rle_present_frame_facade_ok
// std_compositor_tile_rle_present_frame_prepare_error_kind_runtime_ok

fn prepare_error_kind_code %fn GuiRgba8888CompositorTileRlePresentFramePrepareErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRlePresentFramePrepareErrorKind::FrameIdInvalid:
            11
        GuiRgba8888CompositorTileRlePresentFramePrepareErrorKind::FrameIdMismatch:
            17
        GuiRgba8888CompositorTileRlePresentFramePrepareErrorKind::PresentFramePrepareFailed lower:
            match lower:
                GuiRgba8888RowTileRlePresentFramePrepareErrorKind::FrameIdMismatch:
                    23
                _:
                    0

fn main %impure fn void i32 \void:
    let lower %GuiRgba8888RowTileRlePresentFramePrepareErrorKind GuiRgba8888RowTileRlePresentFramePrepareErrorKind::FrameIdMismatch
    let kind %GuiRgba8888CompositorTileRlePresentFramePrepareErrorKind GuiRgba8888CompositorTileRlePresentFramePrepareErrorKind::PresentFramePrepareFailed lower
    let report:
        test::test_report_new "gui_std_compositor_tile_present"
        |> test::test_report_push test::assert_eq_i32 "present kind" 23 prepare_error_kind_code kind
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## source-policy fixtures

neplg2:test[skip]
```neplg2
#indent 4

#import "std/gui/compositor_tile_present" as *

// std_compositor_tile_rle_present_frame_prepare_ok
// std_compositor_tile_rle_present_frame_metadata_validation_ok
// std_compositor_tile_rle_present_frame_owner_recovery_ok
// std_compositor_tile_rle_present_frame_lower_present_recovery_ok
// std_compositor_tile_rle_present_frame_free_delegates_packet_ok
// std_compositor_tile_rle_present_frame_no_cursor_record_host_platform_fallback
```

## owner constructor stays restricted

neplg2:test[compile_fail]
```neplg2
#indent 4

#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "std/gui/compositor_tile_present" as *
#import "std/gui/tile_present" as *

fn forged %fn GuiRgba8888RowTileRlePresentFrameOwner fn GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorTileRlePresentFrameOwner \present\metadata:
    GuiRgba8888CompositorTileRlePresentFrameOwner present metadata
```
