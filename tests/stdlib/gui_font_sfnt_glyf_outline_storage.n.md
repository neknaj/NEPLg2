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

#import "alloc/io" as *
#import "alloc/gui/font/sfnt/glyf" as *
#import "alloc/gui/font/sfnt/metadata" as *
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

fn make_topology_with_point_data_offset %fn GuiGlyphId fn i32 fn i32 fn i32 GuiSfntSimpleGlyphTopology \glyph\contours\points\point_data_offset:
    let bounds %GuiSfntGlyphBounds make_bounds glyph
    gui_sfnt_simple_glyph_topology glyph bounds contours points 0 point_data_offset 0

fn make_point_x_stream %fn GuiSfntSimpleGlyphTopology GuiSfntSimpleGlyphPointStream \topology:
    gui_sfnt_simple_glyph_point_stream topology 0 4 4 4 1000 1000 2000 0

fn make_point_x_bad_stream %fn GuiSfntSimpleGlyphTopology GuiSfntSimpleGlyphPointStream \topology:
    gui_sfnt_simple_glyph_point_stream topology 0 1 1 0 1000 1000 2000 0

fn outline_endpoint_push_u8 %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push_u8"

fn outline_endpoint_push_u16_be %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\value:
    match outline_endpoint_push_u8 builder and shr_u value 8 255:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            outline_endpoint_push_u8 b1 and value 255

fn outline_endpoint_push_zero_run %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\count:
    if:
        le count 0
        then:
            Result::Ok builder
        else:
            match outline_endpoint_push_u8 builder 0:
                Result::Err message:
                    Result::Err message
                Result::Ok next:
                    outline_endpoint_push_zero_run next sub count 1

fn outline_endpoint_finish %impure fn Result ByteBuilder str Result ByteBuf str \builder_result:
    match builder_result:
        Result::Err message:
            Result::Err message
        Result::Ok builder:
            match byte_builder_finish builder:
                Result::Err error:
                    byte_builder_error_free error
                    Result::Err "finish"
                Result::Ok bytes:
                    Result::Ok bytes

fn outline_endpoint_bytes_result %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 14:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            outline_endpoint_finish:
                match outline_endpoint_push_zero_run b0 10:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match outline_endpoint_push_u16_be b1 1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                outline_endpoint_push_u16_be b2 3

fn outline_point_x_bytes_result %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 8:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            outline_endpoint_finish:
                match outline_endpoint_push_u8 b0 50:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match outline_endpoint_push_u8 b1 50:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                match outline_endpoint_push_u8 b2 50:
                                    Result::Err message:
                                        Result::Err message
                                    Result::Ok b3:
                                        match outline_endpoint_push_u8 b3 50:
                                            Result::Err message:
                                                Result::Err message
                                            Result::Ok b4:
                                                match outline_endpoint_push_u8 b4 10:
                                                    Result::Err message:
                                                        Result::Err message
                                                    Result::Ok b5:
                                                        match outline_endpoint_push_u8 b5 5:
                                                            Result::Err message:
                                                                Result::Err message
                                                            Result::Ok b6:
                                                                match outline_endpoint_push_u8 b6 0:
                                                                    Result::Err message:
                                                                        Result::Err message
                                                                    Result::Ok b7:
                                                                        outline_endpoint_push_u8 b7 0

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

fn contour_endpoint_push_error_kind_is %fn &GuiSfntSimpleGlyphContourEndpointPushError fn GuiSfntSimpleGlyphContourEndpointPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphContourEndpointPushErrorKind gui_sfnt_simple_glyph_contour_endpoint_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorInvalid:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorRegionMismatch:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorRegionMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::ContourIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::ContourIndexMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::PreviousEndpointMismatch:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::PreviousEndpointMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointOutOfRange:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointNotIncreasing:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointNotIncreasing:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::FinalEndpointMismatch:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::FinalEndpointMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointPushErrorKind::RegionPushFailed:
            match expected:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::RegionPushFailed:
                    true
                _:
                    false

