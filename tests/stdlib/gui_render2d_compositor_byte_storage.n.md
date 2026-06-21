# GUI render2d compositor byte storage

このファイルは、F5mc の RGBA8888 compositor byte storage bridge が F5mb compositor batch range owner から F5bz row byte storage owner へ進み、tile / RLE / host present / fallback へ進まないことを固定する。

source policy coverage labels:

- render2d_compositor_byte_storage_facade_ok
- render2d_compositor_byte_storage_exact_copy_ok
- render2d_compositor_byte_storage_metadata_recovery_ok
- render2d_compositor_byte_storage_finish_entry_ok
- render2d_compositor_byte_storage_no_tile_no_platform_no_fallback

## compositor byte storage bridge copies row bytes

[目的/もくてき]:
- F5mb range owner の metadata を保持したまま、lower `gui_rgba8888_row_byte_storage_prepare` を 1 回だけ通すことを確認します。
- success owner の checked byte reader が copied byte storage を読み、finish 後に compositor entry owner を回収できることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_render2d_compositor_byte_storage_contract\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d" as *
#import "alloc/gui/render2d/compositor_batch_range" as *
#import "alloc/gui/render2d/compositor_byte_storage" as *
#import "alloc/gui/render2d/compositor_frame_entry" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/software_surface" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as test

// render2d_compositor_byte_storage_facade_ok
// render2d_compositor_byte_storage_exact_copy_ok
// render2d_compositor_byte_storage_metadata_recovery_ok
// render2d_compositor_byte_storage_finish_entry_ok
// render2d_compositor_byte_storage_no_tile_no_platform_no_fallback

fn free_entry_code %fn GuiRgba8888CompositorFrameEntryOwner fn i32 i32 \entry\code:
    match gui_rgba8888_compositor_frame_entry_owner_free entry:
        Result::Ok _:
            code
        Result::Err _:
            code

fn fail_surface_write %fn GuiRgba8888SoftwareSurfaceWriteError fn i32 i32 \error\code:
    match gui_rgba8888_software_surface_write_error_free error:
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

fn checked_byte %fn &GuiRgba8888CompositorByteStorageOwner fn i32 fn i32 bool \owner\index\expected:
    match gui_rgba8888_compositor_byte_storage_owner_byte_at owner index:
        Result::Err _:
            false
        Result::Ok actual:
            eq actual expected

fn metadata_ok %fn &GuiRgba8888CompositorFrameEntryMetadata bool \metadata:
    and:
        eq gui_rgba8888_compositor_frame_entry_metadata_frame_id metadata 92
        and:
            eq gui_rgba8888_compositor_frame_entry_metadata_width metadata 2
            and:
                eq gui_rgba8888_compositor_frame_entry_metadata_height metadata 1
                and:
                    eq gui_rgba8888_compositor_frame_entry_metadata_row_count metadata 1
                    eq gui_rgba8888_compositor_frame_entry_metadata_batch_count metadata 1

fn bytes_ok %fn &GuiRgba8888CompositorByteStorageOwner bool \owner:
    and:
        eq gui_rgba8888_compositor_byte_storage_owner_byte_count owner 8
        and:
            checked_byte owner 0 11
            and:
                checked_byte owner 1 22
                and:
                    checked_byte owner 2 33
                    and:
                        checked_byte owner 3 44
                        and:
                            checked_byte owner 4 55
                            and:
                                checked_byte owner 5 66
                                and:
                                    checked_byte owner 6 77
                                    checked_byte owner 7 88

fn build_entry %fn void Result GuiRgba8888CompositorFrameEntryOwner i32 \void:
    match gui_rgba8888_software_surface_create 2 1:
        Result::Err _:
            Result::Err 1
        Result::Ok surface0:
            let r0 %u8 cast 11
            let g0 %u8 cast 22
            let b0 %u8 cast 33
            let a0 %u8 cast 44
            let first_color %Rgba8888 rgba8888_new r0 g0 b0 a0
            match gui_rgba8888_software_surface_write_pixel surface0 0 0 first_color:
                Result::Err error:
                    Result::Err fail_surface_write error 2
                Result::Ok surface1:
                    let r1 %u8 cast 55
                    let g1 %u8 cast 66
                    let b1 %u8 cast 77
                    let a1 %u8 cast 88
                    let second_color %Rgba8888 rgba8888_new r1 g1 b1 a1
                    match gui_rgba8888_software_surface_write_pixel surface1 1 0 second_color:
                        Result::Err error:
                            Result::Err fail_surface_write error 3
                        Result::Ok surface2:
                            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface2
                            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                                Result::Err error:
                                    Result::Err fail_dirty_push error 4
                                Result::Ok dirty_owner:
                                    let config %GuiRgba8888CompositorFrameEntryConfig gui_rgba8888_compositor_frame_entry_config 92 1
                                    match gui_rgba8888_compositor_frame_entry_prepare dirty_owner config:
                                        Result::Err error:
                                            Result::Err fail_entry_prepare error 5
                                        Result::Ok entry:
                                            Result::Ok entry

fn run_case %fn void i32 \void:
    match build_entry:
        Result::Err code:
            code
        Result::Ok entry:
            match gui_rgba8888_compositor_batch_range_prepare entry:
                Result::Err error:
                    fail_range_prepare error 10
                Result::Ok range_owner:
                    match gui_rgba8888_compositor_byte_storage_prepare range_owner:
                        Result::Err error:
                            fail_byte_prepare error 11
                        Result::Ok storage_owner:
                            let metadata %GuiRgba8888CompositorFrameEntryMetadata gui_rgba8888_compositor_byte_storage_owner_metadata &storage_owner
                            let ok %bool and metadata_ok &metadata bytes_ok &storage_owner
                            match gui_rgba8888_compositor_byte_storage_owner_finish_entry storage_owner:
                                Result::Err error:
                                    fail_byte_finish error 12
                                Result::Ok next_entry:
                                    if ok:
                                        then:
                                            match gui_rgba8888_compositor_batch_range_prepare next_entry:
                                                Result::Ok extra:
                                                    let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_owner_finish_entry extra
                                                    free_entry_code recovered 13
                                                Result::Err complete:
                                                    match gui_rgba8888_compositor_batch_range_error_category_value &complete:
                                                        Option::Some category:
                                                            match category:
                                                                GuiError::InvalidCommand:
                                                                    let recovered %GuiRgba8888CompositorFrameEntryOwner gui_rgba8888_compositor_batch_range_error_finish_entry complete
                                                                    free_entry_code recovered 0
                                                                _:
                                                                    fail_range_prepare complete 14
                                                        Option::None:
                                                            fail_range_prepare complete 15
                                        else free_entry_code next_entry 16

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_render2d_compositor_byte_storage_contract"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
