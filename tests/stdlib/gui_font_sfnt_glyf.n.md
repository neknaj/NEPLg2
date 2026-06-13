# GUI font SFNT glyf doctests

このファイルは、SFNT `loca` / `glyf` parser が platform font API ではなく explicit byte fixture だけから glyph header bounds と typed error を返すことを確認する。

## gui_sfnt_glyf_reads_header_bounds_and_typed_errors

short / long `loca` から glyph offset を読み、`glyf` header の bounds を返す。壊れた `head`、unsupported `loca` format、declared length 不足、empty glyph、inverted bounds は enum error として返す。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_sfnt_glyf_reads_header_bounds_and_typed_errors\" count=13 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"glyf glyph1 x min\" expected=\"-10\" actual=\"-10\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"glyf glyph1 y max\" expected=\"200\" actual=\"200\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"glyf long loca x max\" expected=\"90\" actual=\"90\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"missing loca table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"missing glyf table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"short head for glyf\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"unsupported loca format\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"long loca high bit\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"short loca declared length\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"decreasing glyph offset\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"empty glyph outline\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"short glyf header\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"inverted glyph bounds\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/gui/font/sfnt" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn sfnt_tag4 %fn i32 fn i32 fn i32 fn i32 i32 \a\b\c\d:
    or or or shl a 24 shl b 16 shl c 8 d

fn sfnt_push_u8 %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push_u8"

fn sfnt_push_u16_be %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\value:
    match sfnt_push_u8 builder and shr_u value 8 255:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u8 b1 and value 255

fn sfnt_push_u32_be %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\value:
    match sfnt_push_u8 builder and shr_u value 24 255:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 and shr_u value 16 255:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 and shr_u value 8 255:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u8 b3 and value 255

fn sfnt_push_u32_high_bit %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u8 builder add 64 64:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u8 b3 0

fn sfnt_push_zero_run %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\count:
    if:
        le count 0
        then:
            Result::Ok builder
        else:
            match sfnt_push_u8 builder 0:
                Result::Err message:
                    Result::Err message
                Result::Ok next:
                    sfnt_push_zero_run next sub count 1

fn sfnt_push_header %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\table_count:
    match sfnt_push_u32_be builder 65536:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 table_count:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 0

fn sfnt_push_record %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\tag\offset\length:
    match sfnt_push_u32_be builder tag:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u32_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u32_be b2 offset:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u32_be b3 length

fn sfnt_push_glyf_records %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\head_length\loca_length\glyf_length\loca_tag\glyf_tag:
    let head_offset %i32 92
    let hhea_offset %i32 add head_offset head_length
    let maxp_offset %i32 add hhea_offset 10
    let loca_offset %i32 add maxp_offset 6
    let glyf_offset %i32 add loca_offset loca_length
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' head_offset head_length:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' hhea_offset 10:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' maxp_offset 6:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_record b3 loca_tag loca_offset loca_length:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_record b4 glyf_tag glyf_offset glyf_length

fn sfnt_push_head_table %impure fn ByteBuilder impure fn i32 impure fn i32 Result ByteBuilder str \builder\head_length\index_format:
    match sfnt_push_zero_run builder 18:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 2048:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    if:
                        lt head_length 52
                        then:
                            sfnt_push_zero_run b2 sub head_length 20
                        else:
                            match sfnt_push_zero_run b2 30:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b3:
                                    match sfnt_push_u16_be b3 index_format:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b4:
                                            sfnt_push_zero_run b4 sub head_length 52

fn sfnt_push_hhea_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_zero_run builder 4:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1900:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 65036:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u16_be b3 200

fn sfnt_push_maxp_table %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\num_glyphs:
    match sfnt_push_u32_be builder 65536:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u16_be b1 num_glyphs

fn sfnt_push_glyf_metric_tables %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\head_length\index_format\num_glyphs:
    match sfnt_push_head_table builder head_length index_format:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_hhea_table b1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_maxp_table b2 num_glyphs

fn sfnt_push_glyf_sfnt_prefix %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\head_length\index_format\num_glyphs\loca_length\glyf_length\loca_tag\glyf_tag:
    match sfnt_push_header builder 5:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_glyf_records b1 head_length loca_length glyf_length loca_tag glyf_tag:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_glyf_metric_tables b2 head_length index_format num_glyphs

fn sfnt_push_short_loca_valid %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 5:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 10:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 10

fn sfnt_push_short_loca_decreasing %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 5:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 10:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 10

fn sfnt_push_short_loca_truncated %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 5:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u16_be b3 10

fn sfnt_push_short_loca_single %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\short_end:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u16_be b2 short_end

fn sfnt_push_long_loca_valid %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u32_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u32_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u32_be b2 10

fn sfnt_push_long_loca_high_bit %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u32_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u32_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u32_high_bit b2

fn sfnt_push_glyf_header %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\contours\x_min\y_min\x_max\y_max:
    match sfnt_push_u16_be builder contours:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 x_min:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 y_min:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 x_max:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 y_max

fn sfnt_push_two_glyf_headers %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 1 65526 65516 100 200:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_glyf_header b1 1 1 2 3 4