fn contour_endpoint_read_push_error_kind_is %fn &GuiSfntSimpleGlyphContourEndpointReadPushError fn GuiSfntSimpleGlyphContourEndpointReadPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphContourEndpointReadPushErrorKind gui_sfnt_simple_glyph_contour_endpoint_read_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphContourEndpointReadPushErrorKind::ReadFailed:
            match expected:
                GuiSfntSimpleGlyphContourEndpointReadPushErrorKind::ReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphContourEndpointReadPushErrorKind::PushFailed:
            match expected:
                GuiSfntSimpleGlyphContourEndpointReadPushErrorKind::PushFailed:
                    true
                _:
                    false

fn point_x_push_error_kind_is %fn &GuiSfntSimpleGlyphPointXPushError fn GuiSfntSimpleGlyphPointXPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphPointXPushErrorKind gui_sfnt_simple_glyph_point_x_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphPointXPushErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphPointXPushErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointXPushErrorKind::CursorInvalid:
            match expected:
                GuiSfntSimpleGlyphPointXPushErrorKind::CursorInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointXPushErrorKind::CursorRegionMismatch:
            match expected:
                GuiSfntSimpleGlyphPointXPushErrorKind::CursorRegionMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointXPushErrorKind::RegionPushFailed:
            match expected:
                GuiSfntSimpleGlyphPointXPushErrorKind::RegionPushFailed:
                    true
                _:
                    false

fn point_x_read_push_error_kind_is %fn &GuiSfntSimpleGlyphPointXReadPushError fn GuiSfntSimpleGlyphPointXReadPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphPointXReadPushErrorKind gui_sfnt_simple_glyph_point_x_read_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphPointXReadPushErrorKind::ReadFailed:
            match expected:
                GuiSfntSimpleGlyphPointXReadPushErrorKind::ReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointXReadPushErrorKind::PushFailed:
            match expected:
                GuiSfntSimpleGlyphPointXReadPushErrorKind::PushFailed:
                    true
                _:
                    false

fn point_x_push_error_kind_option_is %fn Option GuiSfntSimpleGlyphPointXPushErrorKind fn GuiSfntSimpleGlyphPointXPushErrorKind bool \kind_option\expected:
    match kind_option:
        Option::None:
            false
        Option::Some observed:
            match observed:
                GuiSfntSimpleGlyphPointXPushErrorKind::StorageCapacityInvalid:
                    match expected:
                        GuiSfntSimpleGlyphPointXPushErrorKind::StorageCapacityInvalid:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointXPushErrorKind::CursorInvalid:
                    match expected:
                        GuiSfntSimpleGlyphPointXPushErrorKind::CursorInvalid:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointXPushErrorKind::CursorRegionMismatch:
                    match expected:
                        GuiSfntSimpleGlyphPointXPushErrorKind::CursorRegionMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexMismatch:
                    match expected:
                        GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexOutOfRange:
                    match expected:
                        GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexOutOfRange:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointXPushErrorKind::RegionPushFailed:
                    match expected:
                        GuiSfntSimpleGlyphPointXPushErrorKind::RegionPushFailed:
                            true
                        _:
                            false

fn contour_endpoint_push_error_kind_option_is %fn Option GuiSfntSimpleGlyphContourEndpointPushErrorKind fn GuiSfntSimpleGlyphContourEndpointPushErrorKind bool \kind_option\expected:
    match kind_option:
        Option::None:
            false
        Option::Some observed:
            match observed:
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::StorageCapacityInvalid:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::StorageCapacityInvalid:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorInvalid:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorInvalid:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorRegionMismatch:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorRegionMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::ContourIndexMismatch:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::ContourIndexMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::PreviousEndpointMismatch:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::PreviousEndpointMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointOutOfRange:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointOutOfRange:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointNotIncreasing:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointNotIncreasing:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::FinalEndpointMismatch:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::FinalEndpointMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphContourEndpointPushErrorKind::RegionPushFailed:
                    match expected:
                        GuiSfntSimpleGlyphContourEndpointPushErrorKind::RegionPushFailed:
                            true
                        _:
                            false

