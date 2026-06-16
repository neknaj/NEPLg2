# GUI render2d row tile RLE cursor

このファイルは、F5cc の RGBA8888 row tile RLE cursor が既存 copied row tile payload view の上で streaming run を返し、encoded buffer / host present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_row_tile_rle_facade_ok
- render2d_row_tile_rle_streaming_cursor_ok
- render2d_row_tile_rle_pixel_run_sequence_ok
- render2d_row_tile_rle_complete_error_owner_recovery_ok
- render2d_row_tile_rle_checked_pixel_channel_offsets_ok
- render2d_row_tile_rle_payload_read_error_wrapped_ok
- render2d_row_tile_rle_no_encoded_buffer_no_vec
- render2d_row_tile_rle_no_platform_no_fallback

## row tile RLE cursor reads RGBA8888 pixel runs from tile payload

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_tile_rle_cursor\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as render2d
#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_range" as *
#import "alloc/gui/render2d/row_byte_storage" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/row_tile_payload" as *
#import "alloc/gui/render2d/row_tile_rle" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_row_tile_rle_facade_ok
// render2d_row_tile_rle_streaming_cursor_ok
// render2d_row_tile_rle_pixel_run_sequence_ok
// render2d_row_tile_rle_complete_error_owner_recovery_ok
// render2d_row_tile_rle_checked_pixel_channel_offsets_ok
// render2d_row_tile_rle_payload_read_error_wrapped_ok
// render2d_row_tile_rle_no_encoded_buffer_no_vec
// render2d_row_tile_rle_no_platform_no_fallback

fn fail_surface_write %fn GuiRgba8888SoftwareSurfaceWriteError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_write_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_dirty_push %fn GuiRgba8888SoftwareSurfaceDirtyPushError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_dirty_push_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_frame %fn GuiRgba8888BitmapFramePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_bitmap_frame_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_plan %fn GuiRgba8888RowBatchPlanPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_plan_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_cursor_start %fn GuiRgba8888RowBatchCursorStartError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_cursor_start_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_cursor_step %fn GuiRgba8888RowBatchCursorStepError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_cursor_step_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_range %fn GuiRgba8888RowBatchRangePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_range_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_storage %fn GuiRgba8888RowByteStoragePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_byte_storage_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_tile_plan %fn GuiRgba8888RowTilePlanPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_plan_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_payload %fn GuiRgba8888RowTilePayloadPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_payload_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_rle_start %fn GuiRgba8888RowTileRleStartError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_start_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn fail_rle_step %fn GuiRgba8888RowTileRleStepError fn i32 i32 \error\code:
    match gui_rgba8888_row_tile_rle_step_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn color_eq %fn &Rgba8888 fn &Rgba8888 bool \left\right:
    and:
        eq rgba8888_r left rgba8888_r right
        and:
            eq rgba8888_g left rgba8888_g right
            and:
                eq rgba8888_b left rgba8888_b right
                eq rgba8888_a left rgba8888_a right

fn run_eq %fn &GuiRgba8888RowTileRleRun fn i32 fn i32 fn &Rgba8888 bool \run\pixel_offset\pixel_count\color:
    let actual_color %Rgba8888 gui_rgba8888_row_tile_rle_run_color run
    and:
        eq gui_rgba8888_row_tile_rle_run_pixel_offset run pixel_offset
        and:
            eq gui_rgba8888_row_tile_rle_run_pixel_count run pixel_count
            color_eq &actual_color color

fn complete_error_ok %fn &GuiRgba8888RowTileRleStepError bool \error:
    match gui_rgba8888_row_tile_rle_step_error_kind error:
        GuiRgba8888RowTileRleStepErrorKind::CursorComplete:
            true
        _:
            false

fn finish_complete_error %fn GuiRgba8888RowTileRleStepError fn i32 i32 \error\code:
    if complete_error_ok &error:
        then fail_rle_step error code
        else fail_rle_step error 23

