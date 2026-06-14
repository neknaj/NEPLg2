# GUI font SFNT glyf outline point y doctests

このファイルは、F5i/F5j の PointY storage population と byte reader bridge が、PointY region だけを対象に owner recovery と typed error を返すことを検査する。

## point y storage and reader preserve owner boundaries

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

fn make_point_y_stream %fn GuiSfntSimpleGlyphTopology GuiSfntSimpleGlyphPointStream \topology:
    gui_sfnt_simple_glyph_point_stream topology 0 4 1000 1000 4 4 2000 0

fn make_point_y_bad_stream %fn GuiSfntSimpleGlyphTopology GuiSfntSimpleGlyphPointStream \topology:
    gui_sfnt_simple_glyph_point_stream topology 0 1 1000 1000 1 0 2000 0

fn push_byte %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
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

fn point_y_bytes_result %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 8:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            finish_bytes:
                match push_byte b0 36:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match push_byte b1 36:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                match push_byte b2 36:
                                    Result::Err message:
                                        Result::Err message
                                    Result::Ok b3:
                                        match push_byte b3 36:
                                            Result::Err message:
                                                Result::Err message
                                            Result::Ok b4:
                                                match push_byte b4 20:
                                                    Result::Err message:
                                                        Result::Err message
                                                    Result::Ok b5:
                                                        match push_byte b5 5:
                                                            Result::Err message:
                                                                Result::Err message
                                                            Result::Ok b6:
                                                                match push_byte b6 0:
                                                                    Result::Err message:
                                                                        Result::Err message
                                                                    Result::Ok b7:
                                                                        push_byte b7 0

fn point_y_push_error_kind_is %fn &GuiSfntSimpleGlyphPointYPushError fn GuiSfntSimpleGlyphPointYPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphPointYPushErrorKind gui_sfnt_simple_glyph_point_y_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphPointYPushErrorKind::StorageCapacityInvalid:
            match expected:
                GuiSfntSimpleGlyphPointYPushErrorKind::StorageCapacityInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointYPushErrorKind::CursorInvalid:
            match expected:
                GuiSfntSimpleGlyphPointYPushErrorKind::CursorInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointYPushErrorKind::CursorRegionMismatch:
            match expected:
                GuiSfntSimpleGlyphPointYPushErrorKind::CursorRegionMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointYPushErrorKind::RegionPushFailed:
            match expected:
                GuiSfntSimpleGlyphPointYPushErrorKind::RegionPushFailed:
                    true
                _:
                    false

fn point_y_read_push_error_kind_is %fn &GuiSfntSimpleGlyphPointYReadPushError fn GuiSfntSimpleGlyphPointYReadPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphPointYReadPushErrorKind gui_sfnt_simple_glyph_point_y_read_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphPointYReadPushErrorKind::ReadFailed:
            match expected:
                GuiSfntSimpleGlyphPointYReadPushErrorKind::ReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPointYReadPushErrorKind::PushFailed:
            match expected:
                GuiSfntSimpleGlyphPointYReadPushErrorKind::PushFailed:
                    true
                _:
                    false

fn point_y_push_error_kind_option_is %fn Option GuiSfntSimpleGlyphPointYPushErrorKind fn GuiSfntSimpleGlyphPointYPushErrorKind bool \kind_option\expected:
    match kind_option:
        Option::None:
            false
        Option::Some observed:
            match observed:
                GuiSfntSimpleGlyphPointYPushErrorKind::StorageCapacityInvalid:
                    match expected:
                        GuiSfntSimpleGlyphPointYPushErrorKind::StorageCapacityInvalid:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointYPushErrorKind::CursorInvalid:
                    match expected:
                        GuiSfntSimpleGlyphPointYPushErrorKind::CursorInvalid:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointYPushErrorKind::CursorRegionMismatch:
                    match expected:
                        GuiSfntSimpleGlyphPointYPushErrorKind::CursorRegionMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexMismatch:
                    match expected:
                        GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexMismatch:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexOutOfRange:
                    match expected:
                        GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexOutOfRange:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphPointYPushErrorKind::RegionPushFailed:
                    match expected:
                        GuiSfntSimpleGlyphPointYPushErrorKind::RegionPushFailed:
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

