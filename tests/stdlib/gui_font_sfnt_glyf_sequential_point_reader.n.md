# GUI font SFNT glyf sequential point reader

2 contour の explicit byte fixture を metadata-backed source に解決し、repeat flag、short / signed coordinate、contour endpoint、cursor progression、normal / repeated `End`、typed malformed error を確認する。

## gui_sfnt_glyf_sequential_point_reader

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/gui/font/sfnt/glyf" as *
#import "alloc/gui/font/sfnt/glyf_sequential" as *
#import "alloc/gui/font/sfnt/metadata" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn push_u8 %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push"
        Result::Ok next:
            Result::Ok next

fn push_zeroes %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\count:
    if:
        le count 0
        then:
            Result::Ok builder
        else:
            match push_u8 builder 0:
                Result::Err message:
                    Result::Err message
                Result::Ok next:
                    push_zeroes next sub count 1

fn push_u16_be %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\value:
    match push_u8 builder and shr_u value 8 255:
        Result::Err message:
            Result::Err message
        Result::Ok next:
            push_u8 next and value 255

fn finish_bytes %impure fn Result ByteBuilder str Result ByteBuf str \result:
    match result:
        Result::Err message:
            Result::Err message
        Result::Ok builder:
            match byte_builder_finish builder:
                Result::Err error:
                    byte_builder_error_free error
                    Result::Err "finish"
                Result::Ok bytes:
                    Result::Ok bytes

fn push_fixture_prefix %impure fn ByteBuilder Result ByteBuilder str \builder:
    match push_zeroes builder 54:
        Result::Err message:
            Result::Err message
        Result::Ok b0:
            match push_u16_be b0 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b1:
                    match push_u16_be b1 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b2:
                            match push_u16_be b2 14:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b3:
                                    match push_u16_be b3 2:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b4:
                                            match push_zeroes b4 8:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b5:
                                                    match push_u16_be b5 1:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b6:
                                                            match push_u16_be b6 3:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b7:
                                                                    push_u16_be b7 0

fn push_fixture_point_data %impure fn ByteBuilder Result ByteBuilder str \builder:
    match push_u8 builder 43:
        Result::Err message:
            Result::Err message
        Result::Ok b0:
            match push_u8 b0 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b1:
                    match push_u8 b1 52:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b2:
                            match push_u8 b2 1:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b3:
                                    match push_u8 b3 5:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b4:
                                            match push_u8 b4 3:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b5:
                                                    match push_u8 b5 255:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b6:
                                                            match push_u8 b6 254:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b7:
                                                                    match push_u8 b7 4:
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok b8:
                                                                            match push_u8 b8 255:
                                                                                Result::Err message:
                                                                                    Result::Err message
                                                                                Result::Ok b9:
                                                                                    match push_u8 b9 250:
                                                                                        Result::Err message:
                                                                                            Result::Err message
                                                                                        Result::Ok b10:
                                                                                            push_u8 b10 0

fn build_fixture %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 88:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok builder:
            match push_fixture_prefix builder:
                Result::Err message:
                    Result::Err message
                Result::Ok prefix:
                    finish_bytes push_fixture_point_data prefix

fn sfnt_tag4 %fn i32 fn i32 fn i32 fn i32 i32 \a\b\c\d:
    or or or shl a 24 shl b 16 shl c 8 d

fn fixture_metadata %fn void GuiSfntMetadata \void:
    let head %GuiSfntTableRecord gui_sfnt_table_record sfnt_tag4 'h' 'e' 'a' 'd' 0 54
    let loca %GuiSfntTableRecord gui_sfnt_table_record sfnt_tag4 'l' 'o' 'c' 'a' 54 6
    let glyf %GuiSfntTableRecord gui_sfnt_table_record sfnt_tag4 'g' 'l' 'y' 'f' 60 28
    let directory %GuiSfntDirectory gui_sfnt_directory 65536 3 Option::Some head Option::None Option::None Option::None Option::None Option::None Option::Some loca Option::Some glyf
    let metrics %GuiSfntMetrics gui_sfnt_metrics 1000 0 0 0 2
    gui_sfnt_metadata GuiSfntContainerKind::TrueTypeSfnt 0 1 directory metrics

