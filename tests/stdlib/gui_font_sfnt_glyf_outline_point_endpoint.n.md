# GUI font SFNT glyf outline point endpoint doctests

このファイルは、F5l の endpoint marker read が ContourEndpoint region 全体を検査し、final endpoint mismatch を partial success にしないことを検査する。

## point endpoint marker read validates full endpoint topology

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

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

fn endpoint_read_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError fn GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind gui_sfnt_simple_glyph_outline_point_endpoint_marker_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::ScalarSlotCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::ScalarSlotCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::ScalarStorageCapacityMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::ScalarStorageCapacityMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::PointIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::PointIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointNotReady:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointNotReady:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointSlotMissing:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointSlotMissing:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointTopologyInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointTopologyInvalid:
                    true
                _:
                    false

fn push_region_scalar_or_free %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn GuiSfntSimpleGlyphOutlineScalarRegionCursor impure fn i32 Result GuiSfntSimpleGlyphOutlineRegionPush str \storage\cursor\value:
    match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage cursor value:
        Result::Ok pushed:
            Result::Ok pushed
        Result::Err error:
            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error
            gui_sfnt_simple_glyph_outline_storage_free recovered
            Result::Err "push_region_scalar"

fn prepare_endpoint_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit:
    match gui_sfnt_simple_glyph_outline_storage_alloc capacity limit:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok storage0:
            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                Result::Err _cursor_error:
                    gui_sfnt_simple_glyph_outline_storage_free storage0
                    Result::Err "endpoint_cursor"
                Result::Ok endpoint_cursor0:
                    let endpoint0 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 0 1
                    let none_previous %Option i32 none
                    match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage0 endpoint_cursor0 endpoint0 none_previous:
                        Result::Err error0:
                            let recovered0 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error0
                            gui_sfnt_simple_glyph_outline_storage_free recovered0
                            Result::Err "endpoint0"
                        Result::Ok endpoint_push0:
                            let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_contour_endpoint_push_cursor &endpoint_push0
                            let previous1 %i32 gui_sfnt_simple_glyph_contour_endpoint_push_previous_endpoint &endpoint_push0
                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push0
                            let endpoint1 %GuiSfntSimpleGlyphContourEndpointSlot gui_sfnt_simple_glyph_contour_endpoint_slot 1 3
                            let previous_option %Option i32 some previous1
                            match gui_sfnt_simple_glyph_outline_storage_push_contour_endpoint storage1 endpoint_cursor1 endpoint1 previous_option:
                                Result::Err error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_error_storage error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    Result::Err "endpoint1"
                                Result::Ok endpoint_push1:
                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_contour_endpoint_push_storage endpoint_push1
                                    Result::Ok storage2

fn prepare_forged_endpoint_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit:
    match gui_sfnt_simple_glyph_outline_storage_alloc capacity limit:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok storage0:
            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                Result::Err _cursor_error:
                    gui_sfnt_simple_glyph_outline_storage_free storage0
                    Result::Err "endpoint_cursor"
                Result::Ok endpoint_cursor0:
                    match push_region_scalar_or_free storage0 endpoint_cursor0 1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok endpoint_push0:
                            let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &endpoint_push0
                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage endpoint_push0
                            match push_region_scalar_or_free storage1 endpoint_cursor1 2:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok endpoint_push1:
                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage endpoint_push1
                                    Result::Ok storage2

fn marker_matches %fn &GuiSfntSimpleGlyphOutlineStorage fn i32 fn i32 fn bool bool \storage\point_index\expected_contour\expected_end:
    match gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker storage point_index:
        Result::Err _error:
            false
        Result::Ok marker:
            let contour_ok %bool eq expected_contour gui_sfnt_simple_glyph_outline_point_endpoint_marker_contour_index &marker
            let observed_end %bool gui_sfnt_simple_glyph_outline_point_endpoint_marker_end_of_contour &marker
            let end_ok %bool match observed_end:
                true:
                    match expected_end:
                        true:
                            true
                        false:
                            false
                false:
                    match expected_end:
                        true:
                            false
                        false:
                            true
            and contour_ok end_ok

fn point_endpoint_marker_read_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 70
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_endpoint_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    let p0_ok %bool marker_matches &storage 0 0 false
                    let p1_ok %bool marker_matches &storage 1 0 true
                    let p2_ok %bool marker_matches &storage 2 1 false
                    let p3_ok %bool marker_matches &storage 3 1 true
                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage
                    gui_sfnt_simple_glyph_outline_storage_free storage
                    and p0_ok and p1_ok and p2_ok and p3_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_endpoint_marker_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 71
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_endpoint_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker &storage 4:
                        Result::Ok _marker:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Err error:
                            let kind_ok %bool endpoint_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::PointIndexOutOfRange
                            let index_ok %bool eq 4 gui_sfnt_simple_glyph_outline_point_endpoint_marker_read_error_point_index &error
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and kind_ok index_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_endpoint_marker_not_ready_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 72
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Err _error:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker &storage 0:
                        Result::Ok _marker:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Err error:
                            let kind_ok %bool endpoint_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointNotReady
                            let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_point_endpoint_marker_read_error_scalar_slots_len &error
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and kind_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_endpoint_marker_topology_invalid_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 73
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_forged_endpoint_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_storage_read_point_endpoint_marker &storage 0:
                        Result::Ok _marker:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Err error:
                            let kind_ok %bool endpoint_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointTopologyInvalid
                            let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and kind_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn main %impure fn void i32 \void:
    let success_ok %bool point_endpoint_marker_read_success_ok
    let out_of_range_ok %bool point_endpoint_marker_out_of_range_ok
    let not_ready_ok %bool point_endpoint_marker_not_ready_ok
    let topology_invalid_ok %bool point_endpoint_marker_topology_invalid_ok
    test_assertion_exit_code assert "point endpoint marker read contract" and success_ok and out_of_range_ok and not_ready_ok topology_invalid_ok
```
