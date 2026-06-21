# GUI render2d compositor frame entry

このファイルは、F5lz の RGBA8888 compositor frame entry owner が dirty surface owner を bitmap frame / row batch plan / row batch cursor の入口へ正規化し、row byte storage、tile / RLE、host present、platform transport、fallback に進まないことを固定する。

source policy coverage labels:

- render2d_compositor_frame_entry_facade_ok
- render2d_compositor_frame_entry_empty_dirty_complete_ok
- render2d_compositor_frame_entry_full_dirty_metadata_ok
- render2d_compositor_frame_entry_invalid_frame_id_recovery_ok
- render2d_compositor_frame_entry_invalid_max_rows_recovery_ok
- render2d_compositor_frame_entry_no_transport_no_platform_no_fallback

## compositor frame entry owner contract

[目的/もくてき]:
- F5ly 後の dirty owner を、既存の F5bu / F5bv / F5bw contract に[順番/じゅんばん]に[通/とお]すことを確認します。
- config が[不正/ふせい]でも ownerless error にせず、lower owner-bearing error から元の dirty owner を[回収/かいしゅう]できることを確認します。
- success metadata が row batch cursor start の[前/まえ]に plan から[読/よ]まれ、cursor が有効な continuation として[返/かえ]ることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_frame_entry_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_frame_entry_facade_ok
// render2d_compositor_frame_entry_empty_dirty_complete_ok
// render2d_compositor_frame_entry_full_dirty_metadata_ok
// render2d_compositor_frame_entry_invalid_frame_id_recovery_ok
// render2d_compositor_frame_entry_invalid_max_rows_recovery_ok
// render2d_compositor_frame_entry_no_transport_no_platform_no_fallback

fn entry_kind_is_bitmap_frame_id_invalid %fn GuiRgba8888CompositorFrameEntryPrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888CompositorFrameEntryPrepareErrorKind::BitmapFramePrepareFailed lower_kind:
            match lower_kind:
                GuiRgba8888BitmapFramePrepareErrorKind::FrameIdInvalid:
                    true
                _:
                    false
        _:
            false

fn entry_kind_is_plan_max_rows_invalid %fn GuiRgba8888CompositorFrameEntryPrepareErrorKind bool \kind:
    match kind:
        GuiRgba8888CompositorFrameEntryPrepareErrorKind::RowBatchPlanPrepareFailed lower_kind:
            match lower_kind:
                GuiRgba8888RowBatchPlanPrepareErrorKind::MaxRowsPerBatchInvalid:
                    true
                _:
                    false
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

fn free_entry_error_code %fn GuiRgba8888CompositorFrameEntryPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_frame_entry_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn free_entry_code %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_frame_entry_owner_free entry:
        Result::Err _:
            code
        Result::Ok _:
            code

fn free_dirty_owner_code %fn GuiRgba8888SoftwareSurfaceDirtyOwner fn i32 i32 \owner\code:
    match gui_rgba8888_software_surface_dirty_owner_free owner:
        Result::Err _:
            code
        Result::Ok _:
            code

fn free_cursor_code %fn GuiRgba8888RowBatchCursorOwner fn i32 i32 \cursor\code:
    match gui_rgba8888_row_batch_cursor_free cursor:
        Result::Err _:
            code
        Result::Ok _:
            code

fn free_batch_code %fn GuiRgba8888RowBatchCursorBatchOwner fn i32 i32 \batch\code:
    match gui_rgba8888_row_batch_cursor_batch_free batch:
        Result::Err _:
            code
        Result::Ok _:
            code

fn cursor_step_error_code %fn GuiRgba8888RowBatchCursorStepError fn i32 i32 \error\code:
    match gui_rgba8888_row_batch_cursor_step_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn dirty_owner_empty %fn i32 fn i32 Result GuiRgba8888SoftwareSurfaceDirtyOwner i32 \width\height:
    match gui_rgba8888_software_surface_create width height:
        Result::Err _:
            Result::Err 1
        Result::Ok surface:
            Result::Ok gui_rgba8888_software_surface_dirty_owner_from_surface surface

fn dirty_owner_full %fn i32 fn i32 Result GuiRgba8888SoftwareSurfaceDirtyOwner i32 \width\height:
    match dirty_owner_empty width height:
        Result::Err code:
            Result::Err code
        Result::Ok owner:
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked owner dirty_region_full:
                Result::Err error:
                    match gui_rgba8888_software_surface_dirty_push_error_free error:
                        Result::Err _:
                            Result::Err 2
                        Result::Ok _:
                            Result::Err 2
                Result::Ok next:
                    Result::Ok next

fn metadata_shape_ok %fn &GuiRgba8888CompositorFrameEntryMetadata fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 bool \metadata\frame_id\width\height\row_start\row_count\batch_count\max_rows:
    and:
        eq gui_rgba8888_compositor_frame_entry_metadata_frame_id metadata frame_id
        and:
            eq gui_rgba8888_compositor_frame_entry_metadata_width metadata width
            and:
                eq gui_rgba8888_compositor_frame_entry_metadata_height metadata height
                and:
                    eq gui_rgba8888_compositor_frame_entry_metadata_row_start metadata row_start
                    and:
                        eq gui_rgba8888_compositor_frame_entry_metadata_row_count metadata row_count
                        and:
                            eq gui_rgba8888_compositor_frame_entry_metadata_batch_count metadata batch_count
                            eq gui_rgba8888_compositor_frame_entry_metadata_max_rows_per_batch metadata max_rows