fn point_matches %fn &GuiSfntSimpleGlyphPoint fn i32 fn i32 fn i32 fn bool fn bool bool \point\index\x\y\on_curve\end_of_contour:
    let actual_on_curve %bool gui_sfnt_simple_glyph_point_on_curve point
    let actual_end %bool gui_sfnt_simple_glyph_point_end_of_contour point
    let on_curve_ok %bool if on_curve actual_on_curve else not actual_on_curve
    let end_ok %bool if end_of_contour actual_end else not actual_end
    and eq gui_sfnt_simple_glyph_point_index point index and eq gui_sfnt_simple_glyph_point_x point x and eq gui_sfnt_simple_glyph_point_y point y and on_curve_ok end_ok

fn cursor_matches %fn &GuiSfntSimpleGlyphSequentialPointCursor fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 fn i32 bool \cursor\logical_index\flag_cursor\repeat_remaining\x_cursor\y_cursor\x\y\contour_index\endpoint:
    and eq gui_sfnt_simple_glyph_sequential_point_cursor_logical_index cursor logical_index and eq gui_sfnt_simple_glyph_sequential_point_cursor_flag_cursor cursor flag_cursor and eq gui_sfnt_simple_glyph_sequential_point_cursor_active_repeat_remaining cursor repeat_remaining and eq gui_sfnt_simple_glyph_sequential_point_cursor_x_cursor cursor x_cursor and eq gui_sfnt_simple_glyph_sequential_point_cursor_y_cursor cursor y_cursor and eq gui_sfnt_simple_glyph_sequential_point_cursor_x cursor x and eq gui_sfnt_simple_glyph_sequential_point_cursor_y cursor y and eq gui_sfnt_simple_glyph_sequential_point_cursor_contour_index cursor contour_index eq gui_sfnt_simple_glyph_sequential_point_cursor_contour_endpoint cursor endpoint

fn read_all_points %fn &ByteBuf fn GuiSfntSimpleGlyphSequentialPointCursor bool \bytes\cursor0:
    match gui_sfnt_simple_glyph_sequential_point_step bytes cursor0:
        Result::Err _error:
            false
        Result::Ok terminal0:
            match terminal0:
                GuiSfntSimpleGlyphSequentialPointTerminal::End _cursor:
                    false
                GuiSfntSimpleGlyphSequentialPointTerminal::Point step0:
                    let point0 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_sequential_point_step_point &step0
                    let cursor1 %GuiSfntSimpleGlyphSequentialPointCursor gui_sfnt_simple_glyph_sequential_point_step_next_cursor &step0
                    if:
                        not and point_matches &point0 0 5 0 true false cursor_matches &cursor1 1 18 1 21 24 5 0 0 1
                        then:
                            false
                        else:
                            match gui_sfnt_simple_glyph_sequential_point_step bytes cursor1:
                                Result::Err _error:
                                    false
                                Result::Ok terminal1:
                                    match terminal1:
                                        GuiSfntSimpleGlyphSequentialPointTerminal::End _cursor:
                                            false
                                        GuiSfntSimpleGlyphSequentialPointTerminal::Point step1:
                                            let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_sequential_point_step_point &step1
                                            let cursor2 %GuiSfntSimpleGlyphSequentialPointCursor gui_sfnt_simple_glyph_sequential_point_step_next_cursor &step1
                                            if:
                                                not and point_matches &point1 1 8 0 true true cursor_matches &cursor2 2 18 0 22 24 8 0 1 3
                                                then:
                                                    false
                                                else:
                                                    match gui_sfnt_simple_glyph_sequential_point_step bytes cursor2:
                                                        Result::Err _error:
                                                            false
                                                        Result::Ok terminal2:
                                                            match terminal2:
                                                                GuiSfntSimpleGlyphSequentialPointTerminal::End _cursor:
                                                                    false
                                                                GuiSfntSimpleGlyphSequentialPointTerminal::Point step2:
                                                                    let point2 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_sequential_point_step_point &step2
                                                                    let cursor3 %GuiSfntSimpleGlyphSequentialPointCursor gui_sfnt_simple_glyph_sequential_point_step_next_cursor &step2
                                                                    if:
                                                                        not and point_matches &point2 2 8 4 false false cursor_matches &cursor3 3 19 0 22 25 8 4 1 3
                                                                        then:
                                                                            false
                                                                        else:
                                                                            match gui_sfnt_simple_glyph_sequential_point_step bytes cursor3:
                                                                                Result::Err _error:
                                                                                    false
                                                                                Result::Ok terminal3:
                                                                                    match terminal3:
                                                                                        GuiSfntSimpleGlyphSequentialPointTerminal::End _cursor:
                                                                                            false
                                                                                        GuiSfntSimpleGlyphSequentialPointTerminal::Point step3:
                                                                                            let point3 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_sequential_point_step_point &step3
                                                                                            let cursor4 %GuiSfntSimpleGlyphSequentialPointCursor gui_sfnt_simple_glyph_sequential_point_step_next_cursor &step3
                                                                                            if:
                                                                                                not and point_matches &point3 3 6 -2 true true cursor_matches &cursor4 4 20 0 24 27 6 -2 1 3
                                                                                                then:
                                                                                                    false
                                                                                                else:
                                                                                                    match gui_sfnt_simple_glyph_sequential_point_step bytes cursor4:
                                                                                                        Result::Err _error:
                                                                                                            false
                                                                                                        Result::Ok terminal4:
                                                                                                            match terminal4:
                                                                                                                GuiSfntSimpleGlyphSequentialPointTerminal::Point _step:
                                                                                                                    false
                                                                                                                GuiSfntSimpleGlyphSequentialPointTerminal::End end_cursor:
                                                                                                                    match gui_sfnt_simple_glyph_sequential_point_step bytes end_cursor:
                                                                                                                        Result::Err _error:
                                                                                                                            false
                                                                                                                        Result::Ok terminal5:
                                                                                                                            match terminal5:
                                                                                                                                GuiSfntSimpleGlyphSequentialPointTerminal::Point _step:
                                                                                                                                    false
                                                                                                                                GuiSfntSimpleGlyphSequentialPointTerminal::End repeated_end_cursor:
                                                                                                                                    cursor_matches &repeated_end_cursor 4 20 0 24 27 6 -2 1 3

