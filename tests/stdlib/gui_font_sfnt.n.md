# GUI font SFNT parser

このファイルは、SFNT metadata parser が platform font API ではなく explicit byte fixture だけから numeric metrics と typed error を返すことを確認する。

## gui_sfnt_parser_reads_numeric_metrics_and_typed_errors

valid standalone sfnt metrics 用の `head` / `hhea` / `maxp` を持つ最小 SFNT byte 列から metrics を読み、壊れた header、table 欠落、table offset 不正、collection face selection error を enum error として返す。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_sfnt_parser_reads_numeric_metrics_and_typed_errors\" count=37 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"units per em\" expected=\"2048\" actual=\"2048\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"ascent\" expected=\"1900\" actual=\"1900\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"descent\" expected=\"-500\" actual=\"-500\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"line gap\" expected=\"200\" actual=\"200\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"glyph count\" expected=\"321\" actual=\"321\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"face count\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"ttf container\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"truncated header\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"missing maxp\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"invalid table offset\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"high bit table offset\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"ttc face required\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"ttc out of range\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=bool label=\"ttc oversized face count\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"single face rejects one\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=15 status=ok kind=bool label=\"family name\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=16 status=ok kind=bool label=\"subfamily name\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=17 status=ok kind=bool label=\"full name\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=18 status=ok kind=bool label=\"missing name table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=19 status=ok kind=bool label=\"unsupported name encoding\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=20 status=ok kind=bool label=\"odd utf16 name length\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=21 status=ok kind=bool label=\"non ascii name character\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=22 status=ok kind=bool label=\"name string offset overlaps records\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=23 status=ok kind=eq_i32 label=\"cmap glyph A\" expected=\"36\" actual=\"36\" message=\"\"\nassertion index=24 status=ok kind=bool label=\"cmap missing glyph\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=25 status=ok kind=bool label=\"cmap outside bmp\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=26 status=ok kind=bool label=\"missing cmap table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=27 status=ok kind=bool label=\"unsupported cmap encoding\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=28 status=ok kind=bool label=\"unsupported selected cmap format\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=29 status=ok kind=bool label=\"cmap glyph zero rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=30 status=ok kind=bool label=\"malformed cmap segment count\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=31 status=ok kind=eq_i32 label=\"cmap glyph array A\" expected=\"36\" actual=\"36\" message=\"\"\nassertion index=32 status=ok kind=bool label=\"cmap glyph array zero rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=33 status=ok kind=bool label=\"cmap glyph array range malformed\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=34 status=ok kind=bool label=\"cmap subtable overlaps records\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=35 status=ok kind=bool label=\"short cmap header\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=36 status=ok kind=bool label=\"short cmap subtable\" expected=\"true\" actual=\"true\" message=\"\"\n"
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

fn sfnt_push_named_records %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 76 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 96 10:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 106 6:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_record b3 sfnt_tag4 'n' 'a' 'm' 'e' 112 88

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

fn sfnt_push_name_record %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\platform_id\encoding_id\language_id\name_id\length\offset:
    match sfnt_push_u16_be builder platform_id:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 encoding_id:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 language_id:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 name_id:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u16_be b4 length:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            sfnt_push_u16_be b5 offset

fn sfnt_push_utf16_demo %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 'D':
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 'e':
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 'm':
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u16_be b3 'o'

fn sfnt_push_utf16_regular %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 'R':
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 'e':
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 'g':
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 'u':
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u16_be b4 'l':
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u16_be b5 'a':
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    sfnt_push_u16_be b6 'r'

fn sfnt_push_utf16_demo_regular %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_utf16_demo builder:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 ' ':
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_utf16_regular b2

fn sfnt_push_windows_name_strings %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_utf16_demo builder:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_utf16_regular b1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_utf16_demo_regular b2

fn sfnt_push_windows_name_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 3:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 42:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_name_record b3 3 1 1033 1 8 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_name_record b4 3 1 1033 2 14 8:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_name_record b5 3 1 1033 4 24 22:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    sfnt_push_windows_name_strings b6

fn sfnt_push_unsupported_name_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 18:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_name_record b3 3 2 1033 1 8 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_utf16_demo b4:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            sfnt_push_zero_run b5 62

fn sfnt_push_odd_utf16_name_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 18:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_name_record b3 3 1 1033 1 7 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_utf16_demo b4:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            sfnt_push_zero_run b5 62

fn sfnt_push_non_ascii_name_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 18:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_name_record b3 3 1 1033 1 2 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u16_be b4 256:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            sfnt_push_zero_run b5 68

fn sfnt_push_overlapping_string_offset_name_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 6:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_name_record b3 3 1 1033 1 8 0:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_zero_run b4 70

fn sfnt_push_cmap_records %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\cmap_length:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 76 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 96 10:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 106 6:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_record b3 sfnt_tag4 'c' 'm' 'a' 'p' 112 cmap_length