fn drain_complete_cursor %fn GuiRgba8888RowTileRleCursorOwner i32 \cursor:
    match gui_rgba8888_row_tile_rle_cursor_next_run cursor:
        Result::Err error:
            finish_complete_error error 0
        Result::Ok step:
            let next_cursor %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_step_finish_cursor step
            match gui_rgba8888_row_tile_rle_cursor_free next_cursor:
                Result::Err _:
                    24
                Result::Ok _:
                    24

fn run_third %fn GuiRgba8888RowTileRleCursorOwner fn Rgba8888 i32 \cursor\color_a:
    match gui_rgba8888_row_tile_rle_cursor_next_run cursor:
        Result::Err error:
            fail_rle_step error 20
        Result::Ok step:
            let run %GuiRgba8888RowTileRleRun gui_rgba8888_row_tile_rle_step_run &step
            let next_cursor %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_step_finish_cursor step
            if run_eq &run 3 1 &color_a:
                then drain_complete_cursor next_cursor
                else:
                    match gui_rgba8888_row_tile_rle_cursor_free next_cursor:
                        Result::Err _:
                            21
                        Result::Ok _:
                            21

fn run_second %fn GuiRgba8888RowTileRleCursorOwner fn Rgba8888 fn Rgba8888 i32 \cursor\color_a\color_b:
    match gui_rgba8888_row_tile_rle_cursor_next_run cursor:
        Result::Err error:
            fail_rle_step error 18
        Result::Ok step:
            let run %GuiRgba8888RowTileRleRun gui_rgba8888_row_tile_rle_step_run &step
            let next_cursor %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_step_finish_cursor step
            if run_eq &run 2 1 &color_b:
                then run_third next_cursor color_a
                else:
                    match gui_rgba8888_row_tile_rle_cursor_free next_cursor:
                        Result::Err _:
                            19
                        Result::Ok _:
                            19

fn run_first %fn GuiRgba8888RowTileRleCursorOwner fn Rgba8888 fn Rgba8888 i32 \cursor\color_a\color_b:
    match gui_rgba8888_row_tile_rle_cursor_next_run cursor:
        Result::Err error:
            fail_rle_step error 16
        Result::Ok step:
            let run %GuiRgba8888RowTileRleRun gui_rgba8888_row_tile_rle_step_run &step
            let next_cursor %GuiRgba8888RowTileRleCursorOwner gui_rgba8888_row_tile_rle_step_finish_cursor step
            if run_eq &run 0 2 &color_a:
                then run_second next_cursor color_a color_b
                else:
                    match gui_rgba8888_row_tile_rle_cursor_free next_cursor:
                        Result::Err _:
                            17
                        Result::Ok _:
                            17

fn run_payload %fn GuiRgba8888RowTilePlanOwner fn Rgba8888 fn Rgba8888 i32 \tile_owner\color_a\color_b:
    match gui_rgba8888_row_tile_payload_prepare tile_owner 0:
        Result::Err error:
            fail_payload error 14
        Result::Ok payload:
            match gui_rgba8888_row_tile_rle_cursor_start payload:
                Result::Err error:
                    fail_rle_start error 15
                Result::Ok cursor:
                    run_first cursor color_a color_b

fn run_tile_plan %fn GuiRgba8888RowByteStorageOwner fn Rgba8888 fn Rgba8888 i32 \storage_owner\color_a\color_b:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 1
    match gui_rgba8888_row_tile_plan_prepare storage_owner config:
        Result::Err error:
            fail_tile_plan error 13
        Result::Ok tile_owner:
            run_payload tile_owner color_a color_b

fn run_storage %fn GuiRgba8888RowBatchRangeOwner fn Rgba8888 fn Rgba8888 i32 \range_owner\color_a\color_b:
    match gui_rgba8888_row_byte_storage_prepare range_owner:
        Result::Err error:
            fail_storage error 12
        Result::Ok storage_owner:
            run_tile_plan storage_owner color_a color_b

