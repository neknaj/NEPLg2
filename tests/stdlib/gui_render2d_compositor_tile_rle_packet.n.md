# GUI render2d compositor tile RLE packet doctests

このファイルは、F5mp の RGBA8888 compositor tile RLE packet bridge が F5mo encoded owner から lower packet owner へ進み、packet record / std present / raw byte / fallback へ進まないことを固定する。

source policy labels:

- render2d_compositor_tile_rle_packet_facade_ok
- render2d_compositor_tile_rle_packet_prepare_error_kind_runtime_ok
- render2d_compositor_tile_rle_packet_prepare_ok
- render2d_compositor_tile_rle_packet_descriptor_wrapper_ok
- render2d_compositor_tile_rle_packet_descriptor_scalar_accessors_ok
- render2d_compositor_tile_rle_packet_metadata_ok
- render2d_compositor_tile_rle_packet_finish_encoded_recovery_ok
- render2d_compositor_tile_rle_packet_prepare_error_recovery_source_policy_ok
- render2d_compositor_tile_rle_packet_free_delegates_lower_packet_ok
- render2d_compositor_tile_rle_packet_no_record_present_raw_fallback

## facade and wrapped kind smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_rle_packet\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"packet prepare kind\" expected=\"17\" actual=\"17\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "alloc/gui/render2d/compositor_tile_rle_packet" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "std/test" as test

// render2d_compositor_tile_rle_packet_facade_ok
// render2d_compositor_tile_rle_packet_prepare_error_kind_runtime_ok
// render2d_compositor_tile_rle_packet_finish_error_kind_runtime_ok

fn prepare_error_kind_code %fn GuiRgba8888CompositorTileRlePacketPrepareErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRlePacketPrepareErrorKind::PacketPrepareFailed lower:
            match lower:
                GuiRgba8888RowTileRlePacketPrepareErrorKind::EncodedByteCountInvalid:
                    17
                _:
                    0

fn main %impure fn void i32 \void:
    let lower_prepare %GuiRgba8888RowTileRlePacketPrepareErrorKind GuiRgba8888RowTileRlePacketPrepareErrorKind::EncodedByteCountInvalid
    let prepare_kind %GuiRgba8888CompositorTileRlePacketPrepareErrorKind GuiRgba8888CompositorTileRlePacketPrepareErrorKind::PacketPrepareFailed lower_prepare
    let report:
        test::test_report_new "gui_render2d_compositor_tile_rle_packet"
        |> test::test_report_push test::assert_eq_i32 "packet prepare kind" 17 prepare_error_kind_code prepare_kind
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## source-policy fixtures

neplg2:test[skip]
```neplg2
#indent 4

#import "alloc/gui/render2d/compositor_tile_rle_packet" as *

// render2d_compositor_tile_rle_packet_prepare_ok
// render2d_compositor_tile_rle_packet_descriptor_wrapper_ok
// render2d_compositor_tile_rle_packet_descriptor_scalar_accessors_ok
// render2d_compositor_tile_rle_packet_metadata_ok
// render2d_compositor_tile_rle_packet_finish_encoded_recovery_ok
// render2d_compositor_tile_rle_packet_prepare_error_recovery_source_policy_ok
// render2d_compositor_tile_rle_packet_free_delegates_lower_packet_ok
// render2d_compositor_tile_rle_packet_no_record_present_raw_fallback
```

## owner constructor stays restricted

neplg2:test[compile_fail]
```neplg2
#indent 4

#import "alloc/gui/render2d/compositor_tile_rle_packet" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *

fn forged %fn GuiRgba8888RowTileRlePacketOwner fn GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorTileRlePacketOwner \packet\metadata:
    GuiRgba8888CompositorTileRlePacketOwner packet metadata
```