fn descriptor_ok %fn &GuiRgba8888RowBatchDescriptor fn i32 fn i32 fn i32 bool \descriptor\batch_index\row_start\row_count:
    and:
        eq gui_rgba8888_row_batch_descriptor_batch_index descriptor batch_index
        and:
            eq gui_rgba8888_row_batch_descriptor_row_start descriptor row_start
            eq gui_rgba8888_row_batch_descriptor_row_count descriptor row_count

fn empty_dirty_case %fn void i32 \void:
    match dirty_owner_empty 3 2:
        Result::Err code:
            code
        Result::Ok owner:
            let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config 71 2
            match gui_rgba8888_compositor_frame_entry_prepare owner config:
                Result::Err error:
                    free_entry_error_code error 3
                Result::Ok entry:
                    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_frame_entry_metadata &entry
                    if metadata_shape_ok &metadata 71 3 2 0 0 0 2:
                        then:
                            let cursor %GuiRgba8888RowBatchCursorOwner gui_rgba8888_compositor_frame_entry_finish_cursor entry
                            match gui_rgba8888_row_batch_cursor_status &cursor:
                                Result::Err _:
                                    free_cursor_code cursor 5
                                Result::Ok status:
                                    match status:
                                        GuiRgba8888RowBatchCursorStatus::Ready:
                                            free_cursor_code cursor 6
                                        GuiRgba8888RowBatchCursorStatus::Complete:
                                            free_cursor_code cursor 0
                        else free_entry_code entry 4

fn full_dirty_case %fn void i32 \void:
    match dirty_owner_full 4 5:
        Result::Err code:
            code
        Result::Ok owner:
            let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config 72 2
            match gui_rgba8888_compositor_frame_entry_prepare owner config:
                Result::Err error:
                    free_entry_error_code error 10
                Result::Ok entry:
                    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_frame_entry_metadata &entry
                    if metadata_shape_ok &metadata 72 4 5 0 5 3 2:
                        then:
                            let cursor %GuiRgba8888RowBatchCursorOwner gui_rgba8888_compositor_frame_entry_finish_cursor entry
                            match gui_rgba8888_row_batch_cursor_next_batch cursor:
                                Result::Err error:
                                    cursor_step_error_code error 12
                                Result::Ok batch:
                                    let descriptor %GuiRgba8888RowBatchDescriptor gui_rgba8888_row_batch_cursor_batch_descriptor &batch
                                    if descriptor_ok &descriptor 0 0 2:
                                        then:
                                            let next_cursor %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_cursor_batch_finish_cursor batch
                                            free_cursor_code next_cursor 0
                                        else free_batch_code batch 13
                        else free_entry_code entry 11

fn invalid_frame_id_case %fn void i32 \void:
    match dirty_owner_empty 1 1:
        Result::Err code:
            code
        Result::Ok owner:
            let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config 0 1
            match gui_rgba8888_compositor_frame_entry_prepare owner config:
                Result::Ok entry:
                    free_entry_code entry 20
                Result::Err error:
                    let kind %GuiRgba8888CompositorFrameEntryPrepareErrorKind gui_rgba8888_compositor_frame_entry_prepare_error_kind &error
                    let category %Option GuiError gui_rgba8888_compositor_frame_entry_prepare_error_category_value &error
                    let ok_flag %bool and entry_kind_is_bitmap_frame_id_invalid kind category_is_invalid_command category
                    let recovered %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_compositor_frame_entry_prepare_error_owner error
                    if ok_flag:
                        then free_dirty_owner_code recovered 0
                        else free_dirty_owner_code recovered 21

fn invalid_max_rows_case %fn void i32 \void:
    match dirty_owner_empty 2 2:
        Result::Err code:
            code
        Result::Ok owner:
            let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config 73 0
            match gui_rgba8888_compositor_frame_entry_prepare owner config:
                Result::Ok entry:
                    free_entry_code entry 30
                Result::Err error:
                    let kind %GuiRgba8888CompositorFrameEntryPrepareErrorKind gui_rgba8888_compositor_frame_entry_prepare_error_kind &error
                    let category %Option GuiError gui_rgba8888_compositor_frame_entry_prepare_error_category_value &error
                    let ok_flag %bool and entry_kind_is_plan_max_rows_invalid kind category_is_invalid_command category
                    let recovered %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_compositor_frame_entry_prepare_error_owner error
                    if ok_flag:
                        then free_dirty_owner_code recovered 0
                        else free_dirty_owner_code recovered 31

fn run_case %fn void i32 \void:
    let empty_code %i32 empty_dirty_case
    if ne empty_code 0:
        then empty_code
        else:
            let full_code %i32 full_dirty_case
            if ne full_code 0:
                then full_code
                else:
                    let invalid_frame_code %i32 invalid_frame_id_case
                    if ne invalid_frame_code 0:
                        then invalid_frame_code
                        else invalid_max_rows_case

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_frame_entry_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