fn contour_endpoint_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 20
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok cursor0:
                            let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 cursor0 endpoint0 none_previous:
                                Result::Ok pushed1:
                                    let cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &pushed1
                                    let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &pushed1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed1
                                    let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                    let previous_option %Option i32 some previous1
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 cursor1 endpoint1 previous_option:
                                        Result::Ok pushed2:
                                            let cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &pushed2
                                            let previous2 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &pushed2
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed2
                                            let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                            let next_ok %bool eq 2 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &cursor2
                                            let previous_ok %bool eq 3 previous2
                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                            and len_ok and next_ok previous_ok
                                        Result::Err error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                Result::Err error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error1
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

fn contour_endpoint_non_final_last_point_rejected_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 21
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok cursor:
                            let endpoint %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 3
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint none_previous:
                                Result::Ok pushed:
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    false
                                Result::Err error:
                                    let kind_ok %bool contour_endpoint_push_error_kind_is &error GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointOutOfRange
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error
                                    let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    and kind_ok len_ok
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn contour_endpoint_final_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 22
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok cursor0:
                            let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 cursor0 endpoint0 none_previous:
                                Result::Ok pushed1:
                                    let cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &pushed1
                                    let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &pushed1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed1
                                    let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 2
                                    let previous_option %Option i32 some previous1
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 cursor1 endpoint1 previous_option:
                                        Result::Ok pushed2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                        Result::Err error2:
                                            let kind_ok %bool contour_endpoint_push_error_kind_is &error2 GuiSfntSimpleGlyphContourEndpointPushErrorKind::FinalEndpointMismatch
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error2
                                            let len_ok %bool eq 1 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            and kind_ok len_ok
                                Result::Err error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error1
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

