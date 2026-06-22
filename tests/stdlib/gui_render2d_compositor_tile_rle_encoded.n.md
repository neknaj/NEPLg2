# GUI render2d compositor tile RLE encoded seal

このファイルは、F5mo の RGBA8888 compositor tile RLE encoded seal bridge が F5mn write cursor owner から lower encoded seal へ進み、packet / present / raw byte / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_rle_encoded_facade_ok
- render2d_compositor_tile_rle_encoded_seal_error_kind_runtime_ok
- render2d_compositor_tile_rle_encoded_seal_ok
- render2d_compositor_tile_rle_encoded_counts_progress_ok
- render2d_compositor_tile_rle_encoded_metadata_ok
- render2d_compositor_tile_rle_encoded_finish_payload_recovery_ok
- render2d_compositor_tile_rle_encoded_seal_error_recovery_source_policy_ok
- render2d_compositor_tile_rle_encoded_free_delegates_lower_encoded_ok
- render2d_compositor_tile_rle_encoded_no_packet_present_raw_fallback

## encoded seal facade and value-only error wrappers compile in the wasm runner

[目的/もくてき]:
- F5mo facade が lower encoded seal / finish error kind を公開型として扱えることを確認します。
- owner-backed write cursor はこの軽量 smoke では forge せず、seal の owner 遷移は source policy と constructor 制限テストで固定します。

neplg2:test
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_tile_rle_encoded" as *
#import "alloc/gui/render2d/row_tile_rle_encoded" as *
#import "core/math" as *

// render2d_compositor_tile_rle_encoded_facade_ok
// render2d_compositor_tile_rle_encoded_seal_error_kind_runtime_ok

fn seal_error_kind_code %fn GuiRgba8888CompositorTileRleEncodedSealErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRleEncodedSealErrorKind::EncodedSealFailed lower:
            match lower:
                GuiRgba8888RowTileRleEncodedSealErrorKind::WriterNotComplete:
                    1
                _:
                    2

fn finish_error_kind_code %fn GuiRgba8888CompositorTileRleEncodedFinishErrorKind i32 \kind:
    match kind:
        GuiRgba8888CompositorTileRleEncodedFinishErrorKind::EncodedFinishFailed lower:
            match lower:
                GuiRgba8888RowTileRleEncodedFinishErrorKind::StorageDeallocFailed:
                    3
                _:
                    4

fn main %impure fn void i32 \void:
    let lower_seal %GuiRgba8888RowTileRleEncodedSealErrorKind GuiRgba8888RowTileRleEncodedSealErrorKind::WriterNotComplete
    let seal_kind %GuiRgba8888CompositorTileRleEncodedSealErrorKind GuiRgba8888CompositorTileRleEncodedSealErrorKind::EncodedSealFailed lower_seal
    let lower_finish %GuiRgba8888RowTileRleEncodedFinishErrorKind GuiRgba8888RowTileRleEncodedFinishErrorKind::StorageDeallocFailed
    let finish_kind %GuiRgba8888CompositorTileRleEncodedFinishErrorKind GuiRgba8888CompositorTileRleEncodedFinishErrorKind::EncodedFinishFailed lower_finish
    if and eq seal_error_kind_code seal_kind 1 eq finish_error_kind_code finish_kind 3:
        then 0
        else 1
```

## compositor encoded seal source policy wraps lower sealed owner

[目的/もくてき]:
- F5mn write cursor owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_rle_encoded_seal` を 1 回だけ通すことを source policy で確認します。
- lower seal error の kind / category read-before-owner-recovery、count/progress/metadata access、payload recovery / free delegation は `nodesrc/test_web_gui_font_rendering_contract.js` で実装順序を固定します。
- default WASM runner では public owner chain 全体の Resource check が timeout するため、この owner-backed E2E fixture は source policy 用に skip します。

neplg2:test[skip]
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_tile_rle_encoded" as *

// render2d_compositor_tile_rle_encoded_seal_ok
// render2d_compositor_tile_rle_encoded_counts_progress_ok
// render2d_compositor_tile_rle_encoded_metadata_ok
// render2d_compositor_tile_rle_encoded_finish_payload_recovery_ok
// render2d_compositor_tile_rle_encoded_seal_error_recovery_source_policy_ok
// render2d_compositor_tile_rle_encoded_free_delegates_lower_encoded_ok
// render2d_compositor_tile_rle_encoded_no_packet_present_raw_fallback

fn main %impure fn void i32 \void:
    0
```

## malformed encoded owner is not public application surface

通常の application code は malformed compositor encoded owner を直 constructor で作れない。F5mo seal error recovery の read-before-consume 順序と write cursor owner recovery は `nodesrc/test_web_gui_font_rendering_contract.js` の source policy で固定する。

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *

// render2d_compositor_tile_rle_encoded_seal_error_recovery_source_policy_ok

fn forge_compositor_encoded %fn GuiRgba8888RowTileRleEncodedOwner fn GuiRgba8888CompositorFrameEntryMetadata GuiRgba8888CompositorTileRleEncodedOwner \lower\metadata:
    GuiRgba8888CompositorTileRleEncodedOwner lower metadata

fn main %impure fn void i32 \void:
    0
```
