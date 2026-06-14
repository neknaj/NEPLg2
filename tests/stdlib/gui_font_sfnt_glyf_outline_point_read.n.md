# GUI font SFNT glyf outline point read doctests

このファイルは、F5n の full point read composition が shared precondition を component read より前に検査し、F5k/F5l/F5m の失敗を typed sub-error として保持することを検査する。

## point read composes coordinate endpoint and flag

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

fn make_stream %fn GuiSfntSimpleGlyphTopology fn i32 GuiSfntSimpleGlyphPointStream \topology\flag_length:
    gui_sfnt_simple_glyph_point_stream topology 0 flag_length 1000 0 1000 0 1000 0

fn bool_matches %fn bool fn bool bool \observed\expected:
    match observed:
        true:
            match expected:
                true:
                    true
                false:
                    false
        false:
            match expected:
                true:
                    false
                false:
                    true

fn point_read_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointReadError fn GuiSfntSimpleGlyphOutlinePointReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointReadErrorKind gui_sfnt_simple_glyph_outline_point_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamGlyphMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamContourCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamContourCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamPointCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamPointCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::PointIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::PointIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::CoordinateReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::CoordinateReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::EndpointMarkerReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::EndpointMarkerReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::ComponentGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::ComponentGlyphMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::ComponentPointIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::ComponentPointIndexMismatch:
                    true
                _:
                    false

fn coordinate_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointCoordinateReadError fn GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind gui_sfnt_simple_glyph_outline_point_coordinate_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::CoordinateNotReady:
            match expected:
                GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::CoordinateNotReady:
                    true
                _:
                    false
        _:
            false

fn endpoint_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadError fn GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind gui_sfnt_simple_glyph_outline_point_endpoint_marker_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointTopologyInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointTopologyInvalid:
                    true
                _:
                    false
        _:
            false

fn parse_error_kind_is %fn &GuiSfntParseError fn GuiSfntParseErrorKind bool \error\expected:
    let observed %GuiSfntParseErrorKind gui_sfnt_parse_error_kind error
    match observed:
        GuiSfntParseErrorKind::MalformedGlyfRecord:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    true
                _:
                    false
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

fn push2_region_scalars %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn GuiSfntSimpleGlyphOutlineScalarRegionCursor impure fn i32 impure fn i32 Result GuiSfntSimpleGlyphOutlineStorage str \storage\cursor\a\b:
    match push_region_scalar_or_free storage cursor a:
        Result::Err message:
            Result::Err message
        Result::Ok push_a:
            let cursor_b %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_a
            let storage_b %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_a
            match push_region_scalar_or_free storage_b cursor_b b:
                Result::Err message:
                    Result::Err message
                Result::Ok push_b:
                    Result::Ok gui_sfnt_simple_glyph_outline_region_push_storage push_b

fn push4_region_scalars %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn GuiSfntSimpleGlyphOutlineScalarRegionCursor impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result GuiSfntSimpleGlyphOutlineStorage str \storage\cursor\a\b\c\d:
    match push_region_scalar_or_free storage cursor a:
        Result::Err message:
            Result::Err message
        Result::Ok push_a:
            let cursor_b %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_a
            let storage_b %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_a
            match push_region_scalar_or_free storage_b cursor_b b:
                Result::Err message:
                    Result::Err message
                Result::Ok push_b:
                    let cursor_c %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_b
                    let storage_c %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_b
                    match push_region_scalar_or_free storage_c cursor_c c:
                        Result::Err message:
                            Result::Err message
                        Result::Ok push_c:
                            let cursor_d %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_c
                            let storage_d %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_c
                            match push_region_scalar_or_free storage_d cursor_d d:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok push_d:
                                    Result::Ok gui_sfnt_simple_glyph_outline_region_push_storage push_d

fn prepare_point_x_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit impure fn i32 impure fn i32 Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit\endpoint0\endpoint1:
    match gui_sfnt_simple_glyph_outline_storage_alloc capacity limit:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok storage0:
            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                Result::Err _cursor_error:
                    gui_sfnt_simple_glyph_outline_storage_free storage0
                    Result::Err "endpoint_cursor"
                Result::Ok endpoint_cursor:
                    match push2_region_scalars storage0 endpoint_cursor endpoint0 endpoint1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok storage1:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage1
                                    Result::Err "point_x_cursor"
                                Result::Ok x_cursor:
                                    push4_region_scalars storage1 x_cursor 10 15 15 15

fn prepare_full_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit impure fn i32 impure fn i32 Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit\endpoint0\endpoint1:
    match prepare_point_x_storage capacity limit endpoint0 endpoint1:
        Result::Err message:
            Result::Err message
        Result::Ok storage_x:
            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                Result::Err _cursor_error:
                    gui_sfnt_simple_glyph_outline_storage_free storage_x
                    Result::Err "point_y_cursor"
                Result::Ok y_cursor:
                    push4_region_scalars storage_x y_cursor 20 25 30 35

fn push_u8_or_free %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push_u8"

fn finish_bytes %impure fn Result ByteBuilder str Result ByteBuf str \builder_result:
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

