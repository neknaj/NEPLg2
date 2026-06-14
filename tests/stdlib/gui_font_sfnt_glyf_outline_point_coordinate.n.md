# GUI font SFNT glyf outline point coordinate doctests

このファイルは、F5k の outline storage coordinate read が、既に population 済みの PointX / PointY scalar region だけを読み、full point flag や byte decode へ進まないことを検査する。

## point coordinate read stays read-only

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
#import "core/result" as *
#import "std/test" as *

fn make_bounds %fn GuiGlyphId GuiSfntGlyphBounds \glyph:
    gui_sfnt_glyph_bounds glyph 0 0 10 12

fn make_topology %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphTopology \glyph\contours\points:
    let bounds %GuiSfntGlyphBounds make_bounds glyph
    gui_sfnt_simple_glyph_topology glyph bounds contours points 0 0 0

fn coordinate_read_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointCoordinateReadError fn GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind gui_sfnt_simple_glyph_outline_point_coordinate_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::ScalarSlotCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::ScalarSlotCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::ScalarStorageCapacityMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::ScalarStorageCapacityMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::PointIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::PointIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::CoordinateNotReady:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::CoordinateNotReady:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::ScalarSlotMissing:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::ScalarSlotMissing:
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

fn prepare_point_x_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit:
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
                            match push_region_scalar_or_free storage1 endpoint_cursor1 3:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok endpoint_push1:
                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage endpoint_push1
                                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                        Result::Err _cursor_error:
                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                            Result::Err "point_x_cursor"
                                        Result::Ok x_cursor0:
                                            match push_region_scalar_or_free storage2 x_cursor0 10:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok x_push0:
                                                    let x_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &x_push0
                                                    let storage3 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage x_push0
                                                    match push_region_scalar_or_free storage3 x_cursor1 15:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok x_push1:
                                                            let x_cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &x_push1
                                                            let storage4 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage x_push1
                                                            match push_region_scalar_or_free storage4 x_cursor2 15:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok x_push2:
                                                                    let x_cursor3 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &x_push2
                                                                    let storage5 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage x_push2
                                                                    match push_region_scalar_or_free storage5 x_cursor3 15:
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok x_push3:
                                                                            let storage6 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage x_push3
                                                                            Result::Ok storage6

fn prepare_point_coordinate_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit:
    match prepare_point_x_storage capacity limit:
        Result::Err message:
            Result::Err message
        Result::Ok storage6:
            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                Result::Err _cursor_error:
                    gui_sfnt_simple_glyph_outline_storage_free storage6
                    Result::Err "point_y_cursor"
                Result::Ok y_cursor0:
                    match push_region_scalar_or_free storage6 y_cursor0 20:
                        Result::Err message:
                            Result::Err message
                        Result::Ok y_push0:
                            let y_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &y_push0
                            let storage7 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage y_push0
                            match push_region_scalar_or_free storage7 y_cursor1 25:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok y_push1:
                                    let y_cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &y_push1
                                    let storage8 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage y_push1
                                    match push_region_scalar_or_free storage8 y_cursor2 30:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok y_push2:
                                            let y_cursor3 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &y_push2
                                            let storage9 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage y_push2
                                            match push_region_scalar_or_free storage9 y_cursor3 35:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok y_push3:
                                                    let storage10 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage y_push3
                                                    Result::Ok storage10

fn point_coordinate_read_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 60
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_coordinate_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_storage_read_point_coordinate &storage 0:
                        Result::Err _error:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok point0:
                            match gui_sfnt_simple_glyph_outline_storage_read_point_coordinate &storage 1:
                                Result::Err _error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Ok point1:
                                    let point0_ok %bool and eq 0 gui_sfnt_simple_glyph_outline_point_coordinate_index &point0 and eq 10 gui_sfnt_simple_glyph_outline_point_coordinate_x &point0 eq 20 gui_sfnt_simple_glyph_outline_point_coordinate_y &point0
                                    let point1_ok %bool and eq 1 gui_sfnt_simple_glyph_outline_point_coordinate_index &point1 and eq 15 gui_sfnt_simple_glyph_outline_point_coordinate_x &point1 eq 25 gui_sfnt_simple_glyph_outline_point_coordinate_y &point1
                                    let len_ok %bool eq 10 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and point0_ok and point1_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_coordinate_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 61
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_coordinate_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_storage_read_point_coordinate &storage 4:
                        Result::Ok _point:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Err error:
                            let kind_ok %bool coordinate_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::PointIndexOutOfRange
                            let index_ok %bool eq 4 gui_sfnt_simple_glyph_outline_point_coordinate_read_error_point_index &error
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and kind_ok index_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_coordinate_not_ready_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 62
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_x_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_storage_read_point_coordinate &storage 0:
                        Result::Ok _point:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Err error:
                            let kind_ok %bool coordinate_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::CoordinateNotReady
                            let len_ok %bool eq 6 gui_sfnt_simple_glyph_outline_point_coordinate_read_error_scalar_slots_len &error
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and kind_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn main %impure fn void i32 \void:
    let success_ok %bool point_coordinate_read_success_ok
    let out_of_range_ok %bool point_coordinate_out_of_range_ok
    let not_ready_ok %bool point_coordinate_not_ready_ok
    test_assertion_exit_code assert "point coordinate read contract" and success_ok and out_of_range_ok not_ready_ok
```
