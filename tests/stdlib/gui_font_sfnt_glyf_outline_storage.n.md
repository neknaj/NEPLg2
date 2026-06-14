# GUI font SFNT glyf outline storage doctests

このファイルは、F5b の simple glyph outline scalar storage owner が typed capacity / limit だけを使い、byte fixture、renderer、rasterizer、platform API に依存しないことを検査する。

## outline storage validates owner allocation boundaries

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn make_bounds %fn GuiGlyphId GuiSfntGlyphBounds \glyph:
    gui_sfnt_glyph_bounds glyph 0 0 10 12

fn make_topology %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphTopology \glyph\contours\points:
    let bounds %GuiSfntGlyphBounds make_bounds glyph
    gui_sfnt_simple_glyph_topology glyph bounds contours points 0 0 0

fn outline_storage_error_kind_is %fn &GuiSfntSimpleGlyphOutlineStorageAllocError fn GuiSfntSimpleGlyphOutlineStorageAllocErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlineStorageAllocErrorKind gui_sfnt_simple_glyph_outline_storage_alloc_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::InvalidCapacity:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotCountOverflow:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotCountOverflow:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotStorageAllocFailed:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotStorageAllocFailed:
                    true
                _:
                    false

fn outline_storage_error_has_capacity_check %fn &GuiSfntSimpleGlyphOutlineStorageAllocError bool \error:
    match gui_sfnt_simple_glyph_outline_storage_alloc_error_capacity_check error:
        Option::Some _check:
            true
        Option::None:
            false

fn outline_storage_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 10
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    let count_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slot_count &storage
                    let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage
                    let cap_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slots_cap &storage
                    gui_sfnt_simple_glyph_outline_storage_free storage
                    and count_ok and len_ok cap_ok
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_storage_invalid_capacity_precedes_limit_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 11
    let bad_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_storage_capacity glyph 2 4 5 4 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 4 5 8
    match gui_sfnt_simple_glyph_outline_storage_alloc &bad_capacity &limit:
        Result::Ok storage:
            gui_sfnt_simple_glyph_outline_storage_free storage
            false
        Result::Err error:
            let kind_ok %bool outline_storage_error_kind_is &error GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::InvalidCapacity
            let no_check_ok %bool not outline_storage_error_has_capacity_check &error
            and kind_ok no_check_ok

fn outline_storage_limit_reject_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 12
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    gui_sfnt_simple_glyph_outline_storage_free storage
                    false
                Result::Err error:
                    let kind_ok %bool outline_storage_error_kind_is &error GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected
                    let check_ok %bool outline_storage_error_has_capacity_check &error
                    and kind_ok check_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_storage_scalar_overflow_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 13
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_storage_capacity glyph 1 1073741823 1073741823 1073741823 2147483646
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 1073741823 1073741823 2147483647
    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
        Result::Ok storage:
            gui_sfnt_simple_glyph_outline_storage_free storage
            false
        Result::Err error:
            let kind_ok %bool outline_storage_error_kind_is &error GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotCountOverflow
            let check_ok %bool outline_storage_error_has_capacity_check &error
            and kind_ok check_ok

fn outline_storage_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 14
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_storage_push_scalar_slot storage0 17:
                        Result::Ok storage1:
                            match gui_sfnt_simple_glyph_outline_storage_push_scalar_slot storage1 23:
                                Result::Ok storage2:
                                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                    let cap_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slots_cap &storage2
                                    let count_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slot_count &storage2
                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                    and len_ok and cap_ok count_ok
                                Result::Err error2:
                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_storage_push_error_storage error2
                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                    false
                        Result::Err error1:
                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_storage_push_error_storage error1
                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                            false
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_storage_push_error_recovery_callback %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn i32 impure fn StdErrorKind bool \storage\value\kind:
    let value_ok %bool eq value 77
    let kind_ok %bool match kind:
        StdErrorKind::CapacityExceeded:
            true
        _:
            false
    gui_sfnt_simple_glyph_outline_storage_free storage
    and value_ok kind_ok

fn outline_storage_push_error_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 15
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    let error %GuiSfntSimpleGlyphOutlineStoragePushError gui_sfnt_simple_glyph_outline_storage_push_error storage 77 StdErrorKind::CapacityExceeded
                    let value_ok %bool eq 77 gui_sfnt_simple_glyph_outline_storage_push_error_scalar_value &error
                    let kind_ok %bool match gui_sfnt_simple_glyph_outline_storage_push_error_kind &error:
                        StdErrorKind::CapacityExceeded:
                            true
                        _:
                            false
                    let recovered_ok %bool gui_sfnt_simple_glyph_outline_storage_push_error_with error @outline_storage_push_error_recovery_callback
                    and value_ok and kind_ok recovered_ok
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_region_push_error_kind_is %fn &GuiSfntSimpleGlyphOutlineRegionPushError fn GuiSfntSimpleGlyphOutlineRegionPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlineRegionPushErrorKind gui_sfnt_simple_glyph_outline_region_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineRegionPushErrorKind::CursorInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlineRegionPushErrorKind::CursorInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineRegionPushErrorKind::CursorRegionMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlineRegionPushErrorKind::CursorRegionMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StorageCursorMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StorageCursorMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineRegionPushErrorKind::RegionFull:
            match expected:
                GuiSfntSimpleGlyphOutlineRegionPushErrorKind::RegionFull:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StoragePushFailed:
            match expected:
                GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StoragePushFailed:
                    true
                _:
                    false

