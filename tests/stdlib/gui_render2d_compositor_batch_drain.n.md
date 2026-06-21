# GUI render2d compositor batch drain

このファイルは、F5ma の RGBA8888 compositor batch drain continuation が F5lz entry owner を row batch drain に 1 回だけ委譲し、row range / row byte / tile / RLE / host present / fallback に進まないことを固定する。

source policy coverage labels:

- render2d_compositor_batch_drain_facade_ok
- render2d_compositor_batch_drain_empty_negative_complete_ok
- render2d_compositor_batch_drain_full_budget_continuation_ok
- render2d_compositor_batch_drain_negative_budget_recovery_ok
- render2d_compositor_batch_drain_metadata_recovery_ok
- render2d_compositor_batch_drain_no_payload_no_platform_no_fallback

## compositor batch drain continuation contract

[目的/もくてき]:
- F5lz entry owner の metadata を保持したまま、F5bx row batch drain へ scheduler budget を 1 回だけ委譲することを確認します。
- complete cursor は負 budget でも lower drain の契約どおり Completed になることを確認します。
- ready cursor への負 budget は owner-bearing error になり、metadata と entry owner を回収できることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_batch_drain_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/compositor_batch_drain" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_batch_drain_facade_ok
// render2d_compositor_batch_drain_empty_negative_complete_ok
// render2d_compositor_batch_drain_full_budget_continuation_ok
// render2d_compositor_batch_drain_negative_budget_recovery_ok
// render2d_compositor_batch_drain_metadata_recovery_ok
// render2d_compositor_batch_drain_no_payload_no_platform_no_fallback

fn status_is_completed %fn GuiRgba8888CompositorBatchDrainStatus bool \status:
    match status:
        GuiRgba8888CompositorBatchDrainStatus::Completed:
            true
        _:
            false

fn status_is_exhausted %fn GuiRgba8888CompositorBatchDrainStatus bool \status:
    match status:
        GuiRgba8888CompositorBatchDrainStatus::StepBudgetExhausted:
            true
        _:
            false

fn kind_is_row_batch_drain_failed %fn GuiRgba8888CompositorBatchDrainErrorKind bool \kind:
    match kind:
        GuiRgba8888CompositorBatchDrainErrorKind::RowBatchDrainFailed _:
            true

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
        Result::Err _:
            code
        Result::Ok _:
            code

fn free_prepare_error_code %fn GuiRgba8888CompositorFrameEntryPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_frame_entry_prepare_error_free error:
        Result::Err _:
            code
        Result::Ok _:
            code

fn free_drain_error_code %fn GuiRgba8888CompositorBatchDrainError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_batch_drain_error_free error:
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

fn entry_from_dirty %fn GuiRgba8888SoftwareSurfaceDirtyOwner fn i32 fn i32 Result GuiRgba8888CompositorFrameEntryOwner i32 \owner\frame_id\max_rows:
    let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config frame_id max_rows
    match gui_rgba8888_compositor_frame_entry_prepare owner config:
        Result::Err error:
            match gui_rgba8888_compositor_frame_entry_prepare_error_free error:
                Result::Err _:
                    Result::Err 3
                Result::Ok _:
                    Result::Err 3
        Result::Ok entry:
            Result::Ok entry

fn metadata_ok %fn &GuiRgba8888CompositorFrameEntryMetadata fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 bool \metadata\frame_id\width\height\row_start\row_count\batch_count\max_rows:
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

