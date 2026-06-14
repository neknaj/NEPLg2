# GUI font SFNT glyf outline point step doctests

このファイルは、F5o の full point read step が shared precondition を終端判定より前に検査し、point と正常終端を typed status と `Option` で分けることを検査する。

## point read step distinguishes point and terminal end

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

fn point_step_status_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn GuiSfntSimpleGlyphOutlinePointReadStepStatus bool \step\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointReadStepStatus gui_sfnt_simple_glyph_outline_point_read_step_status step
    match observed:
        GuiSfntSimpleGlyphOutlinePointReadStepStatus::Point:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepStatus::Point:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadStepStatus::End:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepStatus::End:
                    true
                _:
                    false

fn point_step_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointReadStepError fn GuiSfntSimpleGlyphOutlinePointReadStepErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointReadStepErrorKind gui_sfnt_simple_glyph_outline_point_read_step_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageStreamGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageStreamGlyphMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageStreamContourCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageStreamContourCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageStreamPointCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::StorageStreamPointCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::CursorOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::CursorOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::PointReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::PointReadFailed:
                    true
                _:
                    false

fn point_read_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointReadError fn GuiSfntSimpleGlyphOutlinePointReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointReadErrorKind gui_sfnt_simple_glyph_outline_point_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed:
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

fn point_step_first_point_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 100
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
                            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
                            match gui_sfnt_simple_glyph_outline_storage_read_point_step &bytes glyf stream &storage cursor:
                                Result::Err _error:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Ok step:
                                    let status_ok %bool point_step_status_is &step GuiSfntSimpleGlyphOutlinePointReadStepStatus::Point
                                    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_step_next_cursor &step
                                    let next_ok %bool eq 1 gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &next_cursor
                                    let point_ok %bool match gui_sfnt_simple_glyph_outline_point_read_step_point &step:
                                        Option::None:
                                            false
                                        Option::Some point:
                                            let index_ok %bool eq 0 gui_sfnt_simple_glyph_point_index &point
                                            let x_ok %bool eq 10 gui_sfnt_simple_glyph_point_x &point
                                            let y_ok %bool eq 20 gui_sfnt_simple_glyph_point_y &point
                                            let curve_ok %bool bool_matches gui_sfnt_simple_glyph_point_on_curve &point true
                                            and index_ok and x_ok and y_ok curve_ok
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and status_ok and next_ok point_ok
        _:
            false

fn point_step_last_point_advances_to_end_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 101
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
                            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 3
                            match gui_sfnt_simple_glyph_outline_storage_read_point_step &bytes glyf stream &storage cursor:
                                Result::Err _error:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Ok step:
                                    let status_ok %bool point_step_status_is &step GuiSfntSimpleGlyphOutlinePointReadStepStatus::Point
                                    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_step_next_cursor &step
                                    let next_ok %bool eq 4 gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &next_cursor
                                    let point_ok %bool match gui_sfnt_simple_glyph_outline_point_read_step_point &step:
                                        Option::None:
                                            false
                                        Option::Some point:
                                            let index_ok %bool eq 3 gui_sfnt_simple_glyph_point_index &point
                                            let y_ok %bool eq 35 gui_sfnt_simple_glyph_point_y &point
                                            let end_ok %bool bool_matches gui_sfnt_simple_glyph_point_end_of_contour &point true
                                            and index_ok and y_ok end_ok
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and status_ok and next_ok point_ok
        _:
            false

fn point_step_terminal_end_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 102
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
                            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 4
                            match gui_sfnt_simple_glyph_outline_storage_read_point_step &bytes glyf stream &storage cursor:
                                Result::Err _error:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Ok step:
                                    let status_ok %bool point_step_status_is &step GuiSfntSimpleGlyphOutlinePointReadStepStatus::End
                                    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_step_next_cursor &step
                                    let next_ok %bool eq 4 gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &next_cursor
                                    let point_ok %bool match gui_sfnt_simple_glyph_outline_point_read_step_point &step:
                                        Option::None:
                                            true
                                        Option::Some point:
                                            let _index %i32 gui_sfnt_simple_glyph_point_index &point
                                            false
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and status_ok and next_ok point_ok
        _:
            false

fn point_step_cursor_too_far_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 103
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
                            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 5
                            match gui_sfnt_simple_glyph_outline_storage_read_point_step &bytes glyf stream &storage cursor:
                                Result::Ok _step:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_step_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::CursorOutOfRange
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    kind_ok
        _:
            false

fn point_step_wraps_point_read_failure_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 104
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
                            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
                            match gui_sfnt_simple_glyph_outline_storage_read_point_step &bytes glyf stream &storage cursor:
                                Result::Ok _step:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_step_error_kind_is &error GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::PointReadFailed
                                    let sub_ok %bool match gui_sfnt_simple_glyph_outline_point_read_step_error_point_error &error:
                                        Option::None:
                                            false
                                        Option::Some point_error:
                                            point_read_error_kind_is &point_error GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    and kind_ok sub_ok
        _:
            false

fn main %impure fn void i32 \void:
    let first_ok %bool point_step_first_point_ok
    let last_ok %bool point_step_last_point_advances_to_end_ok
    let terminal_ok %bool point_step_terminal_end_ok
    let too_far_ok %bool point_step_cursor_too_far_ok
    let wrap_ok %bool point_step_wraps_point_read_failure_ok
    test_assertion_exit_code assert "point read step contract" and first_ok and last_ok and terminal_ok and too_far_ok wrap_ok
```