fn outline_region_cursor_span_ok %fn &GuiSfntSimpleGlyphOutlineStorageCapacity fn GuiSfntSimpleGlyphOutlineScalarRegion fn i32 fn i32 bool \capacity\region\expected_start\expected_end:
    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity region:
        Result::Ok cursor:
            let start_ok %bool eq expected_start gui_sfnt_simple_glyph_outline_scalar_region_cursor_start &cursor
            let end_ok %bool eq expected_end gui_sfnt_simple_glyph_outline_scalar_region_cursor_end &cursor
            let next_ok %bool eq expected_start gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &cursor
            and start_ok and end_ok next_ok
        Result::Err _error:
            false

fn outline_region_cursor_boundaries_ok %fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 16
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            let contour_ok %bool outline_region_cursor_span_ok &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint 0 2
            let x_ok %bool outline_region_cursor_span_ok &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX 2 6
            let y_ok %bool outline_region_cursor_span_ok &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY 6 10
            let edge_ok %bool outline_region_cursor_span_ok &capacity GuiSfntSimpleGlyphOutlineScalarRegion::Edge 10 14
            let path_ok %bool outline_region_cursor_span_ok &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PathCommandTag 14 22
            and contour_ok and x_ok and y_ok and edge_ok path_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_region_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 17
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok cursor0:
                            match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage0 cursor0 101:
                                Result::Ok pushed1:
                                    let cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &pushed1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage pushed1
                                    match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage1 cursor1 202:
                                        Result::Ok pushed2:
                                            let cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &pushed2
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage pushed2
                                            let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                            let next_ok %bool eq 2 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &cursor2
                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                            and len_ok next_ok
                                        Result::Err error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                Result::Err error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    false
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage0
                            false
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_region_full_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 18
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok cursor0:
                            match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage0 cursor0 11:
                                Result::Ok pushed1:
                                    let cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &pushed1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage pushed1
                                    match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage1 cursor1 22:
                                        Result::Ok pushed2:
                                            let cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &pushed2
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage pushed2
                                            match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage2 cursor2 33:
                                                Result::Ok pushed3:
                                                    let storage3 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage pushed3
                                                    gui_sfnt_simple_glyph_outline_storage_free storage3
                                                    false
                                                Result::Err error3:
                                                    let kind_ok %bool outline_region_push_error_kind_is &error3 GuiSfntSimpleGlyphOutlineRegionPushErrorKind::RegionFull
                                                    let value_ok %bool eq 33 gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &error3
                                                    let recovered3 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error3
                                                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered3
                                                    gui_sfnt_simple_glyph_outline_storage_free recovered3
                                                    and kind_ok and value_ok len_ok
                                        Result::Err error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                Result::Err error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    false
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage0
                            false
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_region_storage_cursor_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 19
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    let forged_full_cursor %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_scalar_region_cursor GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint 0 2 2
                    match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage forged_full_cursor 44:
                        Result::Ok pushed:
                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage pushed
                            gui_sfnt_simple_glyph_outline_storage_free recovered
                            false
                        Result::Err error:
                            let kind_ok %bool outline_region_push_error_kind_is &error GuiSfntSimpleGlyphOutlineRegionPushErrorKind::StorageCursorMismatch
                            let value_ok %bool eq 44 gui_sfnt_simple_glyph_outline_region_push_error_scalar_value &error
                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error
                            let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                            gui_sfnt_simple_glyph_outline_storage_free recovered
                            and kind_ok and value_ok len_ok
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn main %impure fn void i32 \void:
    let success_ok %bool outline_storage_success_ok
    let invalid_ok %bool outline_storage_invalid_capacity_precedes_limit_ok
    let reject_ok %bool outline_storage_limit_reject_ok
    let overflow_ok %bool outline_storage_scalar_overflow_ok
    let push_ok %bool outline_storage_push_success_ok
    let push_recovery_ok %bool outline_storage_push_error_recovery_ok
    let cursor_ok %bool outline_region_cursor_boundaries_ok
    let region_push_ok %bool outline_region_push_success_ok
    let region_full_ok %bool outline_region_full_ok
    let region_mismatch_ok %bool outline_region_storage_cursor_mismatch_ok
    test_assertion_exit_code assert "outline storage owner contract" and success_ok and invalid_ok and reject_ok and overflow_ok and push_ok and push_recovery_ok and cursor_ok and region_push_ok and region_full_ok region_mismatch_ok
```
