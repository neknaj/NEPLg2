# GUI render2d row batch cursor

このファイルは、F5bw の RGBA8888 row batch cursor が row batch plan owner を batch metadata stream として 1 step ずつ進め、byte payload / host present / fallback に進まないことを固定する。

source policy coverage labels:

- render2d_row_batch_cursor_facade_ok
- render2d_row_batch_cursor_start_revalidates_plan_ok
- render2d_row_batch_cursor_empty_dirty_complete_ok
- render2d_row_batch_cursor_full_dirty_first_descriptor_ok
- render2d_row_batch_cursor_continuation_sequence_ok
- render2d_row_batch_cursor_owner_constructor_restricted
- render2d_row_batch_cursor_no_platform_no_fallback

## empty dirty reports complete status

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_batch_cursor_empty\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "core/result" as *
#import "std/test" as test

// render2d_row_batch_cursor_facade_ok
// render2d_row_batch_cursor_start_revalidates_plan_ok
// render2d_row_batch_cursor_empty_dirty_complete_ok
// render2d_row_batch_cursor_no_platform_no_fallback

fn fail_frame_error %fn GuiRgba8888BitmapFramePrepareError i32 \error:
    match gui_rgba8888_bitmap_frame_prepare_error_free error:
        Result::Err _:
            9
        Result::Ok _:
            9

fn fail_plan_error %fn GuiRgba8888RowBatchPlanPrepareError i32 \error:
    match gui_rgba8888_row_batch_plan_prepare_error_free error:
        Result::Err _:
            8
        Result::Ok _:
            8

fn fail_cursor_start_error %fn GuiRgba8888RowBatchCursorStartError i32 \error:
    match gui_rgba8888_row_batch_cursor_start_error_free error:
        Result::Err _:
            7
        Result::Ok _:
            7

fn run_case %fn void i32 \void:
    match gui_rgba8888_software_surface_create 3 2:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 61
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err error:
                    fail_frame_error error
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 2
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err error:
                            fail_plan_error error
                        Result::Ok plan:
                            match gui_rgba8888_row_batch_cursor_start plan:
                                Result::Err error:
                                    fail_cursor_start_error error
                                Result::Ok cursor:
                                    match gui_rgba8888_row_batch_cursor_status &cursor:
                                        Result::Err _:
                                            match gui_rgba8888_row_batch_cursor_free cursor:
                                                Result::Err _:
                                                    6
                                                Result::Ok _:
                                                    6
                                        Result::Ok status:
                                            match status:
                                                GuiRgba8888RowBatchCursorStatus::Ready:
                                                    match gui_rgba8888_row_batch_cursor_free cursor:
                                                        Result::Err _:
                                                            5
                                                        Result::Ok _:
                                                            5
                                                GuiRgba8888RowBatchCursorStatus::Complete:
                                                    match gui_rgba8888_row_batch_cursor_free cursor:
                                                        Result::Err _:
                                                            4
                                                        Result::Ok _:
                                                            0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_batch_cursor_empty"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## full dirty emits continuation descriptors

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_row_batch_cursor_full\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

// render2d_row_batch_cursor_full_dirty_first_descriptor_ok
// render2d_row_batch_cursor_continuation_sequence_ok

fn descriptor_ok %fn &GuiRgba8888RowBatchDescriptor fn i32 fn i32 fn i32 bool \descriptor\batch_index\row_start\row_count:
    let actual_batch_index %i32 gui_rgba8888_row_batch_descriptor_batch_index descriptor
    let actual_row_start %i32 gui_rgba8888_row_batch_descriptor_row_start descriptor
    let actual_row_count %i32 gui_rgba8888_row_batch_descriptor_row_count descriptor
    and:
        eq actual_batch_index batch_index
        and:
            eq actual_row_start row_start
            eq actual_row_count row_count

fn free_cursor_code %fn GuiRgba8888RowBatchCursorOwner fn i32 i32 \cursor\code:
    match gui_rgba8888_row_batch_cursor_free cursor:
        Result::Err _:
            code
        Result::Ok _:
            code