fn sfnt_push_long_glyf_header %impure fn ByteBuilder Result ByteBuilder str \builder:
    sfnt_push_glyf_header builder 1 10 20 90 120

fn sfnt_push_inverted_glyf_header %impure fn ByteBuilder Result ByteBuilder str \builder:
    sfnt_push_glyf_header builder 1 100 20 10 120

fn sfnt_push_short_glyf_header %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 1:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u16_be b1 0

fn sfnt_finish %impure fn Result ByteBuilder str Result ByteBuf str \builder_result:
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

fn build_short_loca_glyf_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 4 10 20 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_valid b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_long_loca_glyf_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 1 2 12 10 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_long_loca_valid b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_long_glyf_header b2

fn build_missing_loca_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 4 10 20 sfnt_tag4 'z' 'z' 'z' 'z' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_valid b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_missing_glyf_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 4 10 20 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'z' 'z' 'z' 'z':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_valid b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_short_head_glyf_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 20 0 4 10 20 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_valid b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_unsupported_loca_format_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 2 4 10 20 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_valid b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_long_loca_high_bit_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 1 2 12 10 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_long_loca_high_bit b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_long_glyf_header b2

fn build_short_loca_declared_length_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 4 8 20 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_truncated b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_decreasing_offset_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 4 10 20 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_decreasing b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_two_glyf_headers b2

fn build_short_glyf_header_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 2 6 4 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_single b1 2:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_short_glyf_header b2

fn build_inverted_bounds_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 2 6 10 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_single b1 5:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_inverted_glyf_header b2

fn sfnt_glyf_error_is %fn Result GuiSfntGlyphBounds GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MissingTable:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingTable:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedLocaFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedLocaFormat:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphOutline:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _bounds:
            false

fn append_success_cases %impure fn TestReport TestReport \report0:
    let glyph1 %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let report1 %TestReport match build_short_loca_glyf_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_glyph_bounds &bytes none glyph1:
                Result::Err _error:
                    test_report_push report0 assert false
                Result::Ok bounds:
                    let r1 %TestReport test_report_push report0 assert_eq_i32 "glyf glyph1 x min" -10 gui_sfnt_glyph_bounds_x_min &bounds
                    test_report_push r1 assert_eq_i32 "glyf glyph1 y max" 200 gui_sfnt_glyph_bounds_y_max &bounds
            io_bytebuf_free bytes
            next_report
    match build_long_loca_glyf_sfnt:
        Result::Err _message:
            test_report_push report1 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_glyph_bounds &bytes none glyph1:
                Result::Err _error:
                    test_report_push report1 assert false
                Result::Ok bounds:
                    test_report_push report1 assert_eq_i32 "glyf long loca x max" 90 gui_sfnt_glyph_bounds_x_max &bounds
            io_bytebuf_free bytes
            next_report

fn append_error_cases %impure fn TestReport TestReport \report0:
    let glyph1 %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let glyph3 %GuiGlyphId unwrap_ok gui_glyph_id_result 3
    let report1 %TestReport match build_missing_loca_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MissingTable
            io_bytebuf_free bytes
            test_report_push report0 assert "missing loca table" ok
    let report2 %TestReport match build_missing_glyf_sfnt:
        Result::Err _message:
            test_report_push report1 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MissingTable
            io_bytebuf_free bytes
            test_report_push report1 assert "missing glyf table" ok
    let report3 %TestReport match build_short_head_glyf_sfnt:
        Result::Err _message:
            test_report_push report2 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report2 assert "short head for glyf" ok
    let report4 %TestReport match build_unsupported_loca_format_sfnt:
        Result::Err _message:
            test_report_push report3 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::UnsupportedLocaFormat
            io_bytebuf_free bytes
            test_report_push report3 assert "unsupported loca format" ok
    let report5 %TestReport match build_long_loca_high_bit_sfnt:
        Result::Err _message:
            test_report_push report4 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report4 assert "long loca high bit" ok
    let report6 %TestReport match build_short_loca_declared_length_sfnt:
        Result::Err _message:
            test_report_push report5 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report5 assert "short loca declared length" ok
    let report7 %TestReport match build_decreasing_offset_sfnt:
        Result::Err _message:
            test_report_push report6 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report6 assert "decreasing glyph offset" ok
    let report8 %TestReport match build_short_loca_glyf_sfnt:
        Result::Err _message:
            test_report_push report7 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph3 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report7 assert "empty glyph outline" ok
    let report9 %TestReport match build_short_glyf_header_sfnt:
        Result::Err _message:
            test_report_push report8 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report8 assert "short glyf header" ok
    match build_inverted_bounds_sfnt:
        Result::Err _message:
            test_report_push report9 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report9 assert "inverted glyph bounds" ok

fn main %impure fn void i32 \void:
    let report0 %TestReport test_report_new "gui_sfnt_glyf_reads_header_bounds_and_typed_errors"
    let report1 %TestReport append_success_cases report0
    let report2 %TestReport append_error_cases report1
    let shown test_report_print_stdout report2
    test_report_exit_code shown
```
