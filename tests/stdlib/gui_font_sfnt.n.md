# GUI font SFNT parser

このファイルは、SFNT metadata parser が platform font API ではなく explicit byte fixture だけから numeric metrics と typed error を返すことを確認する。

## gui_sfnt_parser_reads_numeric_metrics_and_typed_errors

valid standalone sfnt metrics 用の `head` / `hhea` / `maxp` を持つ最小 SFNT byte 列から metrics を読み、壊れた header、table 欠落、table offset 不正、collection face selection error を enum error として返す。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_sfnt_parser_reads_numeric_metrics_and_typed_errors\" count=15 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"units per em\" expected=\"2048\" actual=\"2048\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"ascent\" expected=\"1900\" actual=\"1900\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"descent\" expected=\"-500\" actual=\"-500\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"line gap\" expected=\"200\" actual=\"200\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"glyph count\" expected=\"321\" actual=\"321\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"face count\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"ttf container\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"truncated header\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"missing maxp\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"invalid table offset\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"high bit table offset\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"ttc face required\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"ttc out of range\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=bool label=\"ttc oversized face count\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"single face rejects one\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "alloc/io" as *
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

fn sfnt_push_valid_records %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 60 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 80 10:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 90 6

fn sfnt_push_missing_maxp_records %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 44 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 64 10

fn sfnt_push_invalid_offset_records %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 200 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 80 10:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 90 6

fn sfnt_push_high_bit_offset_record %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u32_be builder sfnt_tag4 'h' 'e' 'a' 'd':
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u32_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 add 64 64:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 0:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u8 b5 0:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    sfnt_push_u32_be b6 20

fn sfnt_push_valid_tables %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_zero_run builder 18:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 2048:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_zero_run b2 4:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 1900:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u16_be b4 65036:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u16_be b5 200:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u32_be b6 65536:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            sfnt_push_u16_be b7 321

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

fn build_valid_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 96:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 3:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_valid_records b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_valid_tables b2

fn build_missing_maxp_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 74:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 2:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_missing_maxp_records b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                match sfnt_push_zero_run b2 18:
                                    Result::Err message:
                                        Result::Err message
                                    Result::Ok b3:
                                        match sfnt_push_u16_be b3 2048:
                                            Result::Err message:
                                                Result::Err message
                                            Result::Ok b4:
                                                match sfnt_push_zero_run b4 4:
                                                    Result::Err message:
                                                        Result::Err message
                                                    Result::Ok b5:
                                                        match sfnt_push_u16_be b5 1900:
                                                            Result::Err message:
                                                                Result::Err message
                                                            Result::Ok b6:
                                                                match sfnt_push_u16_be b6 65036:
                                                                    Result::Err message:
                                                                        Result::Err message
                                                                    Result::Ok b7:
                                                                        sfnt_push_u16_be b7 200

fn build_invalid_offset_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 60:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 3:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_invalid_offset_records b1

fn build_high_bit_offset_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 28:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 1:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_high_bit_offset_record b1

fn build_ttc_header %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 16:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_u32_be b0 sfnt_tag4 't' 't' 'c' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_u32_be b1 65536:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                match sfnt_push_u32_be b2 1:
                                    Result::Err message:
                                        Result::Err message
                                    Result::Ok b3:
                                        sfnt_push_u32_be b3 16

fn build_oversized_ttc_header %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 12:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_u32_be b0 sfnt_tag4 't' 't' 'c' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_u32_be b1 65536:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_u32_be b2 65535

fn sfnt_error_is_unexpected_eof %fn Result GuiSfntMetadata GuiSfntParseError bool \result:
    match result:
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::UnexpectedEof:
                    true
                _:
                    false
        Result::Ok _metadata:
            false

fn sfnt_error_is_missing_table %fn Result GuiSfntMetadata GuiSfntParseError bool \result:
    match result:
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::MissingTable:
                    true
                _:
                    false
        Result::Ok _metadata:
            false

fn sfnt_error_is_invalid_offset %fn Result GuiSfntMetadata GuiSfntParseError bool \result:
    match result:
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::InvalidTableOffset:
                    true
                _:
                    false
        Result::Ok _metadata:
            false

fn sfnt_error_is_invalid_directory %fn Result GuiSfntMetadata GuiSfntParseError bool \result:
    match result:
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::InvalidTableDirectory:
                    true
                _:
                    false
        Result::Ok _metadata:
            false

fn sfnt_error_is_face_required %fn Result GuiSfntMetadata GuiSfntParseError bool \result:
    match result:
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::FaceIndexRequired:
                    true
                _:
                    false
        Result::Ok _metadata:
            false

fn sfnt_error_is_invalid_face %fn Result GuiSfntMetadata GuiSfntParseError bool \result:
    match result:
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::InvalidFaceIndex:
                    true
                _:
                    false
        Result::Ok _metadata:
            false