fn prepare_point_y_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit:
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
                                        Result::Err _x_cursor_error:
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

fn point_y_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 33
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_y_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage0
                            false
                        Result::Ok y_cursor0:
                            let point0 %GuiSfntSimpleGlyphPointYSlot gui_sfnt_simple_glyph_point_y_slot 0 20
                            match gui_sfnt_simple_glyph_outline_storage_push_point_y storage0 y_cursor0 point0:
                                Result::Err error1:
                                    let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_error_storage error1
                                    gui_sfnt_simple_glyph_outline_storage_free recovered1
                                    false
                                Result::Ok y_push1:
                                    let y_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_y_push_cursor &y_push1
                                    let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_storage y_push1
                                    let point1 %GuiSfntSimpleGlyphPointYSlot gui_sfnt_simple_glyph_point_y_slot 1 25
                                    match gui_sfnt_simple_glyph_outline_storage_push_point_y storage1 y_cursor1 point1:
                                        Result::Err error2:
                                            let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_error_storage error2
                                            gui_sfnt_simple_glyph_outline_storage_free recovered2
                                            false
                                        Result::Ok y_push2:
                                            let y_cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_y_push_cursor &y_push2
                                            let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_storage y_push2
                                            let len_ok %bool eq 8 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                            let next_ok %bool eq 8 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &y_cursor2
                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                            and len_ok next_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_y_index_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 34
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_y_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok y_cursor:
                            let point %GuiSfntSimpleGlyphPointYSlot gui_sfnt_simple_glyph_point_y_slot 1 25
                            match gui_sfnt_simple_glyph_outline_storage_push_point_y storage y_cursor point:
                                Result::Ok pushed:
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_storage pushed
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_y_push_error_kind_is &error GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexMismatch
                                    let rejected %GuiSfntSimpleGlyphPointYSlot gui_sfnt_simple_glyph_point_y_push_error_point &error
                                    let rejected_ok %bool and eq 1 gui_sfnt_simple_glyph_point_y_slot_point_index &rejected eq 25 gui_sfnt_simple_glyph_point_y_slot_y &rejected
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_error_storage error
                                    let len_ok %bool eq 6 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    and kind_ok and rejected_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_y_wrong_region_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 35
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_point_y_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                        Result::Err _cursor_error:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok x_cursor:
                            let point %GuiSfntSimpleGlyphPointYSlot gui_sfnt_simple_glyph_point_y_slot 0 20
                            match gui_sfnt_simple_glyph_outline_storage_push_point_y storage x_cursor point:
                                Result::Ok pushed:
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_storage pushed
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    false
                                Result::Err error:
                                    let kind_ok %bool point_y_push_error_kind_is &error GuiSfntSimpleGlyphPointYPushErrorKind::CursorRegionMismatch
                                    let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_push_error_storage error
                                    let len_ok %bool eq 6 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                    gui_sfnt_simple_glyph_outline_storage_free recovered
                                    and kind_ok len_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn point_y_read_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 36
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_point_y_stream topology
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match point_y_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match prepare_point_y_storage &capacity &limit:
                        Result::Err _message:
                            io_bytebuf_free bytes
                            false
                        Result::Ok storage0:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage0
                                    io_bytebuf_free bytes
                                    false
                                Result::Ok y_cursor0:
                                    match gui_sfnt_glyf_read_push_point_y &bytes glyf stream storage0 y_cursor0 0:
                                        Result::Err error1:
                                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_error_storage error1
                                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                                            io_bytebuf_free bytes
                                            false
                                        Result::Ok y_push1:
                                            let y_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_y_read_push_cursor &y_push1
                                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_storage y_push1
                                            match gui_sfnt_glyf_read_push_point_y &bytes glyf stream storage1 y_cursor1 1:
                                                Result::Err error2:
                                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_error_storage error2
                                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                                    io_bytebuf_free bytes
                                                    false
                                                Result::Ok y_push2:
                                                    let y_cursor2 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_point_y_read_push_cursor &y_push2
                                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_storage y_push2
                                                    let len_ok %bool eq 8 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                                    let next_ok %bool eq 8 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &y_cursor2
                                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                                    io_bytebuf_free bytes
                                                    and len_ok next_ok
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn point_y_read_failure_recovers_owner_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 37
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_point_y_bad_stream topology
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match point_y_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match prepare_point_y_storage &capacity &limit:
                        Result::Err _message:
                            io_bytebuf_free bytes
                            false
                        Result::Ok storage:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    io_bytebuf_free bytes
                                    false
                                Result::Ok y_cursor:
                                    match gui_sfnt_glyf_read_push_point_y &bytes glyf stream storage y_cursor 0:
                                        Result::Ok pushed:
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_storage pushed
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            false
                                        Result::Err error:
                                            let kind_ok %bool point_y_read_push_error_kind_is &error GuiSfntSimpleGlyphPointYReadPushErrorKind::ReadFailed
                                            let parse_some %bool match gui_sfnt_simple_glyph_point_y_read_push_error_parse_error &error:
                                                Option::Some _parse_error:
                                                    true
                                                Option::None:
                                                    false
                                            let point_none %bool match gui_sfnt_simple_glyph_point_y_read_push_error_point &error:
                                                Option::None:
                                                    true
                                                Option::Some _point:
                                                    false
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_error_storage error
                                            let len_ok %bool eq 6 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            and kind_ok and parse_some and point_none len_ok
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    io_bytebuf_free bytes
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    io_bytebuf_free bytes
                    false