fn run_pipeline %fn GuiRgba8888SoftwareSurfaceOwner fn Rgba8888 fn Rgba8888 i32 \surface\color_a\color_b:
    let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
    match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
        Result::Err error:
            fail_dirty_push error 4
        Result::Ok dirty_owner:
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 703
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err error:
                    fail_frame error 5
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err error:
                            fail_plan error 6
                        Result::Ok plan:
                            match gui_rgba8888_row_batch_cursor_start plan:
                                Result::Err error:
                                    fail_cursor_start error 7
                                Result::Ok cursor:
                                    match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                        Result::Err error:
                                            fail_cursor_step error 8
                                        Result::Ok batch:
                                            match gui_rgba8888_row_batch_range_prepare batch:
                                                Result::Err error:
                                                    fail_range error 9
                                                Result::Ok range_owner:
                                                    run_storage range_owner color_a color_b

fn write_four_pixels %fn GuiRgba8888SoftwareSurfaceOwner fn Rgba8888 fn Rgba8888 i32 \surface0\color_a\color_b:
    match gui_rgba8888_software_surface_write_pixel surface0 0 0 color_a:
        Result::Err error:
            fail_surface_write error 1
        Result::Ok surface1:
            match gui_rgba8888_software_surface_write_pixel surface1 1 0 color_a:
                Result::Err error:
                    fail_surface_write error 2
                Result::Ok surface2:
                    match gui_rgba8888_software_surface_write_pixel surface2 2 0 color_b:
                        Result::Err error:
                            fail_surface_write error 3
                        Result::Ok surface3:
                            match gui_rgba8888_software_surface_write_pixel surface3 3 0 color_a:
                                Result::Err error:
                                    fail_surface_write error 10
                                Result::Ok surface4:
                                    run_pipeline surface4 color_a color_b

fn run_case %fn void i32 \void:
    let a_r %u8 cast 11
    let a_g %u8 cast 12
    let a_b %u8 cast 13
    let opaque %u8 cast 255
    let b_r %u8 cast 31
    let b_g %u8 cast 32
    let b_b %u8 cast 33
    let color_a %Rgba8888 rgba8888_new a_r a_g a_b opaque
    let color_b %Rgba8888 rgba8888_new b_r b_g b_b opaque
    match gui_rgba8888_software_surface_create 4 1:
        Result::Err _:
            11
        Result::Ok surface:
            write_four_pixels surface color_a color_b

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_tile_rle_cursor"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged row tile RLE cursor owners are not public application surface

通常の application code は row tile RLE cursor owner を直 constructor で作れない。RLE cursor が必要な場合は `gui_rgba8888_row_tile_rle_cursor_start` で payload byte count と RGBA8888 alignment 検査を通す。

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
#import "alloc/gui/render2d/row_byte_storage" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/row_tile_payload" as *
#import "alloc/gui/render2d/row_tile_rle" as *
#import "core/gui/dirty_region" as *
#import "core/result" as *

// render2d_row_tile_rle_streaming_cursor_ok
// render2d_row_tile_rle_no_encoded_buffer_no_vec

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 704
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err _:
                    0
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err _:
                            0
                        Result::Ok plan:
                            match gui_rgba8888_row_batch_cursor_start plan:
                                Result::Err _:
                                    0
                                Result::Ok cursor:
                                    match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                        Result::Err _:
                                            0
                                        Result::Ok batch:
                                            match gui_rgba8888_row_batch_range_prepare batch:
                                                Result::Err _:
                                                    0
                                                Result::Ok range_owner:
                                                    match gui_rgba8888_row_byte_storage_prepare range_owner:
                                                        Result::Err _:
                                                            0
                                                        Result::Ok storage_owner:
                                                            let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 1
                                                            match gui_rgba8888_row_tile_plan_prepare storage_owner config:
                                                                Result::Err _:
                                                                    0
                                                                Result::Ok tile_owner:
                                                                    match gui_rgba8888_row_tile_payload_prepare tile_owner 0:
                                                                        Result::Err _:
                                                                            0
                                                                        Result::Ok payload:
                                                                            let cursor_owner %GuiRgba8888RowTileRleCursorOwner GuiRgba8888RowTileRleCursorOwner payload 1 0
                                                                            match gui_rgba8888_row_tile_rle_cursor_free cursor_owner:
                                                                                Result::Err _:
                                                                                    0
                                                                                Result::Ok _:
                                                                                    0
```