fn malformed_is_typed %fn &GuiSfntSimpleGlyphPointStreamSource bool \source:
    let empty %ByteBuf io_bytebuf_empty
    let ok %bool match gui_sfnt_simple_glyph_sequential_point_start &empty *source:
        Result::Ok _cursor:
            false
        Result::Err error:
            let error_source %GuiSfntSimpleGlyphPointStreamSource gui_sfnt_simple_glyph_sequential_point_error_source &error
            let error_cursor %GuiSfntSimpleGlyphSequentialPointCursor gui_sfnt_simple_glyph_sequential_point_error_cursor &error
            let parse_error %GuiSfntParseError gui_sfnt_simple_glyph_sequential_point_error_parse_error &error
            let kind %GuiSfntParseErrorKind gui_sfnt_parse_error_kind &parse_error
            match kind:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    and gui_sfnt_simple_glyph_point_stream_source_eq source &error_source eq gui_sfnt_simple_glyph_sequential_point_cursor_logical_index &error_cursor 0
                _:
                    false
    io_bytebuf_free empty
    ok

fn run_fixture %fn &ByteBuf bool \bytes:
    let metadata %GuiSfntMetadata fixture_metadata
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    match gui_sfnt_simple_glyph_point_stream_source_from_metadata bytes &metadata glyph:
        Result::Err _error:
            false
        Result::Ok source:
            let source_copy %GuiSfntSimpleGlyphPointStreamSource source
            if:
                not gui_sfnt_simple_glyph_point_stream_source_eq &source &source_copy
                then:
                    false
                else:
                    match gui_sfnt_simple_glyph_sequential_point_start bytes source:
                        Result::Err _error:
                            false
                        Result::Ok cursor:
                            and cursor_matches &cursor 0 16 0 20 24 0 0 0 1 and read_all_points bytes cursor malformed_is_typed &source

fn main %impure fn void i32 \void:
    match build_fixture:
        Result::Err _message:
            1
        Result::Ok bytes:
            let ok %bool run_fixture &bytes
            io_bytebuf_free bytes
            if ok 0 1
```