fn point_y_read_push_failure_preserves_point_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 38
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_point_y_stream topology
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match point_y_bytes_result:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
                    match prepare_point_y_storage &capacity &limit:
                        Result::Err _message:
                            io_bytebuf_free bytes
                            false
                        Result::Ok storage:
                            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity &capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                                Result::Err _cursor_error:
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    io_bytebuf_free bytes
                                    false
                                Result::Ok y_cursor:
                                    match gui_sfnt_glyf_read_push_point_y &bytes glyf stream storage y_cursor 1:
                                        Result::Ok pushed:
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_storage pushed
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            false
                                        Result::Err error:
                                            let kind_ok %bool point_y_read_push_error_kind_is &error GuiSfntSimpleGlyphPointYReadPushErrorKind::PushFailed
                                            let parse_none %bool match gui_sfnt_simple_glyph_point_y_read_push_error_parse_error &error:
                                                Option::None:
                                                    true
                                                Option::Some _parse_error:
                                                    false
                                            let point_ok %bool match gui_sfnt_simple_glyph_point_y_read_push_error_point &error:
                                                Option::Some point:
                                                    and eq 1 gui_sfnt_simple_glyph_point_y_slot_point_index &point eq 25 gui_sfnt_simple_glyph_point_y_slot_y &point
                                                Option::None:
                                                    false
                                            let push_kind_ok %bool point_y_push_error_kind_option_is gui_sfnt_simple_glyph_point_y_read_push_error_push_error_kind &error GuiSfntSimpleGlyphPointYPushErrorKind::PointIndexMismatch
                                            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_point_y_read_push_error_storage error
                                            let len_ok %bool eq 6 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &recovered
                                            gui_sfnt_simple_glyph_outline_storage_free recovered
                                            io_bytebuf_free bytes
                                            and kind_ok and parse_none and point_ok and push_kind_ok len_ok
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
    let point_y_success_ok %bool point_y_push_success_ok
    let point_y_mismatch_ok %bool point_y_index_mismatch_ok
    let point_y_region_ok %bool point_y_wrong_region_ok
    let point_y_read_success_ok %bool point_y_read_push_success_ok
    let point_y_read_failure_ok %bool point_y_read_failure_recovers_owner_ok
    let point_y_read_push_failure_ok %bool point_y_read_push_failure_preserves_point_ok
    test_assertion_exit_code assert "point y storage and reader contract" and point_y_success_ok and point_y_mismatch_ok and point_y_region_ok and point_y_read_success_ok and point_y_read_failure_ok point_y_read_push_failure_ok
```
