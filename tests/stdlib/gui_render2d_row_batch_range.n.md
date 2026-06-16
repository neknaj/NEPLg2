# GUI render2d row batch range

このファイルは、F5by の RGBA8888 row batch range が batch descriptor を embedded plan authority と照合し、row range と byte offset range の metadata だけを公開することを固定する。

source policy coverage labels:

- render2d_row_batch_range_facade_ok
- render2d_row_batch_range_first_batch_ok
- render2d_row_batch_range_partial_batch_offset_ok
- render2d_row_batch_range_descriptor_authority_error_ok
- render2d_row_batch_range_plan_authority_error_ok
- render2d_row_batch_range_continuation_status_error_ok
- render2d_row_batch_range_metadata_mismatch_error_ok
- render2d_row_batch_range_no_platform_no_fallback

## first batch range metadata

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_range" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *

// render2d_row_batch_range_facade_ok
// render2d_row_batch_range_first_batch_ok
// render2d_row_batch_range_no_platform_no_fallback

fn first_range_ok %fn &GuiRgba8888RowBatchRange bool \range:
    and:
        eq gui_rgba8888_row_batch_range_batch_index range 0
        and:
            eq gui_rgba8888_row_batch_range_row_start range 0
            and:
                eq gui_rgba8888_row_batch_range_row_count range 2
                and:
                    eq gui_rgba8888_row_batch_range_width range 4
                    and:
                        eq gui_rgba8888_row_batch_range_height range 5
                        and:
                            eq gui_rgba8888_row_batch_range_stride_bytes range 16
                            and:
                                eq gui_rgba8888_row_batch_range_byte_len range 80
                                and:
                                    eq gui_rgba8888_row_batch_range_start_byte_offset range 0
                                    eq gui_rgba8888_row_batch_range_byte_count range 32

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 4 5:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err _:
                    2
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 401
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err _:
                            3
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 2
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err _:
                                    4
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err _:
                                            5
                                        Result::Ok cursor:
                                            match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                                Result::Err error:
                                                    match gui_rgba8888_row_batch_cursor_step_error_free error:
                                                        Result::Ok _:
                                                            6
                                                        Result::Err _:
                                                            6
                                                Result::Ok batch:
                                                    match gui_rgba8888_row_batch_range_prepare batch:
                                                        Result::Err error:
                                                            match gui_rgba8888_row_batch_range_prepare_error_free error:
                                                                Result::Ok _:
                                                                    7
                                                                Result::Err _:
                                                                    7
                                                        Result::Ok owner:
                                                            let range %GuiRgba8888RowBatchRange gui_rgba8888_row_batch_range_owner_range &owner
                                                            let metadata_ok %bool first_range_ok &range
                                                            let next_cursor %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_range_owner_finish_cursor owner
                                                            let cursor_ok %bool eq gui_rgba8888_row_batch_cursor_batch_index &next_cursor 1
                                                            match gui_rgba8888_row_batch_cursor_free next_cursor:
                                                                Result::Ok _:
                                                                    if and metadata_ok cursor_ok 0 8
                                                                Result::Err _:
                                                                    8
```

## partial batch offset metadata

`render2d_row_batch_range_partial_batch_offset_ok` は source policy で `row_start * stride_bytes` と `row_count * stride_bytes` の checked arithmetic を検査する。public application code で二つ目の batch まで進める full runtime case は現 checkpoint の Resource 検査が重くなり過ぎるため、focused runtime は first batch の owner recovery と metadata projection に限定する。

## forged batch owner is not public application surface

descriptor authority / plan authority / continuation status error branches are fixed by source policy because public application code must not be able to forge an owner-bearing batch aggregate.

neplg2:test[compile_fail]
diag_code: type.owner_aggregate.constructor_restricted
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_range" as *
#import "core/result" as *

// render2d_row_batch_range_descriptor_authority_error_ok
// render2d_row_batch_range_plan_authority_error_ok
// render2d_row_batch_range_continuation_status_error_ok
// render2d_row_batch_range_metadata_mismatch_error_ok

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 403
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err _:
                    0
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err _:
                            0
                        Result::Ok plan:
                            let cursor %GuiRgba8888RowBatchCursorOwner GuiRgba8888RowBatchCursorOwner plan 1
                            gui_rgba8888_row_batch_cursor_free cursor
```
