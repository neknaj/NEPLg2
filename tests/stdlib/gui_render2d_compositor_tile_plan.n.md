# GUI render2d compositor tile plan

このファイルは、F5md の RGBA8888 compositor tile plan bridge が F5mc compositor byte storage owner から F5ca row tile plan owner へ進み、tile payload / RLE / host present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_tile_plan_facade_ok
- render2d_compositor_tile_plan_prepare_ok
- render2d_compositor_tile_plan_descriptor_metadata_ok
- render2d_compositor_tile_plan_invalid_config_recovery_ok
- render2d_compositor_tile_plan_finish_entry_ok
- render2d_compositor_tile_plan_no_payload_no_platform_no_fallback

## compositor tile plan bridge keeps metadata and descriptors

[目的/もくてき]:
- F5mc byte storage owner の metadata を保持したまま、lower `gui_rgba8888_row_tile_plan_prepare` を 1 回だけ通すことを確認します。
- success owner の descriptor access が storage-relative metadata を返し、finish 後に compositor entry owner を回収できることを確認します。
- invalid `tile_rows` でも owner-bearing prepare error から byte storage owner を回収し、metadata と category を失わないことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_tile_plan_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "alloc/gui/render2d/compositor_batch_range" as *
#import "alloc/gui/render2d/compositor_byte_storage" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/compositor_tile_plan" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/row_tile_plan" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_tile_plan_facade_ok
// render2d_compositor_tile_plan_prepare_ok
// render2d_compositor_tile_plan_descriptor_metadata_ok
// render2d_compositor_tile_plan_invalid_config_recovery_ok
// render2d_compositor_tile_plan_finish_entry_ok
// render2d_compositor_tile_plan_no_payload_no_platform_no_fallback

fn free_entry_code %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_frame_entry_owner_free entry:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_dirty_push %fn GuiRgba8888SoftwareSurfaceDirtyPushError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_dirty_push_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_entry_prepare %fn GuiRgba8888CompositorFrameEntryPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_frame_entry_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_range_prepare %fn GuiRgba8888CompositorBatchRangeError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_batch_range_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_byte_prepare %fn GuiRgba8888CompositorByteStoragePrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_byte_storage_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_byte_finish %fn GuiRgba8888CompositorByteStorageFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_byte_storage_finish_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_tile_prepare %fn GuiRgba8888CompositorTilePlanPrepareError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_plan_prepare_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_tile_finish %fn GuiRgba8888CompositorTilePlanFinishError fn i32 i32 \error\code:
    match gui_rgba8888_compositor_tile_plan_finish_error_free error:
        Result::Ok _:
            code
        Result::Err _:
            code

fn finish_tile_plan_code %fn GuiRgba8888CompositorTilePlanOwner fn i32 i32 \owner\code:
    match gui_rgba8888_compositor_tile_plan_owner_finish_entry owner:
        Result::Err error:
            fail_tile_finish error code
        Result::Ok entry:
            free_entry_code entry code

fn metadata_ok %fn &GuiRgba8888CompositorFrameEntryMetadata fn i32 bool \metadata\frame_id:
    and:
        eq gui_rgba8888_compositor_frame_entry_metadata_frame_id metadata frame_id
        and:
            eq gui_rgba8888_compositor_frame_entry_metadata_width metadata 2
            and:
                eq gui_rgba8888_compositor_frame_entry_metadata_height metadata 5
                and:
                    eq gui_rgba8888_compositor_frame_entry_metadata_row_start metadata 0
                    and:
                        eq gui_rgba8888_compositor_frame_entry_metadata_row_count metadata 5
                        and:
                            eq gui_rgba8888_compositor_frame_entry_metadata_batch_count metadata 1
                            eq gui_rgba8888_compositor_frame_entry_metadata_max_rows_per_batch metadata 5

fn descriptor_matches %fn &GuiRgba8888CompositorTilePlanOwner fn i32 fn i32 fn i32 fn i32 fn i32 bool \owner\index\row_start\row_count\byte_offset\byte_count:
    match gui_rgba8888_compositor_tile_plan_owner_descriptor_at owner index:
        Result::Err _:
            false
        Result::Ok descriptor:
            and:
                eq gui_rgba8888_row_tile_descriptor_tile_index &descriptor index
                and:
                    eq gui_rgba8888_row_tile_descriptor_row_start &descriptor row_start
                    and:
                        eq gui_rgba8888_row_tile_descriptor_row_count &descriptor row_count
                        and:
                            eq gui_rgba8888_row_tile_descriptor_byte_offset &descriptor byte_offset
                            eq gui_rgba8888_row_tile_descriptor_byte_count &descriptor byte_count

fn tile_plan_ok %fn &GuiRgba8888CompositorTilePlanOwner bool \owner:
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_tile_plan_owner_metadata owner
    let plan %GuiRgba8888RowTilePlan gui_rgba8888_compositor_tile_plan_owner_plan owner
    and:
        metadata_ok &metadata 93
        and:
            eq gui_rgba8888_row_tile_plan_tile_rows &plan 2
            and:
                eq gui_rgba8888_compositor_tile_plan_owner_tile_count owner 3
                and:
                    eq gui_rgba8888_compositor_tile_plan_owner_byte_count owner 40
                    and:
                        descriptor_matches owner 0 0 2 0 16
                        and:
                            descriptor_matches owner 1 2 2 16 16
                            descriptor_matches owner 2 4 1 32 8