fn contour_endpoint_cursor_region_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 23
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                        Result::Ok cursor:
                            let endpoint %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage cursor endpoint none_previous:
                                Result::Ok pushed:
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage pushed
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    false
                                Result::Err error:
                                    let kind_ok %bool contour_endpoint_push_error_kind_is &error GuiSfntSimpleGlyphContourEndpointPushErrorKind::CursorRegionMismatch
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error
                                    let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    and kind_ok len_ok
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn contour_endpoint_read_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 24
    let topology %GuiSfntSimpleGlyphTopology make_topology_with_point_data_offset glyph 2 4 16
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 14
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match outline_endpoint_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                        Result::Ok storage0:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                                Result::Ok cursor0:
                                    let none_previous %Option i32 none
                                    match gui_sfnt_glyf_read_push_contour_endpoint &bytes glyf topology storage0 cursor0 0 none_previous:
                                        Result::Ok pushed1:
                                            let cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_read_push_cursor &pushed1
                                            let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_read_push_previous_endpoint &pushed1
                                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_storage pushed1
                                            let previous_option %Option i32 some previous1
                                            match gui_sfnt_glyf_read_push_contour_endpoint &bytes glyf topology storage1 cursor1 1 previous_option:
                                                Result::Ok pushed2:
                                                    let cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_read_push_cursor &pushed2
                                                    let previous2 %i32 gui_sfnt_simple_glyph_contour_endpoint_read_push_previous_endpoint &pushed2
                                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_storage pushed2
                                                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                                    let next_ok %bool eq 2 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &cursor2
                                                    let previous_ok %bool eq 3 previous2
                                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                                    io_bytebuf_free bytes
                                                    and len_ok and next_ok previous_ok
                                                Result::Err error2:
                                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_error_storage error2
                                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                    io_bytebuf_free bytes
                                                    false
                                        Result::Err error1:
                                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_error_storage error1
                                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                                            io_bytebuf_free bytes
                                            false
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage0
                                    io_bytebuf_free bytes
                                    false
                        Result::Err _error:
                            io_bytebuf_free bytes
                            false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn contour_endpoint_read_failure_recovers_owner_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 25
    let topology %GuiSfntSimpleGlyphTopology make_topology_with_point_data_offset glyph 2 4 16
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 11
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match outline_endpoint_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                        Result::Ok storage:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                                Result::Ok cursor:
                                    let none_previous %Option i32 none
                                    match gui_sfnt_glyf_read_push_contour_endpoint &bytes glyf topology storage cursor 0 none_previous:
                                        Result::Ok pushed:
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_storage pushed
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            false
                                        Result::Err error:
                                            let kind_ok %bool contour_endpoint_read_push_error_kind_is &error GuiSfntSimpleGlyphContourEndpointReadPushErrorKind::ReadFailed
                                            let parse_ok %bool match gui_sfnt_simple_glyph_contour_endpoint_read_push_error_parse_error &error:
                                                Option::Some parse_error:
                                                    match gui_sfnt_parse_error_kind &parse_error:
                                                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                                                            true
                                                        _:
                                                            false
                                                Option::None:
                                                    false
                                            let endpoint_none %bool match gui_sfnt_simple_glyph_contour_endpoint_read_push_error_endpoint &error:
                                                Option::None:
                                                    true
                                                Option::Some _endpoint:
                                                    false
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_error_storage error
                                            let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            and kind_ok and parse_ok and endpoint_none len_ok
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    io_bytebuf_free bytes
                                    false
                        Result::Err _error:
                            io_bytebuf_free bytes
                            false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn contour_endpoint_read_push_failure_preserves_endpoint_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 26
    let topology %GuiSfntSimpleGlyphTopology make_topology_with_point_data_offset glyph 2 4 16
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 14
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match outline_endpoint_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                        Result::Ok storage:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                                Result::Ok cursor:
                                    let none_previous %Option i32 none
                                    match gui_sfnt_glyf_read_push_contour_endpoint &bytes glyf topology storage cursor 1 none_previous:
                                        Result::Ok pushed:
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_storage pushed
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            false
                                        Result::Err error:
                                            let kind_ok %bool contour_endpoint_read_push_error_kind_is &error GuiSfntSimpleGlyphContourEndpointReadPushErrorKind::PushFailed
                                            let parse_none %bool match gui_sfnt_simple_glyph_contour_endpoint_read_push_error_parse_error &error:
                                                Option::None:
                                                    true
                                                Option::Some _parse_error:
                                                    false
                                            let endpoint_ok %bool match gui_sfnt_simple_glyph_contour_endpoint_read_push_error_endpoint &error:
                                                Option::Some endpoint:
                                                    and eq 1 gui_sfnt_simple_glyph_contour_endpoint_slot_contour_index &endpoint eq 3 gui_sfnt_simple_glyph_contour_endpoint_slot_end_point_index &endpoint
                                                Option::None:
                                                    false
                                            let push_kind_ok %bool contour_endpoint_push_error_kind_option_is gui_sfnt_simple_glyph_contour_endpoint_read_push_error_push_error_kind &error GuiSfntSimpleGlyphContourEndpointPushErrorKind::ContourIndexMismatch
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_read_push_error_storage error
                                            let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            and kind_ok and parse_none and endpoint_ok and push_kind_ok len_ok
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    io_bytebuf_free bytes
                                    false
                        Result::Err _error:
                            io_bytebuf_free bytes
                            false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn point_x_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 27
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok endpoint_cursor0:
                            let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                                Result::Ok endpoint_push1:
                                    let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push1
                                    let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                    let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                    let previous_option %Option i32 some previous1
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                        Result::Ok endpoint_push2:
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push2
                                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                                Result::Ok x_cursor0:
                                                    let point0 %GuiSfntSimpleGlyphPointXSlot gui_sfnt_simple_glyph_point_x_slot 0 10
                                                    match gui_sfnt_simple_glyph_outline_storage_push_point_x storage2 x_cursor0 point0:
                                                        Result::Ok x_push1:
                                                            let x_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_x_push_cursor &x_push1
                                                            let storage3 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_storage x_push1
                                                            let point1 %GuiSfntSimpleGlyphPointXSlot gui_sfnt_simple_glyph_point_x_slot 1 sub 0 5
                                                            match gui_sfnt_simple_glyph_outline_storage_push_point_x storage3 x_cursor1 point1:
                                                                Result::Ok x_push2:
                                                                    let x_cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_x_push_cursor &x_push2
                                                                    let storage4 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_storage x_push2
                                                                    let len_ok %bool eq 4 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage4
                                                                    let next_ok %bool eq 4 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &x_cursor2
                                                                    gui_sfnt_simple_glyph_outline_storage_free storage4
                                                                    and len_ok next_ok
                                                                Result::Err error2:
                                                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_error_storage error2
                                                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                                    false
                                                        Result::Err error1:
                                                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_error_storage error1
                                                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                                                            false
                                                Result::Err _x_cursor_error:
                                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                                    false
                                        Result::Err endpoint_error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                Result::Err endpoint_error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    false
                        Result::Err _endpoint_cursor_error:
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

