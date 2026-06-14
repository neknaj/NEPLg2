# GUI font SFNT glyf outline point flag doctests

このファイルは、F5m の point flag marker read が checked point stream の flag range だけを読み、coordinate decode や endpoint storage へ進まないことを検査する。

## point flag marker read validates repeat runs before success

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
#import "core/result" as *
#import "std/test" as *

fn make_bounds %fn GuiGlyphId GuiSfntGlyphBounds \glyph:
    gui_sfnt_glyph_bounds glyph 0 0 10 12

fn make_topology %fn GuiGlyphId fn i32 GuiSfntSimpleGlyphTopology \glyph\points:
    let bounds %GuiSfntGlyphBounds make_bounds glyph
    gui_sfnt_simple_glyph_topology glyph bounds 1 points 0 0 0

fn make_stream %fn GuiSfntSimpleGlyphTopology fn i32 GuiSfntSimpleGlyphPointStream \topology\flag_length:
    gui_sfnt_simple_glyph_point_stream topology 0 flag_length 1000 0 1000 0 1000 0

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

fn bytes3_result %impure fn i32 impure fn i32 impure fn i32 Result ByteBuf str \a\b\c:
    match byte_builder_with_capacity 3:
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
                                push_u8_or_free b2 c

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

fn bytes1_result %impure fn i32 Result ByteBuf str \a:
    match byte_builder_with_capacity 1:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            finish_bytes push_u8_or_free b0 a

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

fn parse_error_kind_is %fn &GuiSfntParseError fn GuiSfntParseErrorKind bool \error\expected:
    let observed %GuiSfntParseErrorKind gui_sfnt_parse_error_kind error
    match observed:
        GuiSfntParseErrorKind::MissingGlyphOutline:
            match expected:
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    true
                _:
                    false
        GuiSfntParseErrorKind::MalformedGlyfRecord:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    true
                _:
                    false
        _:
            false

fn marker_matches %fn &ByteBuf fn GuiSfntTableRecord fn GuiSfntSimpleGlyphPointStream fn i32 fn i32 fn bool bool \bytes\glyf\stream\point_index\expected_flag\expected_on_curve:
    match gui_sfnt_glyf_read_point_flag_from_stream bytes glyf stream point_index:
        Result::Err _error:
            false
        Result::Ok marker:
            let flag_ok %bool eq expected_flag gui_sfnt_simple_glyph_point_flag_marker_raw_flag &marker
            let on_curve_ok %bool bool_matches gui_sfnt_simple_glyph_point_flag_marker_on_curve &marker expected_on_curve
            and flag_ok on_curve_ok

fn point_flag_read_no_repeat_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 80
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 3
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 3
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 3
    match bytes3_result 1 0 1:
        Result::Err _message:
            false
        Result::Ok bytes:
            let p0_ok %bool marker_matches &bytes glyf stream 0 1 true
            let p1_ok %bool marker_matches &bytes glyf stream 1 0 false
            let p2_ok %bool marker_matches &bytes glyf stream 2 1 true
            io_bytebuf_free bytes
            and p0_ok and p1_ok p2_ok

fn point_flag_read_repeat_run_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 81
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 3
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 3
    match bytes3_result 9 2 0:
        Result::Err _message:
            false
        Result::Ok bytes:
            let repeated_ok %bool marker_matches &bytes glyf stream 2 9 true
            let final_ok %bool marker_matches &bytes glyf stream 3 0 false
            io_bytebuf_free bytes
            and repeated_ok final_ok

fn point_flag_read_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 82
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 2
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 2
    match bytes2_result 1 0:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_glyf_read_point_flag_from_stream &bytes glyf stream 2:
                Result::Ok _marker:
                    io_bytebuf_free bytes
                    false
                Result::Err error:
                    let kind_ok %bool parse_error_kind_is &error GuiSfntParseErrorKind::MissingGlyphOutline
                    io_bytebuf_free bytes
                    kind_ok

fn point_flag_read_repeat_overrun_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 83
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 2
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 2
    match bytes2_result 9 4:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_glyf_read_point_flag_from_stream &bytes glyf stream 0:
                Result::Ok _marker:
                    io_bytebuf_free bytes
                    false
                Result::Err error:
                    let kind_ok %bool parse_error_kind_is &error GuiSfntParseErrorKind::MalformedGlyfRecord
                    io_bytebuf_free bytes
                    kind_ok

fn point_flag_read_missing_repeat_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 84
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 1
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 1
    match bytes1_result 9:
        Result::Err _message:
            false
        Result::Ok bytes:
            match gui_sfnt_glyf_read_point_flag_from_stream &bytes glyf stream 0:
                Result::Ok _marker:
                    io_bytebuf_free bytes
                    false
                Result::Err error:
                    let kind_ok %bool parse_error_kind_is &error GuiSfntParseErrorKind::MalformedGlyfRecord
                    io_bytebuf_free bytes
                    kind_ok

fn main %impure fn void i32 \void:
    let no_repeat_ok %bool point_flag_read_no_repeat_ok
    let repeat_ok %bool point_flag_read_repeat_run_ok
    let out_of_range_ok %bool point_flag_read_out_of_range_ok
    let repeat_overrun_ok %bool point_flag_read_repeat_overrun_ok
    let missing_repeat_ok %bool point_flag_read_missing_repeat_ok
    test_assertion_exit_code assert "point flag marker read contract" and no_repeat_ok and repeat_ok and out_of_range_ok and repeat_overrun_ok missing_repeat_ok
```