fn sfnt_push_cmap_record %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\platform_id\encoding_id\offset:
    match sfnt_push_u16_be builder platform_id:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 encoding_id:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u32_be b2 offset

fn sfnt_push_format4_a_segment %impure fn ByteBuilder impure fn i32 impure fn i32 Result ByteBuilder str \builder\seg_count_x2\delta_raw:
    match sfnt_push_u16_be builder 4:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 32:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 seg_count_x2:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_zero_run b4 6:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u16_be b5 'A':
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u16_be b6 65535:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_u16_be b7 0:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    match sfnt_push_u16_be b8 'A':
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok b9:
                                                                            match sfnt_push_u16_be b9 65535:
                                                                                Result::Err message:
                                                                                    Result::Err message
                                                                                Result::Ok b10:
                                                                                    match sfnt_push_u16_be b10 delta_raw:
                                                                                        Result::Err message:
                                                                                            Result::Err message
                                                                                        Result::Ok b11:
                                                                                            match sfnt_push_u16_be b11 1:
                                                                                                Result::Err message:
                                                                                                    Result::Err message
                                                                                                Result::Ok b12:
                                                                                                    match sfnt_push_u16_be b12 0:
                                                                                                        Result::Err message:
                                                                                                            Result::Err message
                                                                                                        Result::Ok b13:
                                                                                                            sfnt_push_u16_be b13 0

fn sfnt_push_valid_cmap_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_format4_a_segment b3 4 65507

fn sfnt_push_zero_glyph_cmap_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_format4_a_segment b3 4 65471

fn sfnt_push_unsupported_cmap_encoding_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 0 3 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_format4_a_segment b3 4 65507

fn sfnt_push_format0_subtable %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 6:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u16_be b2 0

fn sfnt_push_unsupported_selected_format_cmap_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 2:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 20:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_cmap_record b3 3 10 26:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_format0_subtable b4:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            sfnt_push_format4_a_segment b5 4 65507

fn sfnt_push_malformed_cmap_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_format4_a_segment b3 3 65507

fn sfnt_push_format4_a_glyph_array_segment %impure fn ByteBuilder impure fn i32 impure fn i32 Result ByteBuilder str \builder\glyph_raw\range_offset:
    match sfnt_push_u16_be builder 4:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 34:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 4:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_zero_run b4 6:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u16_be b5 'A':
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u16_be b6 65535:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_u16_be b7 0:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    match sfnt_push_u16_be b8 'A':
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok b9:
                                                                            match sfnt_push_u16_be b9 65535:
                                                                                Result::Err message:
                                                                                    Result::Err message
                                                                                Result::Ok b10:
                                                                                    match sfnt_push_u16_be b10 0:
                                                                                        Result::Err message:
                                                                                            Result::Err message
                                                                                        Result::Ok b11:
                                                                                            match sfnt_push_u16_be b11 1:
                                                                                                Result::Err message:
                                                                                                    Result::Err message
                                                                                                Result::Ok b12:
                                                                                                    match sfnt_push_u16_be b12 range_offset:
                                                                                                        Result::Err message:
                                                                                                            Result::Err message
                                                                                                        Result::Ok b13:
                                                                                                            match sfnt_push_u16_be b13 0:
                                                                                                                Result::Err message:
                                                                                                                    Result::Err message
                                                                                                                Result::Ok b14:
                                                                                                                    sfnt_push_u16_be b14 glyph_raw

fn sfnt_push_glyph_array_cmap_table %impure fn ByteBuilder impure fn i32 impure fn i32 Result ByteBuilder str \builder\glyph_raw\range_offset:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_format4_a_glyph_array_segment b3 glyph_raw range_offset

fn sfnt_push_overlapping_cmap_subtable_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 8:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_format4_a_segment b3 4 65507

fn sfnt_push_short_cmap_header_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    sfnt_push_u16_be builder 0

fn sfnt_push_short_cmap_subtable_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_cmap_record b2 3 1 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_u16_be b3 4

fn sfnt_push_named_sfnt_prefix %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_header builder 4:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_named_records b1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_valid_tables b2

fn sfnt_push_cmap_sfnt_prefix %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\cmap_length:
    match sfnt_push_header builder 4:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_cmap_records b1 cmap_length:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_valid_tables b2

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

fn build_named_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 200:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_named_sfnt_prefix b0:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_windows_name_table b1

fn build_unsupported_name_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 150:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_named_sfnt_prefix b0:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_unsupported_name_table b1

fn build_odd_utf16_name_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 150:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_named_sfnt_prefix b0:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_odd_utf16_name_table b1

fn build_non_ascii_name_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 150:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_named_sfnt_prefix b0:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_non_ascii_name_table b1

