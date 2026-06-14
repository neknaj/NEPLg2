# GUI font SFNT glyf outline point x reader success doctests

このファイルは、F5h の PointX byte reader bridge が x-only read success を storage mutation へ接続できることを検査する。

## point x reader bridge commits valid x coordinates

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

fn make_point_x_stream %fn GuiSfntSimpleGlyphTopology GuiSfntSimpleGlyphPointStream \topology:
    gui_sfnt_simple_glyph_point_stream topology 0 4 4 4 1000 1000 2000 0

fn outline_endpoint_push_u8 %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push_u8"

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

fn main %impure fn void i32 \void:
    let point_x_read_success_ok %bool point_x_read_push_success_ok
    test_assertion_exit_code assert "outline point x reader success contract" point_x_read_success_ok
```