fn sfnt_container_is_ttf %fn GuiSfntContainerKind bool \kind:
    match kind:
        GuiSfntContainerKind::TrueTypeSfnt:
            true
        _:
            false

fn parse_valid_values %impure fn void TestReport \void:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push test_report_new "gui_sfnt_parser_reads_numeric_metrics_and_typed_errors" assert false
        Result::Ok bytes:
            let report %TestReport match gui_sfnt_parse_metadata &bytes none:
                Result::Err _error:
                    test_report_push test_report_new "gui_sfnt_parser_reads_numeric_metrics_and_typed_errors" assert false
                Result::Ok metadata:
                    let metrics %GuiSfntMetrics gui_sfnt_metadata_metrics &metadata
                    let container_kind %GuiSfntContainerKind gui_sfnt_metadata_container_kind &metadata
                    let report0 %TestReport test_report_new "gui_sfnt_parser_reads_numeric_metrics_and_typed_errors"
                    let report1 %TestReport test_report_push report0 assert_eq_i32 "units per em" 2048 gui_sfnt_metrics_units_per_em &metrics
                    let report2 %TestReport test_report_push report1 assert_eq_i32 "ascent" 1900 gui_sfnt_metrics_ascent &metrics
                    let report3 %TestReport test_report_push report2 assert_eq_i32 "descent" -500 gui_sfnt_metrics_descent &metrics
                    let report4 %TestReport test_report_push report3 assert_eq_i32 "line gap" 200 gui_sfnt_metrics_line_gap &metrics
                    let report5 %TestReport test_report_push report4 assert_eq_i32 "glyph count" 321 gui_sfnt_metrics_num_glyphs &metrics
                    let report6 %TestReport test_report_push report5 assert_eq_i32 "face count" 1 gui_sfnt_metadata_face_count &metadata
                    test_report_push report6 assert "ttf container" sfnt_container_is_ttf container_kind
            io_bytebuf_free bytes
            report

fn append_error_cases %impure fn TestReport TestReport \report0:
    let empty %ByteBuf io_bytebuf_empty
    let truncated_ok %bool sfnt_error_is_unexpected_eof gui_sfnt_parse_metadata &empty none
    io_bytebuf_free empty
    let report1 %TestReport test_report_push report0 assert "truncated header" truncated_ok
    let report2 %TestReport match build_missing_maxp_sfnt:
        Result::Err _message:
            test_report_push report1 assert false
        Result::Ok bytes:
            let ok %bool sfnt_error_is_missing_table gui_sfnt_parse_metadata &bytes none
            io_bytebuf_free bytes
            test_report_push report1 assert "missing maxp" ok
    let report3 %TestReport match build_invalid_offset_sfnt:
        Result::Err _message:
            test_report_push report2 assert false
        Result::Ok bytes:
            let offset_ok %bool sfnt_error_is_invalid_offset gui_sfnt_parse_metadata &bytes none
            io_bytebuf_free bytes
            test_report_push report2 assert "invalid table offset" offset_ok
    let report4 %TestReport match build_high_bit_offset_sfnt:
        Result::Err _message:
            test_report_push report3 assert false
        Result::Ok bytes:
            let high_offset_ok %bool sfnt_error_is_invalid_offset gui_sfnt_parse_metadata &bytes none
            io_bytebuf_free bytes
            test_report_push report3 assert "high bit table offset" high_offset_ok
    let report5 %TestReport match build_ttc_header:
        Result::Err _message:
            test_report_push report4 assert false
        Result::Ok bytes:
            let required_ok %bool sfnt_error_is_face_required gui_sfnt_parse_metadata &bytes none
            let range_ok %bool sfnt_error_is_invalid_face gui_sfnt_parse_metadata &bytes some 2
            io_bytebuf_free bytes
            let ttc_report %TestReport test_report_push report4 assert "ttc face required" required_ok
            test_report_push ttc_report assert "ttc out of range" range_ok
    let report6 %TestReport match build_oversized_ttc_header:
        Result::Err _message:
            test_report_push report5 assert false
        Result::Ok bytes:
            let huge_ok %bool sfnt_error_is_invalid_directory gui_sfnt_parse_metadata &bytes some 0
            io_bytebuf_free bytes
            test_report_push report5 assert "ttc oversized face count" huge_ok
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push report6 assert false
        Result::Ok bytes:
            let one_ok %bool sfnt_error_is_invalid_face gui_sfnt_parse_metadata &bytes some 1
            io_bytebuf_free bytes
            test_report_push report6 assert "single face rejects one" one_ok

fn main %impure fn void i32 \void:
    let report0 %TestReport parse_valid_values
    let report1 %TestReport append_error_cases report0
    let shown test_report_print_stdout report1
    test_report_exit_code shown
```