fn bytes4_result %impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result ByteBuf str \a\b\c\d:
    match byte_builder_with_capacity 4:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            finish_bytes:
                match push_u8_or_free b0 a:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match push_u8_or_free b1 b:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                match push_u8_or_free b2 c:
                                    Result::Err message:
                                        Result::Err message
                                    Result::Ok b3:
                                        push_u8_or_free b3 d

fn bytes2_result %impure fn i32 impure fn i32 Result ByteBuf str \a\b:
    match byte_builder_with_capacity 2:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            finish_bytes:
                match push_u8_or_free b0 a:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        push_u8_or_free b1 b

fn point_read_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 90
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 4
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_full_storage &capacity &limit 1 3:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes4_result 1 0 1 0:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match gui_sfnt_simple_glyph_outline_storage_read_point &bytes glyf stream &storage 1:
                                Result::Err _error:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Ok point:
                                    let x_ok %bool eq 15 gui_sfnt_simple_glyph_point_x &point
                                    let y_ok %bool eq 25 gui_sfnt_simple_glyph_point_y &point
                                    let on_curve_ok %bool bool_matches gui_sfnt_simple_glyph_point_on_curve &point false
                                    let end_ok %bool bool_matches gui_sfnt_simple_glyph_point_end_of_contour &point true
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and x_ok and y_ok and on_curve_ok end_ok
        _:
            false

fn point_read_glyph_mismatch_ok %impure fn void bool \void:
    let storage_glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 91
    let stream_glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 92
    let storage_topology %GuiSfntSimpleGlyphTopology make_topology storage_glyph 2 4
    let stream_topology %GuiSfntSimpleGlyphTopology make_topology stream_glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream stream_topology 4
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &storage_topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_full_storage &capacity &limit 1 3:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes4_result 1 0 1 0:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match gui_sfnt_simple_glyph_outline_storage_read_point &bytes glyf stream &storage 0:
                                Result::Ok _point:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadErrorKind::StorageStreamGlyphMismatch
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    kind_ok
        _:
            false

fn point_read_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 93
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 4
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_x_storage &capacity &limit 1 3:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes4_result 1 0 1 0:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match gui_sfnt_simple_glyph_outline_storage_read_point &bytes glyf stream &storage 4:
                                Result::Ok _point:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadErrorKind::PointIndexOutOfRange
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    kind_ok
        _:
            false

fn point_read_coordinate_not_ready_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 94
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 4
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_x_storage &capacity &limit 1 3:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes4_result 1 0 1 0:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match gui_sfnt_simple_glyph_outline_storage_read_point &bytes glyf stream &storage 0:
                                Result::Ok _point:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadErrorKind::CoordinateReadFailed
                                    let sub_ok %bool match gui_sfnt_simple_glyph_outline_point_read_error_coordinate_error &error:
                                        Option::None:
                                            false
                                        Option::Some coordinate_error:
                                            coordinate_error_kind_is &coordinate_error GuiSfntSimpleGlyphOutlinePointCoordinateReadErrorKind::CoordinateNotReady
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and kind_ok sub_ok
        _:
            false

fn point_read_endpoint_topology_invalid_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 95
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 4
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_full_storage &capacity &limit 1 2:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes4_result 1 0 1 0:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match gui_sfnt_simple_glyph_outline_storage_read_point &bytes glyf stream &storage 0:
                                Result::Ok _point:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadErrorKind::EndpointMarkerReadFailed
                                    let sub_ok %bool match gui_sfnt_simple_glyph_outline_point_read_error_endpoint_error &error:
                                        Option::None:
                                            false
                                        Option::Some endpoint_error:
                                            endpoint_error_kind_is &endpoint_error GuiSfntSimpleGlyphOutlinePointEndpointMarkerReadErrorKind::EndpointTopologyInvalid
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and kind_ok sub_ok
        _:
            false

fn point_read_flag_repeat_overrun_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 96
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 2
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 2
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_full_storage &capacity &limit 1 3:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes2_result 9 4:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match gui_sfnt_simple_glyph_outline_storage_read_point &bytes glyf stream &storage 0:
                                Result::Ok _point:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_read_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed
                                    let sub_ok %bool match gui_sfnt_simple_glyph_outline_point_read_error_flag_error &error:
                                        Option::None:
                                            false
                                        Option::Some flag_error:
                                            parse_error_kind_is &flag_error GuiSfntParseErrorKind::MalformedGlyfRecord
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and kind_ok sub_ok
        _:
            false

fn main %impure fn void i32 \void:
    let success_ok %bool point_read_success_ok
    let glyph_mismatch_ok %bool point_read_glyph_mismatch_ok
    let out_of_range_ok %bool point_read_out_of_range_ok
    let coordinate_ok %bool point_read_coordinate_not_ready_ok
    let endpoint_ok %bool point_read_endpoint_topology_invalid_ok
    let flag_ok %bool point_read_flag_repeat_overrun_ok
    test_assertion_exit_code assert "point read composition contract" and success_ok and glyph_mismatch_ok and out_of_range_ok and coordinate_ok and endpoint_ok flag_ok
```
