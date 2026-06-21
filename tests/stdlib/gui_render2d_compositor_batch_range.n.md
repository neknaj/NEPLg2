# GUI render2d compositor batch range

このファイルは、F5mb の RGBA8888 compositor batch range bridge が F5lz entry owner から 1 batch 分の row range metadata を取り出し、row byte storage / tile / RLE / host present / fallback に進まないことを固定する。

source policy coverage labels:

- render2d_compositor_batch_range_facade_ok
- render2d_compositor_batch_range_first_batch_ok
- render2d_compositor_batch_range_continuation_ok
- render2d_compositor_batch_range_complete_cursor_recovery_ok
- render2d_compositor_batch_range_metadata_recovery_ok
- render2d_compositor_batch_range_no_payload_no_platform_no_fallback

## compositor batch range bridge contract

[目的/もくてき]:
- F5lz entry owner の metadata を保持したまま、lower `next_batch` と `row_batch_range_prepare` を 1 回ずつ呼ぶことを確認します。
- success owner から entry owner を再構成し、complete cursor error でも entry owner を回収できることを確認します。
- detailed multi-batch arithmetic は lower row batch range regression で固定し、この focused runtime は bridge の owner recovery に絞ります。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_batch_range_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_batch_range" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_batch_range_facade_ok
// render2d_compositor_batch_range_first_batch_ok
// render2d_compositor_batch_range_continuation_ok
// render2d_compositor_batch_range_complete_cursor_recovery_ok
// render2d_compositor_batch_range_metadata_recovery_ok
// render2d_compositor_batch_range_no_payload_no_platform_no_fallback

fn kind_is_cursor_next_failed %fn GuiRgba8888CompositorBatchRangeErrorKind bool \kind:
    match kind:
        GuiRgba8888CompositorBatchRangeErrorKind::CursorNextBatchFailed _:
            true
        _:
            false

fn category_is_invalid_command %fn Option GuiError bool \category:
    match category:
        Option::Some error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Option::None:
            false

fn free_entry_code %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_frame_entry_owner_free entry:
        Result::Ok _:
            code
        Result::Err _:
            code

fn free_range_error_code %fn GuiRgba8888CompositorBatchRangeError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_batch_range_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn metadata_ok %fn &GuiRgba8888CompositorFrameEntryMetadata bool \metadata:
    and:
        eq gui_rgba8888_compositor_frame_entry_metadata_frame_id metadata 91
        and:
            eq gui_rgba8888_compositor_frame_entry_metadata_width metadata 2
            and:
                eq gui_rgba8888_compositor_frame_entry_metadata_height metadata 1
                and:
                    eq gui_rgba8888_compositor_frame_entry_metadata_row_count metadata 1
                    eq gui_rgba8888_compositor_frame_entry_metadata_batch_count metadata 1

fn range_ok %fn &GuiRgba8888CompositorBatchRangeOwner bool \owner:
    and:
        eq gui_rgba8888_compositor_batch_range_owner_batch_index owner 0
        and:
            eq gui_rgba8888_compositor_batch_range_owner_row_start owner 0
            and:
                eq gui_rgba8888_compositor_batch_range_owner_row_count owner 1
                and:
                    eq gui_rgba8888_compositor_batch_range_owner_stride_bytes owner 8
                    eq gui_rgba8888_compositor_batch_range_owner_byte_count owner 8

fn build_entry %fn void Result GuiRgba8888CompositorFrameEntryOwner i32 \void:
    match gui_rgba8888_software_surface_create 2 1:
        Result::Err _:
            Result::Err 1
        Result::Ok surface:
            let owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner0 dirty_region_full:
                Result::Err error:
                    match gui_rgba8888_software_surface_dirty_push_error_free error:
                        Result::Ok _:
                            Result::Err 2
                        Result::Err _:
                            Result::Err 2
                Result::Ok dirty:
                    let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config 91 1
                    match gui_rgba8888_compositor_frame_entry_prepare dirty config:
                        Result::Err error:
                            match gui_rgba8888_compositor_frame_entry_prepare_error_free error:
                                Result::Ok _:
                                    Result::Err 3
                                Result::Err _:
                                    Result::Err 3
                        Result::Ok entry:
                            Result::Ok entry

fn run_case %fn void i32 \void:
    match build_entry:
        Result::Err code:
            code
        Result::Ok entry:
            match gui_rgba8888_compositor_batch_range_prepare entry:
                Result::Err error:
                    free_range_error_code error 10
                Result::Ok owner:
                    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_batch_range_owner_metadata &owner
                    let ok %bool and range_ok &owner metadata_ok &metadata
                    let next_entry %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_owner_finish_entry owner
                    if ok:
                        then:
                            match gui_rgba8888_compositor_batch_range_prepare next_entry:
                                Result::Ok extra:
                                    let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_owner_finish_entry extra
                                    free_entry_code recovered 11
                                Result::Err error:
                                    let kind %GuiRgba8888CompositorBatchRangeErrorKind gui_rgba8888_compositor_batch_range_error_kind &error
                                    let category %Option GuiError gui_rgba8888_compositor_batch_range_error_category_value &error
                                    let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_error_finish_entry error
                                    if and kind_is_cursor_next_failed kind category_is_invalid_command category:
                                        then free_entry_code recovered 0
                                        else free_entry_code recovered 12
                        else free_entry_code next_entry 13

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_batch_range_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