fn build_overlapping_string_offset_name_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 150:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_named_sfnt_prefix b0:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_overlapping_string_offset_name_table b1

fn build_valid_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 156:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 44:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_valid_cmap_table b1

fn build_zero_glyph_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 156:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 44:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_zero_glyph_cmap_table b1

fn build_unsupported_cmap_encoding_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 156:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 44:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_unsupported_cmap_encoding_table b1

fn build_unsupported_selected_format_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 170:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 58:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_unsupported_selected_format_cmap_table b1

fn build_malformed_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 156:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 44:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_malformed_cmap_table b1

fn build_glyph_array_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 158:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 46:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_glyph_array_cmap_table b1 36 4

fn build_zero_glyph_array_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 158:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 46:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_glyph_array_cmap_table b1 0 4

fn build_out_of_range_glyph_array_cmap_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 158:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 46:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_glyph_array_cmap_table b1 36 8

fn build_overlapping_cmap_subtable_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 156:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 44:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_overlapping_cmap_subtable_table b1

fn build_short_cmap_header_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 114:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 2:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_short_cmap_header_table b1

fn build_short_cmap_subtable_sfnt %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 126:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_cmap_sfnt_prefix b0 14:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        sfnt_push_short_cmap_subtable_table b1

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

fn sfnt_name_error_is %fn Result GuiSfntNames GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MissingTable:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingTable:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedNameEncoding:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedNameEncoding:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MalformedNameRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedNameRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedNameCharacter:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedNameCharacter:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _names:
            false

fn sfnt_cmap_error_is %fn Result GuiGlyphId GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MissingTable:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingTable:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedCmapEncoding:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedCmapEncoding:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedCmapTableFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedCmapTableFormat:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MalformedCmapRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedCmapRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphMapping:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphMapping:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _glyph:
            false

fn sfnt_option_str_eq %fn Option str fn str bool \value\expected:
    match value:
        Option::None:
            false
        Option::Some actual:
            test_str_eq actual expected

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

fn append_name_cases %impure fn TestReport TestReport \report0:
    let report1 %TestReport match build_named_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_parse_names &bytes none:
                Result::Err _error:
                    test_report_push report0 assert false
                Result::Ok names:
                    let family_ok %bool sfnt_option_str_eq gui_sfnt_names_family &names "Demo"
                    let subfamily_ok %bool sfnt_option_str_eq gui_sfnt_names_subfamily &names "Regular"
                    let full_name_ok %bool sfnt_option_str_eq gui_sfnt_names_full_name &names "Demo Regular"
                    let r1 %TestReport test_report_push report0 assert "family name" family_ok
                    let r2 %TestReport test_report_push r1 assert "subfamily name" subfamily_ok
                    test_report_push r2 assert "full name" full_name_ok
            io_bytebuf_free bytes
            next_report
    let report2 %TestReport match build_valid_sfnt:
        Result::Err _message:
            test_report_push report1 assert false
        Result::Ok bytes:
            let missing_ok %bool sfnt_name_error_is gui_sfnt_parse_names &bytes none GuiSfntParseErrorKind::MissingTable
            io_bytebuf_free bytes
            test_report_push report1 assert "missing name table" missing_ok
    let report3 %TestReport match build_unsupported_name_sfnt:
        Result::Err _message:
            test_report_push report2 assert false
        Result::Ok bytes:
            let unsupported_ok %bool sfnt_name_error_is gui_sfnt_parse_names &bytes none GuiSfntParseErrorKind::UnsupportedNameEncoding
            io_bytebuf_free bytes
            test_report_push report2 assert "unsupported name encoding" unsupported_ok
    let report4 %TestReport match build_odd_utf16_name_sfnt:
        Result::Err _message:
            test_report_push report3 assert false
        Result::Ok bytes:
            let malformed_ok %bool sfnt_name_error_is gui_sfnt_parse_names &bytes none GuiSfntParseErrorKind::MalformedNameRecord
            io_bytebuf_free bytes
            test_report_push report3 assert "odd utf16 name length" malformed_ok
    match build_non_ascii_name_sfnt:
        Result::Err _message:
            test_report_push report4 assert false
        Result::Ok bytes:
            let character_ok %bool sfnt_name_error_is gui_sfnt_parse_names &bytes none GuiSfntParseErrorKind::UnsupportedNameCharacter
            io_bytebuf_free bytes
            let report5 %TestReport test_report_push report4 assert "non ascii name character" character_ok
            match build_overlapping_string_offset_name_sfnt:
                Result::Err _message:
                    test_report_push report5 assert false
                Result::Ok bytes2:
                    let offset_ok %bool sfnt_name_error_is gui_sfnt_parse_names &bytes2 none GuiSfntParseErrorKind::MalformedNameRecord
                    io_bytebuf_free bytes2
                    test_report_push report5 assert "name string offset overlaps records" offset_ok