fn point_x_index_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 28
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok endpoint_cursor0:
                            let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                                Result::Ok endpoint_push1:
                                    let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push1
                                    let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                    let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                    let previous_option %Option i32 some previous1
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                        Result::Ok endpoint_push2:
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push2
                                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                                Result::Ok x_cursor:
                                                    let point %GuiSfntSimpleGlyphPointXSlot gui_sfnt_simple_glyph_point_x_slot 1 10
                                                    match gui_sfnt_simple_glyph_outline_storage_push_point_x storage2 x_cursor point:
                                                        Result::Ok pushed:
                                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_storage pushed
                                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                                            false
                                                        Result::Err error:
                                                            let kind_ok %bool point_x_push_error_kind_is &error GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexMismatch
                                                            let rejected %GuiSfntSimpleGlyphPointXSlot gui_sfnt_simple_glyph_point_x_push_error_point &error
                                                            let rejected_ok %bool and eq 1 gui_sfnt_simple_glyph_point_x_slot_point_index &rejected eq 10 gui_sfnt_simple_glyph_point_x_slot_x &rejected
                                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_error_storage error
                                                            let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                                            and kind_ok and rejected_ok len_ok
                                                Result::Err _x_cursor_error:
                                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                                    false
                                        Result::Err endpoint_error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                Result::Err endpoint_error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    false
                        Result::Err _endpoint_cursor_error:
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

fn point_x_wrong_region_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 29
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                        Result::Ok endpoint_cursor0:
                            let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                            let none_previous %Option i32 none
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                                Result::Ok endpoint_push1:
                                    let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push1
                                    let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                    let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                    let previous_option %Option i32 some previous1
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                        Result::Ok endpoint_push2:
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push2
                                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                                                Result::Ok y_cursor:
                                                    let point %GuiSfntSimpleGlyphPointXSlot gui_sfnt_simple_glyph_point_x_slot 0 10
                                                    match gui_sfnt_simple_glyph_outline_storage_push_point_x storage2 y_cursor point:
                                                        Result::Ok pushed:
                                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_storage pushed
                                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                                            false
                                                        Result::Err error:
                                                            let kind_ok %bool point_x_push_error_kind_is &error GuiSfntSimpleGlyphPointXPushErrorKind::CursorRegionMismatch
                                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_push_error_storage error
                                                            let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                                            and kind_ok len_ok
                                                Result::Err _y_cursor_error:
                                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                                    false
                                        Result::Err endpoint_error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                Result::Err endpoint_error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    false
                        Result::Err _endpoint_cursor_error:
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

