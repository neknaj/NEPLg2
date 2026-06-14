# GUI font SFNT glyf outline contour endpoint doctests

このファイルは、F5e/F5f の contour endpoint population と byte reader bridge が owner-preserving に失敗を分離することを検査する。

## contour endpoint population and reader bridge preserve owner state

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

fn main %impure fn void i32 \void:
    let endpoint_success_ok %bool contour_endpoint_push_success_ok
    let endpoint_non_final_ok %bool contour_endpoint_non_final_last_point_rejected_ok
    let endpoint_final_ok %bool contour_endpoint_final_mismatch_ok
    let endpoint_cursor_ok %bool contour_endpoint_cursor_region_mismatch_ok
    let endpoint_read_ok %bool contour_endpoint_read_push_success_ok
    let endpoint_read_failure_ok %bool contour_endpoint_read_failure_recovers_owner_ok
    let endpoint_read_push_failure_ok %bool contour_endpoint_read_push_failure_preserves_endpoint_ok
    test_assertion_exit_code assert "outline contour endpoint contract" and endpoint_success_ok and endpoint_non_final_ok and endpoint_final_ok and endpoint_cursor_ok and endpoint_read_ok and endpoint_read_failure_ok endpoint_read_push_failure_ok
```