fn append_cmap_cases %impure fn TestReport TestReport \report0:
    let report1 %TestReport match build_valid_cmap_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_glyph_id &bytes none 'A':
                Result::Err _error:
                    test_report_push report0 assert false
                Result::Ok glyph:
                    test_report_push report0 assert_eq_i32 "cmap glyph A" 36 gui_glyph_id_raw &glyph
            let missing_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'B' GuiSfntParseErrorKind::MissingGlyphMapping
            let outside_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 65536 GuiSfntParseErrorKind::UnsupportedCmapEncoding
            io_bytebuf_free bytes
            let r1 %TestReport test_report_push next_report assert "cmap missing glyph" missing_ok
            test_report_push r1 assert "cmap outside bmp" outside_ok
    let report2 %TestReport match build_valid_sfnt:
        Result::Err _message:
            test_report_push report1 assert false
        Result::Ok bytes:
            let missing_table_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MissingTable
            io_bytebuf_free bytes
            test_report_push report1 assert "missing cmap table" missing_table_ok
    let report3 %TestReport match build_unsupported_cmap_encoding_sfnt:
        Result::Err _message:
            test_report_push report2 assert false
        Result::Ok bytes:
            let unsupported_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::UnsupportedCmapEncoding
            io_bytebuf_free bytes
            test_report_push report2 assert "unsupported cmap encoding" unsupported_ok
    let report4 %TestReport match build_unsupported_selected_format_cmap_sfnt:
        Result::Err _message:
            test_report_push report3 assert false
        Result::Ok bytes:
            let format_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::UnsupportedCmapTableFormat
            io_bytebuf_free bytes
            test_report_push report3 assert "unsupported selected cmap format" format_ok
    let report5 %TestReport match build_zero_glyph_cmap_sfnt:
        Result::Err _message:
            test_report_push report4 assert false
        Result::Ok bytes:
            let zero_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MissingGlyphMapping
            io_bytebuf_free bytes
            test_report_push report4 assert "cmap glyph zero rejected" zero_ok
    let report6 %TestReport match build_malformed_cmap_sfnt:
        Result::Err _message:
            test_report_push report5 assert false
        Result::Ok bytes:
            let malformed_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MalformedCmapRecord
            io_bytebuf_free bytes
            test_report_push report5 assert "malformed cmap segment count" malformed_ok
    let report7 %TestReport match build_glyph_array_cmap_sfnt:
        Result::Err _message:
            test_report_push report6 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_glyph_id &bytes none 'A':
                Result::Err _error:
                    test_report_push report6 assert false
                Result::Ok glyph:
                    test_report_push report6 assert_eq_i32 "cmap glyph array A" 36 gui_glyph_id_raw &glyph
            io_bytebuf_free bytes
            next_report
    let report8 %TestReport match build_zero_glyph_array_cmap_sfnt:
        Result::Err _message:
            test_report_push report7 assert false
        Result::Ok bytes:
            let zero_entry_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MissingGlyphMapping
            io_bytebuf_free bytes
            test_report_push report7 assert "cmap glyph array zero rejected" zero_entry_ok
    let report9 %TestReport match build_out_of_range_glyph_array_cmap_sfnt:
        Result::Err _message:
            test_report_push report8 assert false
        Result::Ok bytes:
            let range_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MalformedCmapRecord
            io_bytebuf_free bytes
            test_report_push report8 assert "cmap glyph array range malformed" range_ok
    let report10 %TestReport match build_overlapping_cmap_subtable_sfnt:
        Result::Err _message:
            test_report_push report9 assert false
        Result::Ok bytes:
            let overlap_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MalformedCmapRecord
            io_bytebuf_free bytes
            test_report_push report9 assert "cmap subtable overlaps records" overlap_ok
    let report11 %TestReport match build_short_cmap_header_sfnt:
        Result::Err _message:
            test_report_push report10 assert false
        Result::Ok bytes:
            let short_header_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MalformedCmapRecord
            io_bytebuf_free bytes
            test_report_push report10 assert "short cmap header" short_header_ok
    match build_short_cmap_subtable_sfnt:
        Result::Err _message:
            test_report_push report11 assert false
        Result::Ok bytes:
            let short_subtable_ok %bool sfnt_cmap_error_is gui_sfnt_lookup_glyph_id &bytes none 'A' GuiSfntParseErrorKind::MalformedCmapRecord
            io_bytebuf_free bytes
            test_report_push report11 assert "short cmap subtable" short_subtable_ok

fn main %impure fn void i32 \void:
    let report0 %TestReport parse_valid_values
    let report1 %TestReport append_error_cases report0
    let report2 %TestReport append_name_cases report1
    let report3 %TestReport append_cmap_cases report2
    let shown test_report_print_stdout report3
    test_report_exit_code shown
```