fn invalid_prepare_kind_ok %fn &GuiRgba8888CompositorTilePlanPrepareError bool \error:
    match gui_rgba8888_compositor_tile_plan_prepare_error_kind error:
        GuiRgba8888CompositorTilePlanPrepareErrorKind::RowTilePlanPrepareFailed lower_kind:
            match lower_kind:
                GuiRgba8888RowTilePlanPrepareErrorKind::TileRowsInvalid:
                    true
                _:
                    false

fn invalid_prepare_category_ok %fn &GuiRgba8888CompositorTilePlanPrepareError bool \error:
    match gui_rgba8888_compositor_tile_plan_prepare_error_category_value error:
        Option::Some category:
            match category:
                GuiError::InvalidGeometry:
                    true
                _:
                    false
        Option::None:
            false

fn expect_complete_entry %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_batch_range_prepare entry:
        Result::Ok extra:
            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_owner_finish_entry extra
            free_entry_code recovered code
        Result::Err complete:
            match gui_rgba8888_compositor_batch_range_error_category_value &complete:
                Option::Some category:
                    match category:
                        GuiError::InvalidCommand:
                            let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_error_finish_entry complete
                            free_entry_code recovered 0
                        _:
                            fail_range_prepare complete code
                Option::None:
                    fail_range_prepare complete code

fn build_entry %fn i32 Result GuiRgba8888CompositorFrameEntryOwner i32 \frame_id:
    match gui_rgba8888_software_surface_create 2 5:
        Result::Err _:
            Result::Err 1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err error:
                    Result::Err fail_dirty_push error 2
                Result::Ok dirty_owner:
                    let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config frame_id 5
                    match gui_rgba8888_compositor_frame_entry_prepare dirty_owner config:
                        Result::Err error:
                            Result::Err fail_entry_prepare error 3
                        Result::Ok entry:
                            Result::Ok entry

fn build_byte_storage %fn GuiRgba8888CompositorFrameEntryOwner Result GuiRgba8888CompositorByteStorageOwner i32 \entry:
    match gui_rgba8888_compositor_batch_range_prepare entry:
        Result::Err error:
            Result::Err fail_range_prepare error 10
        Result::Ok range_owner:
            match gui_rgba8888_compositor_byte_storage_prepare range_owner:
                Result::Err error:
                    Result::Err fail_byte_prepare error 11
                Result::Ok storage_owner:
                    Result::Ok storage_owner

fn run_success_from_storage %fn GuiRgba8888CompositorByteStorageOwner i32 \storage_owner:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 2
    match gui_rgba8888_compositor_tile_plan_prepare storage_owner config:
        Result::Err error:
            fail_tile_prepare error 12
        Result::Ok tile_owner:
            let ok %bool tile_plan_ok &tile_owner
            match gui_rgba8888_compositor_tile_plan_owner_finish_entry tile_owner:
                Result::Err error:
                    fail_tile_finish error 13
                Result::Ok entry:
                    if ok:
                        then expect_complete_entry entry 14
                        else free_entry_code entry 15

fn run_success_case %fn void i32 \void:
    match build_entry 93:
        Result::Err code:
            code
        Result::Ok entry:
            match build_byte_storage entry:
                Result::Err code:
                    code
                Result::Ok storage_owner:
                    run_success_from_storage storage_owner

fn recover_invalid_tile_prepare %fn GuiRgba8888CompositorTilePlanPrepareError i32 \error:
    let kind_ok %bool invalid_prepare_kind_ok &error
    let category_ok %bool invalid_prepare_category_ok &error
    let storage %GuiRgba8888CompositorByteStorageOwner gui_rgba8888_compositor_tile_plan_prepare_error_storage error
    let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_byte_storage_owner_metadata &storage
    let metadata_matches %bool metadata_ok &metadata 94
    let ok %bool and kind_ok and category_ok metadata_matches
    match gui_rgba8888_compositor_byte_storage_owner_finish_entry storage:
        Result::Err error:
            fail_byte_finish error 22
        Result::Ok entry:
            if ok:
                then expect_complete_entry entry 23
                else free_entry_code entry 24

fn run_invalid_from_storage %fn GuiRgba8888CompositorByteStorageOwner i32 \storage_owner:
    let config %GuiRgba8888RowTilePlanConfig GuiRgba8888RowTilePlanConfig 0
    match gui_rgba8888_compositor_tile_plan_prepare storage_owner config:
        Result::Err error:
            recover_invalid_tile_prepare error
        Result::Ok tile_owner:
            finish_tile_plan_code tile_owner 21

fn run_invalid_tile_rows_case %fn void i32 \void:
    match build_entry 94:
        Result::Err code:
            code
        Result::Ok entry:
            match build_byte_storage entry:
                Result::Err code:
                    code
                Result::Ok storage_owner:
                    run_invalid_from_storage storage_owner

fn run_case %fn void i32 \void:
    let success %i32 run_success_case
    if ne success 0:
        then success
        else run_invalid_tile_rows_case

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_tile_plan_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