fn point_x_read_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 30
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_point_x_stream topology
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match outline_point_x_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                        Result::Ok storage0:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                                Result::Ok endpoint_cursor0:
                                    let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                                    let none_previous %Option i32 none
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                                        Result::Ok endpoint_push1:
                                            let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push1
                                            let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push1
                                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                            let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                            let previous_option %Option i32 some previous1
                                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                                Result::Ok endpoint_push2:
                                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push2
                                                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                                        Result::Ok x_cursor0:
                                                            match gui_sfnt_glyf_read_push_point_x &bytes glyf stream storage2 x_cursor0 0:
                                                                Result::Ok x_push1:
                                                                    let x_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_x_read_push_cursor &x_push1
                                                                    let storage3 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_storage x_push1
                                                                    match gui_sfnt_glyf_read_push_point_x &bytes glyf stream storage3 x_cursor1 1:
                                                                        Result::Ok x_push2:
                                                                            let x_cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_x_read_push_cursor &x_push2
                                                                            let storage4 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_storage x_push2
                                                                            let len_ok %bool eq 4 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage4
                                                                            let next_ok %bool eq 4 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &x_cursor2
                                                                            gui_sfnt_simple_glyph_outline_storage_free storage4
                                                                            io_bytebuf_free bytes
                                                                            and len_ok next_ok
                                                                        Result::Err error2:
                                                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_error_storage error2
                                                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                                            io_bytebuf_free bytes
                                                                            false
                                                                Result::Err error1:
                                                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_error_storage error1
                                                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                                                    io_bytebuf_free bytes
                                                                    false
                                                        Result::Err _x_cursor_error:
                                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                                            io_bytebuf_free bytes
                                                            false
                                                Result::Err endpoint_error2:
                                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error2
                                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                    io_bytebuf_free bytes
                                                    false
                                        Result::Err endpoint_error1:
                                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error1
                                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                                            io_bytebuf_free bytes
                                            false
                                Result::Err _endpoint_cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage0
                                    io_bytebuf_free bytes
                                    false
                        Result::Err _error:
                            io_bytebuf_free bytes
                            false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn point_x_read_failure_recovers_owner_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 31
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_point_x_bad_stream topology
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match outline_point_x_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                        Result::Ok storage0:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                                Result::Ok endpoint_cursor0:
                                    let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                                    let none_previous %Option i32 none
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                                        Result::Ok endpoint_push1:
                                            let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push1
                                            let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push1
                                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                            let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                            let previous_option %Option i32 some previous1
                                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                                Result::Ok endpoint_push2:
                                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push2
                                                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                                        Result::Ok x_cursor:
                                                            match gui_sfnt_glyf_read_push_point_x &bytes glyf stream storage2 x_cursor 0:
                                                                Result::Ok pushed:
                                                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_storage pushed
                                                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                                                    io_bytebuf_free bytes
                                                                    false
                                                                Result::Err error:
                                                                    let kind_ok %bool point_x_read_push_error_kind_is &error GuiSfntSimpleGlyphPointXReadPushErrorKind::ReadFailed
                                                                    let parse_ok %bool match gui_sfnt_simple_glyph_point_x_read_push_error_parse_error &error:
                                                                        Option::Some parse_error:
                                                                            match gui_sfnt_parse_error_kind &parse_error:
                                                                                GuiSfntParseErrorKind::MalformedGlyfRecord:
                                                                                    true
                                                                                _:
                                                                                    false
                                                                        Option::None:
                                                                            false
                                                                    let point_none %bool match gui_sfnt_simple_glyph_point_x_read_push_error_point &error:
                                                                        Option::None:
                                                                            true
                                                                        Option::Some _point:
                                                                            false
                                                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_error_storage error
                                                                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                                                    io_bytebuf_free bytes
                                                                    and kind_ok and parse_ok and point_none len_ok
                                                        Result::Err _x_cursor_error:
                                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                                            io_bytebuf_free bytes
                                                            false
                                                Result::Err endpoint_error2:
                                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error2
                                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                    io_bytebuf_free bytes
                                                    false
                                        Result::Err endpoint_error1:
                                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error1
                                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                                            io_bytebuf_free bytes
                                            false
                                Result::Err _endpoint_cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage0
                                    io_bytebuf_free bytes
                                    false
                        Result::Err _error:
                            io_bytebuf_free bytes
                            false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn point_x_read_push_failure_preserves_point_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 32
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_point_x_stream topology
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match outline_point_x_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                        Result::Ok storage0:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                                Result::Ok endpoint_cursor0:
                                    let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                                    let none_previous %Option i32 none
                                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                                        Result::Ok endpoint_push1:
                                            let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push1
                                            let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push1
                                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                            let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                                            let previous_option %Option i32 some previous1
                                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                                Result::Ok endpoint_push2:
                                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push2
                                                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                                        Result::Ok x_cursor:
                                                            match gui_sfnt_glyf_read_push_point_x &bytes glyf stream storage2 x_cursor 1:
                                                                Result::Ok pushed:
                                                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_storage pushed
                                                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                                                    io_bytebuf_free bytes
                                                                    false
                                                                Result::Err error:
                                                                    let kind_ok %bool point_x_read_push_error_kind_is &error GuiSfntSimpleGlyphPointXReadPushErrorKind::PushFailed
                                                                    let parse_none %bool match gui_sfnt_simple_glyph_point_x_read_push_error_parse_error &error:
                                                                        Option::None:
                                                                            true
                                                                        Option::Some _parse_error:
                                                                            false
                                                                    let point_ok %bool match gui_sfnt_simple_glyph_point_x_read_push_error_point &error:
                                                                        Option::Some point:
                                                                            and eq 1 gui_sfnt_simple_glyph_point_x_slot_point_index &point eq 15 gui_sfnt_simple_glyph_point_x_slot_x &point
                                                                        Option::None:
                                                                            false
                                                                    let push_kind_ok %bool point_x_push_error_kind_option_is gui_sfnt_simple_glyph_point_x_read_push_error_push_error_kind &error GuiSfntSimpleGlyphPointXPushErrorKind::PointIndexMismatch
                                                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_x_read_push_error_storage error
                                                                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                                                    io_bytebuf_free bytes
                                                                    and kind_ok and parse_none and point_ok and push_kind_ok len_ok
                                                        Result::Err _x_cursor_error:
                                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                                            io_bytebuf_free bytes
                                                            false
                                                Result::Err endpoint_error2:
                                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error2
                                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                    io_bytebuf_free bytes
                                                    false
                                        Result::Err endpoint_error1:
                                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage endpoint_error1
                                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                                            io_bytebuf_free bytes
                                            false
                                Result::Err _endpoint_cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage0
                                    io_bytebuf_free bytes
                                    false
                        Result::Err _error:
                            io_bytebuf_free bytes
                            false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
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
    let endpoint_success_ok %bool contour_endpoint_push_success_ok
    let endpoint_non_final_ok %bool contour_endpoint_non_final_last_point_rejected_ok
    let endpoint_final_ok %bool contour_endpoint_final_mismatch_ok
    let endpoint_cursor_ok %bool contour_endpoint_cursor_region_mismatch_ok
    let endpoint_read_ok %bool contour_endpoint_read_push_success_ok
    let endpoint_read_failure_ok %bool contour_endpoint_read_failure_recovers_owner_ok
    let endpoint_read_push_failure_ok %bool contour_endpoint_read_push_failure_preserves_endpoint_ok
    let point_x_success_ok %bool point_x_push_success_ok
    let point_x_mismatch_ok %bool point_x_index_mismatch_ok
    let point_x_region_ok %bool point_x_wrong_region_ok
    let point_x_read_success_ok %bool point_x_read_push_success_ok
    let point_x_read_failure_ok %bool point_x_read_failure_recovers_owner_ok
    let point_x_read_push_failure_ok %bool point_x_read_push_failure_preserves_point_ok
    test_assertion_exit_code assert "outline storage owner contract" and success_ok and invalid_ok and reject_ok and overflow_ok and push_ok and push_recovery_ok and cursor_ok and region_push_ok and region_full_ok and region_mismatch_ok and endpoint_success_ok and endpoint_non_final_ok and endpoint_final_ok and endpoint_cursor_ok and endpoint_read_ok and endpoint_read_failure_ok and endpoint_read_push_failure_ok and point_x_success_ok and point_x_mismatch_ok and point_x_region_ok and point_x_read_success_ok and point_x_read_failure_ok point_x_read_push_failure_ok
```