fn empty_negative_complete_case %fn void i32 \void:
    match dirty_owner_empty 3 2:
        Result::Err code:
            code
        Result::Ok owner:
            match entry_from_dirty owner 81 2:
                Result::Err code:
                    code
                Result::Ok entry:
                    match gui_rgba8888_compositor_batch_drain_budget entry sub 0 1:
                        Result::Err error:
                            free_drain_error_code error 10
                        Result::Ok terminal:
                            let status %GuiRgba8888CompositorBatchDrainStatus gui_rgba8888_compositor_batch_drain_terminal_status &terminal
                            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_batch_drain_terminal_metadata &terminal
                            let count_ok %bool eq gui_rgba8888_compositor_batch_drain_terminal_emitted_count &terminal 0
                            let status_ok %bool status_is_completed status
                            let metadata_flag %bool metadata_ok &metadata 81 3 2 0 0 0 2
                            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_drain_terminal_finish_entry terminal
                            if and status_ok and count_ok metadata_flag:
                                then free_entry_code recovered 0
                                else free_entry_code recovered 11

fn full_budget_continuation_case %fn void i32 \void:
    match dirty_owner_full 4 5:
        Result::Err code:
            code
        Result::Ok owner:
            match entry_from_dirty owner 82 2:
                Result::Err code:
                    code
                Result::Ok entry:
                    match gui_rgba8888_compositor_batch_drain_budget entry 1:
                        Result::Err error:
                            free_drain_error_code error 20
                        Result::Ok terminal:
                            let first_status %GuiRgba8888CompositorBatchDrainStatus gui_rgba8888_compositor_batch_drain_terminal_status &terminal
                            let first_metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_batch_drain_terminal_metadata &terminal
                            let first_count_ok %bool eq gui_rgba8888_compositor_batch_drain_terminal_emitted_count &terminal 1
                            let first_status_ok %bool status_is_exhausted first_status
                            let first_metadata_ok %bool metadata_ok &first_metadata 82 4 5 0 5 3 2
                            let next_entry %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_drain_terminal_finish_entry terminal
                            if and first_status_ok and first_count_ok first_metadata_ok:
                                then:
                                    match gui_rgba8888_compositor_batch_drain_budget next_entry 10:
                                        Result::Err error:
                                            free_drain_error_code error 21
                                        Result::Ok done:
                                            let done_status %GuiRgba8888CompositorBatchDrainStatus gui_rgba8888_compositor_batch_drain_terminal_status &done
                                            let done_count_ok %bool eq gui_rgba8888_compositor_batch_drain_terminal_emitted_count &done 2
                                            let done_status_ok %bool status_is_completed done_status
                                            let done_entry %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_drain_terminal_finish_entry done
                                            if and done_status_ok done_count_ok:
                                                then free_entry_code done_entry 0
                                                else free_entry_code done_entry 22
                                else free_entry_code next_entry 23

fn negative_budget_recovery_case %fn void i32 \void:
    match dirty_owner_full 2 2:
        Result::Err code:
            code
        Result::Ok owner:
            match entry_from_dirty owner 83 1:
                Result::Err code:
                    code
                Result::Ok entry:
                    match gui_rgba8888_compositor_batch_drain_budget entry sub 0 1:
                        Result::Ok terminal:
                            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_drain_terminal_finish_entry terminal
                            free_entry_code recovered 30
                        Result::Err error:
                            let kind %GuiRgba8888CompositorBatchDrainErrorKind gui_rgba8888_compositor_batch_drain_error_kind &error
                            let category %Option GuiError gui_rgba8888_compositor_batch_drain_error_category_value &error
                            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_batch_drain_error_metadata &error
                            let kind_ok %bool kind_is_row_batch_drain_failed kind
                            let category_ok %bool category_is_invalid_command category
                            let metadata_flag %bool metadata_ok &metadata 83 2 2 0 2 2 1
                            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_drain_error_finish_entry error
                            if and kind_ok and category_ok metadata_flag:
                                then free_entry_code recovered 0
                                else free_entry_code recovered 31

fn run_case %fn void i32 \void:
    let empty_code %i32 empty_negative_complete_case
    if ne empty_code 0:
        then empty_code
        else:
            let full_code %i32 full_budget_continuation_case
            if ne full_code 0:
                then full_code
                else negative_budget_recovery_case

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_batch_drain_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