fn next_batch_error_code %fn GuiRgba8888RowBatchCursorStepError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_cursor_step_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn finish_batch_or_code %fn GuiRgba8888RowBatchCursorBatchOwner fn i32 fn i32 fn i32 fn i32 Result GuiRgba8888RowBatchCursorOwner i32 \batch\batch_index\row_start\row_count\code:
    let descriptor %GuiRgba8888RowBatchDescriptor gui_rgba8888_row_batch_cursor_batch_descriptor &batch
    if descriptor_ok &descriptor batch_index row_start row_count:
        then:
            Result::Ok gui_rgba8888_row_batch_cursor_batch_finish_cursor batch
        else:
            match gui_rgba8888_row_batch_cursor_batch_free batch:
                Result::Err _:
                    Result::Err code
                Result::Ok _:
                    Result::Err code

fn expect_complete_code %fn GuiRgba8888RowBatchCursorOwner i32 \cursor:
    match gui_rgba8888_row_batch_cursor_status &cursor:
        Result::Err _:
            free_cursor_code cursor 14
        Result::Ok status:
            match status:
                GuiRgba8888RowBatchCursorStatus::Ready:
                    free_cursor_code cursor 15
                GuiRgba8888RowBatchCursorStatus::Complete:
                    free_cursor_code cursor 0

fn run_third_batch %fn GuiRgba8888RowBatchCursorOwner i32 \cursor2:
    match gui_rgba8888_row_batch_cursor_next_batch cursor2:
        Result::Err error:
            next_batch_error_code error 12
        Result::Ok batch3:
            match finish_batch_or_code batch3 2 4 1 13:
                Result::Err code:
                    code
                Result::Ok cursor3:
                    expect_complete_code cursor3

fn run_second_batch %fn GuiRgba8888RowBatchCursorOwner i32 \cursor1:
    match gui_rgba8888_row_batch_cursor_next_batch cursor1:
        Result::Err error:
            next_batch_error_code error 10
        Result::Ok batch2:
            match finish_batch_or_code batch2 1 2 2 11:
                Result::Err code:
                    code
                Result::Ok cursor2:
                    run_third_batch cursor2

fn run_first_batch %fn GuiRgba8888RowBatchCursorOwner i32 \cursor0:
    match gui_rgba8888_row_batch_cursor_next_batch cursor0:
        Result::Err error:
            next_batch_error_code error 6
        Result::Ok batch1:
            match finish_batch_or_code batch1 0 0 2 8:
                Result::Err code:
                    code
                Result::Ok cursor1:
                    run_second_batch cursor1

fn run_case %fn void i32 \void:
    match gui_rgba8888_software_surface_create 4 5:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err error:
                    match gui_rgba8888_software_surface_dirty_push_error_free error:
                        Result::Err _:
                            2
                        Result::Ok _:
                            2
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 62
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err error:
                            match gui_rgba8888_bitmap_frame_prepare_error_free error:
                                Result::Err _:
                                    3
                                Result::Ok _:
                                    3
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 2
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err error:
                                    match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                        Result::Err _:
                                            4
                                        Result::Ok _:
                                            4
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err error:
                                            match gui_rgba8888_row_batch_cursor_start_error_free error:
                                                Result::Err _:
                                                    5
                                                Result::Ok _:
                                                    5
                                        Result::Ok cursor0:
                                            run_first_batch cursor0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_row_batch_cursor_full"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## public cursor owner constructor boundary

この compile-fail は、通常の application code が cursor owner を直 constructor で forged できないことを固定する。cursor が必要な場合は `start` を使う。

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
#import "core/result" as *

// render2d_row_batch_cursor_owner_constructor_restricted

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 1 1:
        Result::Err _:
            0
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 1
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err error:
                    match gui_rgba8888_bitmap_frame_prepare_error_free error:
                        Result::Err _:
                            1
                        Result::Ok _:
                            1
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err error:
                            match gui_rgba8888_row_batch_plan_prepare_error_free error:
                                Result::Err _:
                                    2
                                Result::Ok _:
                                    2
                        Result::Ok plan:
                            let cursor %GuiRgba8888RowBatchCursorOwner GuiRgba8888RowBatchCursorOwner plan 0
                            match gui_rgba8888_row_batch_cursor_free cursor:
                                Result::Err _:
                                    3
                                Result::Ok _:
                                    3
```
