# GUI font SFNT glyf outline point x population doctests

このファイルは、F5g の PointX population が scalar storage index と glyph logical point index を混同せず、owner-preserving に失敗を返すことを検査する。

## point x population preserves owner state

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

fn main %impure fn void i32 \void:
    let point_x_success_ok %bool point_x_push_success_ok
    let point_x_mismatch_ok %bool point_x_index_mismatch_ok
    let point_x_region_ok %bool point_x_wrong_region_ok
    test_assertion_exit_code assert "outline point x population contract" and point_x_success_ok and point_x_mismatch_ok point_x_region_ok
```
