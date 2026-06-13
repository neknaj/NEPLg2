# GUI font SFNT glyf doctests

このファイルは、SFNT `loca` / `glyf` parser が platform font API ではなく explicit byte fixture だけから glyph header bounds と typed error を返すことを確認する。

## gui_sfnt_glyf_reads_header_bounds_and_typed_errors

short / long `loca` から glyph offset を読み、`glyf` header の bounds を返す。壊れた `head`、unsupported `loca` format、declared length 不足、empty glyph、inverted bounds は enum error として返す。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_sfnt_glyf_reads_header_bounds_and_typed_errors\" count=95 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"glyf glyph1 x min\" expected=\"-10\" actual=\"-10\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"glyf glyph1 y max\" expected=\"200\" actual=\"200\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"glyf long loca x max\" expected=\"90\" actual=\"90\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"topology contour count\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"topology point count\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"topology instruction length\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"topology point data offset\" expected=\"17\" actual=\"17\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"topology point data length\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=8 status=ok kind=eq_i32 label=\"point stream no-repeat flag offset\" expected=\"17\" actual=\"17\" message=\"\"\nassertion index=9 status=ok kind=eq_i32 label=\"point stream no-repeat raw flag length\" expected=\"4\" actual=\"4\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"point stream no-repeat x offset\" expected=\"21\" actual=\"21\" message=\"\"\nassertion index=11 status=ok kind=eq_i32 label=\"point stream no-repeat x length\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=12 status=ok kind=eq_i32 label=\"point stream no-repeat y offset\" expected=\"26\" actual=\"26\" message=\"\"\nassertion index=13 status=ok kind=eq_i32 label=\"point stream no-repeat y length\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=14 status=ok kind=eq_i32 label=\"point stream no-repeat trailing offset\" expected=\"31\" actual=\"31\" message=\"\"\nassertion index=15 status=ok kind=eq_i32 label=\"point stream no-repeat trailing length\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=16 status=ok kind=eq_i32 label=\"point stream repeat raw flag length\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=17 status=ok kind=eq_i32 label=\"point stream repeat x offset\" expected=\"20\" actual=\"20\" message=\"\"\nassertion index=18 status=ok kind=eq_i32 label=\"point stream repeat x length\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=19 status=ok kind=eq_i32 label=\"point stream repeat y length\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=20 status=ok kind=eq_i32 label=\"point stream repeat trailing length\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=21 status=ok kind=eq_i32 label=\"point stream repeat zero raw flag length\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=22 status=ok kind=eq_i32 label=\"point stream repeat zero x offset\" expected=\"16\" actual=\"16\" message=\"\"\nassertion index=23 status=ok kind=eq_i32 label=\"point stream repeat zero x length\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=24 status=ok kind=eq_i32 label=\"point stream repeat zero y length\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=25 status=ok kind=eq_i32 label=\"point stream repeat zero trailing length\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=26 status=ok kind=eq_i32 label=\"point decode no-repeat point0 x\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=27 status=ok kind=eq_i32 label=\"point decode no-repeat point0 y\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=28 status=ok kind=bool label=\"point decode no-repeat point0 off curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=29 status=ok kind=bool label=\"point decode no-repeat point0 not contour end\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=30 status=ok kind=eq_i32 label=\"point decode no-repeat endpoint index\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=31 status=ok kind=bool label=\"point decode no-repeat endpoint contour end\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=32 status=ok kind=eq_i32 label=\"point decode repeat cumulative x\" expected=\"6\" actual=\"6\" message=\"\"\nassertion index=33 status=ok kind=eq_i32 label=\"point decode repeat cumulative y\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=34 status=ok kind=bool label=\"point decode repeat off curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=35 status=ok kind=bool label=\"point decode repeat middle not contour end\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=36 status=ok kind=eq_i32 label=\"point decode repeat zero x\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=37 status=ok kind=eq_i32 label=\"point decode repeat zero y\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=38 status=ok kind=bool label=\"point decode repeat zero contour end\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=39 status=ok kind=eq_i32 label=\"point decode signed x\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=40 status=ok kind=eq_i32 label=\"point decode signed y\" expected=\"-6\" actual=\"-6\" message=\"\"\nassertion index=41 status=ok kind=bool label=\"point decode signed on curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=42 status=ok kind=eq_i32 label=\"contour span first start\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=43 status=ok kind=eq_i32 label=\"contour span first end\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=44 status=ok kind=eq_i32 label=\"contour span first count\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=45 status=ok kind=eq_i32 label=\"contour span second start\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=46 status=ok kind=eq_i32 label=\"contour span second end\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=47 status=ok kind=eq_i32 label=\"contour span second count\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=48 status=ok kind=eq_i32 label=\"contour span single start\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=49 status=ok kind=eq_i32 label=\"contour span single end\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=50 status=ok kind=eq_i32 label=\"contour span single count\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=51 status=ok kind=eq_i32 label=\"contour point first local index\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=52 status=ok kind=eq_i32 label=\"contour point first absolute index\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=53 status=ok kind=eq_i32 label=\"contour point first x\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=54 status=ok kind=eq_i32 label=\"contour point first y\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=55 status=ok kind=bool label=\"contour point first not contour end\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=56 status=ok kind=eq_i32 label=\"contour point second span index\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=57 status=ok kind=eq_i32 label=\"contour point second local index\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=58 status=ok kind=eq_i32 label=\"contour point second absolute index\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=59 status=ok kind=bool label=\"contour point second contour end\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=60 status=ok kind=eq_i32 label=\"contour point signed absolute index\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=61 status=ok kind=eq_i32 label=\"contour point signed x\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=62 status=ok kind=eq_i32 label=\"contour point signed y\" expected=\"-6\" actual=\"-6\" message=\"\"\nassertion index=63 status=ok kind=bool label=\"contour point signed on curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=64 status=ok kind=bool label=\"missing loca table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=65 status=ok kind=bool label=\"missing glyf table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=66 status=ok kind=bool label=\"short head for glyf\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=67 status=ok kind=bool label=\"unsupported loca format\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=68 status=ok kind=bool label=\"long loca high bit\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=69 status=ok kind=bool label=\"short loca declared length\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=70 status=ok kind=bool label=\"decreasing glyph offset\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=71 status=ok kind=bool label=\"empty glyph outline\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=72 status=ok kind=bool label=\"short glyf header\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=73 status=ok kind=bool label=\"inverted glyph bounds\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=74 status=ok kind=bool label=\"composite glyph unsupported\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=75 status=ok kind=bool label=\"zero contour topology\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=76 status=ok kind=bool label=\"non increasing endpoint\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=77 status=ok kind=bool label=\"short endpoint array\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=78 status=ok kind=bool label=\"short instruction length\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=79 status=ok kind=bool label=\"instruction overrun\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=80 status=ok kind=bool label=\"missing point data\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=81 status=ok kind=bool label=\"point stream repeat overrun\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=82 status=ok kind=bool label=\"point stream missing repeat byte\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=83 status=ok kind=bool label=\"point stream x coordinate overrun\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=84 status=ok kind=bool label=\"point stream y coordinate overrun\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=85 status=ok kind=bool label=\"point decode negative index missing\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=86 status=ok kind=bool label=\"point decode index count missing\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=87 status=ok kind=bool label=\"point decode x coordinate overrun\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=88 status=ok kind=bool label=\"point decode y coordinate overrun\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=89 status=ok kind=bool label=\"contour span negative index missing\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=90 status=ok kind=bool label=\"contour span index count missing\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=91 status=ok kind=bool label=\"contour span malformed endpoint observed\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=92 status=ok kind=bool label=\"contour point negative local missing\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=93 status=ok kind=bool label=\"contour point local count missing\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=94 status=ok kind=bool label=\"contour point x coordinate overrun\" expected=\"true\" actual=\"true\" message=\"\"\n"
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

fn sfnt_push_short_loca_glyph1_end %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\short_end:
    match sfnt_push_u16_be builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 short_end:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 short_end:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 short_end

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

fn sfnt_push_simple_topology_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 2 65526 65516 100 200:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 3:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 1:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 77:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u8 b5 1:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u8 b6 2:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            sfnt_push_u8 b7 3

fn sfnt_push_composite_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 65535 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_zero_run b1 10

fn sfnt_push_zero_contour_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 0 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_zero_run b1 10

fn sfnt_push_non_increasing_endpoint_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 2 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 2:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 2:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_zero_run b3 6

fn sfnt_push_short_endpoint_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 2 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u16_be b1 1

fn sfnt_push_short_instruction_length_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 1 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u16_be b1 0

fn sfnt_push_instruction_overrun_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 1 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 4:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 1:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u8 b4 2

fn sfnt_push_missing_point_data_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 1 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 2:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 1:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u8 b4 2

fn sfnt_push_two_contour_point_header %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 2 65526 65516 100 200:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 1:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 3:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 1:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u8 b4 77

fn sfnt_push_one_contour_point_header %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\instruction_length:
    match sfnt_push_glyf_header builder 1 0 0 10 10:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 instruction_length:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_zero_run b3 instruction_length

fn sfnt_push_one_contour_endpoint_header %impure fn ByteBuilder impure fn i32 impure fn i32 Result ByteBuilder str \builder\endpoint\instruction_length:
    match sfnt_push_glyf_header builder 1 0 65530 20 30:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 endpoint:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 instruction_length:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            sfnt_push_zero_run b3 instruction_length

fn sfnt_push_point_stream_no_repeat_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_two_contour_point_header builder:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 18:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 36:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 48:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_zero_run b5 10:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u8 b6 171:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_u8 b7 205:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    sfnt_push_u8 b8 239

fn sfnt_push_point_stream_repeat_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_two_contour_point_header builder:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 26:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 2:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 48:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 1:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u8 b5 2:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u8 b6 3:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_zero_run b7 6:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    sfnt_push_u8 b8 238

fn sfnt_push_point_stream_repeat_zero_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_one_contour_point_header builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 56:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 170:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u8 b4 187

fn sfnt_push_point_stream_repeat_overrun_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_one_contour_point_header builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 56:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u8 b2 1

fn sfnt_push_point_stream_missing_repeat_byte_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_one_contour_point_header builder 1:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_u8 b1 56

fn sfnt_push_point_stream_x_overrun_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_one_contour_point_header builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u8 b2 17

fn sfnt_push_point_stream_y_overrun_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_one_contour_point_header builder 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 16:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    sfnt_push_u8 b2 34

fn sfnt_push_point_decode_signed_glyf %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_one_contour_endpoint_header builder 2 0:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 19:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 7:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 49:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 5:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u8 b5 3:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u8 b6 255:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_u8 b7 254:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    sfnt_push_u8 b8 4

fn sfnt_push_topology_payload %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\kind:
    match kind:
        0:
            sfnt_push_simple_topology_glyf builder
        1:
            sfnt_push_composite_glyf builder
        2:
            sfnt_push_zero_contour_glyf builder
        3:
            sfnt_push_non_increasing_endpoint_glyf builder
        4:
            sfnt_push_short_endpoint_glyf builder
        5:
            sfnt_push_short_instruction_length_glyf builder
        6:
            sfnt_push_instruction_overrun_glyf builder
        7:
            sfnt_push_missing_point_data_glyf builder
        8:
            sfnt_push_point_stream_no_repeat_glyf builder
        9:
            sfnt_push_point_stream_repeat_glyf builder
        10:
            sfnt_push_point_stream_repeat_zero_glyf builder
        11:
            sfnt_push_point_stream_repeat_overrun_glyf builder
        12:
            sfnt_push_point_stream_missing_repeat_byte_glyf builder
        13:
            sfnt_push_point_stream_x_overrun_glyf builder
        14:
            sfnt_push_point_stream_y_overrun_glyf builder
        15:
            sfnt_push_point_decode_signed_glyf builder
        _:
            Result::Err "unknown topology payload"

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

fn build_topology_case_sfnt %impure fn i32 impure fn i32 impure fn i32 Result ByteBuf str \kind\glyf_length\short_end:
    match byte_builder_with_capacity 220:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_glyf_sfnt_prefix b0 52 0 4 10 glyf_length sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f':
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_short_loca_glyph1_end b1 short_end:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_topology_payload b2 kind

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
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _bounds:
            false

fn sfnt_glyf_topology_error_is %fn Result GuiSfntSimpleGlyphTopology GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphOutline:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _topology:
            false

fn sfnt_glyf_point_stream_error_is %fn Result GuiSfntSimpleGlyphPointStream GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphOutline:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _stream:
            false

fn sfnt_glyf_point_error_is %fn Result GuiSfntSimpleGlyphPoint GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphOutline:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _point:
            false

fn sfnt_glyf_contour_span_error_is %fn Result GuiSfntSimpleGlyphContourSpan GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphOutline:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _span:
            false

fn sfnt_glyf_contour_point_error_is %fn Result GuiSfntSimpleGlyphContourPoint GuiSfntParseError fn GuiSfntParseErrorKind bool \result\expected:
    match result:
        Result::Err error:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MalformedGlyfRecord:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::MissingGlyphOutline:
                            true
                        _:
                            false
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    match gui_sfnt_parse_error_kind &error:
                        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                            true
                        _:
                            false
                _:
                    false
        Result::Ok _contour_point:
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
    let report2 %TestReport match build_long_loca_glyf_sfnt:
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
    let report3 %TestReport match build_topology_case_sfnt 0 20 10:
        Result::Err _message:
            test_report_push report2 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1:
                Result::Err _error:
                    test_report_push report2 assert false
                Result::Ok topology:
                    let r1 %TestReport test_report_push report2 assert_eq_i32 "topology contour count" 2 gui_sfnt_simple_glyph_topology_contour_count &topology
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "topology point count" 4 gui_sfnt_simple_glyph_topology_point_count &topology
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "topology instruction length" 1 gui_sfnt_simple_glyph_topology_instruction_length &topology
                    let r4 %TestReport test_report_push r3 assert_eq_i32 "topology point data offset" 17 gui_sfnt_simple_glyph_topology_point_data_offset &topology
                    test_report_push r4 assert_eq_i32 "topology point data length" 3 gui_sfnt_simple_glyph_topology_point_data_length &topology
            io_bytebuf_free bytes
            next_report
    let report4 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report3 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1:
                Result::Err _error:
                    test_report_push report3 assert false
                Result::Ok stream:
                    let r1 %TestReport test_report_push report3 assert_eq_i32 "point stream no-repeat flag offset" 17 gui_sfnt_simple_glyph_point_stream_flag_data_offset &stream
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point stream no-repeat raw flag length" 4 gui_sfnt_simple_glyph_point_stream_flag_data_length &stream
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "point stream no-repeat x offset" 21 gui_sfnt_simple_glyph_point_stream_x_data_offset &stream
                    let r4 %TestReport test_report_push r3 assert_eq_i32 "point stream no-repeat x length" 5 gui_sfnt_simple_glyph_point_stream_x_data_length &stream
                    let r5 %TestReport test_report_push r4 assert_eq_i32 "point stream no-repeat y offset" 26 gui_sfnt_simple_glyph_point_stream_y_data_offset &stream
                    let r6 %TestReport test_report_push r5 assert_eq_i32 "point stream no-repeat y length" 5 gui_sfnt_simple_glyph_point_stream_y_data_length &stream
                    let r7 %TestReport test_report_push r6 assert_eq_i32 "point stream no-repeat trailing offset" 31 gui_sfnt_simple_glyph_point_stream_trailing_data_offset &stream
                    test_report_push r7 assert_eq_i32 "point stream no-repeat trailing length" 3 gui_sfnt_simple_glyph_point_stream_trailing_data_length &stream
            io_bytebuf_free bytes
            next_report
    let report5 %TestReport match build_topology_case_sfnt 9 30 15:
        Result::Err _message:
            test_report_push report4 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1:
                Result::Err _error:
                    test_report_push report4 assert false
                Result::Ok stream:
                    let r1 %TestReport test_report_push report4 assert_eq_i32 "point stream repeat raw flag length" 3 gui_sfnt_simple_glyph_point_stream_flag_data_length &stream
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point stream repeat x offset" 20 gui_sfnt_simple_glyph_point_stream_x_data_offset &stream
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "point stream repeat x length" 3 gui_sfnt_simple_glyph_point_stream_x_data_length &stream
                    let r4 %TestReport test_report_push r3 assert_eq_i32 "point stream repeat y length" 6 gui_sfnt_simple_glyph_point_stream_y_data_length &stream
                    test_report_push r4 assert_eq_i32 "point stream repeat trailing length" 1 gui_sfnt_simple_glyph_point_stream_trailing_data_length &stream
            io_bytebuf_free bytes
            next_report
    let report6 %TestReport match build_topology_case_sfnt 10 18 9:
        Result::Err _message:
            test_report_push report5 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1:
                Result::Err _error:
                    test_report_push report5 assert false
                Result::Ok stream:
                    let r1 %TestReport test_report_push report5 assert_eq_i32 "point stream repeat zero raw flag length" 2 gui_sfnt_simple_glyph_point_stream_flag_data_length &stream
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point stream repeat zero x offset" 16 gui_sfnt_simple_glyph_point_stream_x_data_offset &stream
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "point stream repeat zero x length" 0 gui_sfnt_simple_glyph_point_stream_x_data_length &stream
                    let r4 %TestReport test_report_push r3 assert_eq_i32 "point stream repeat zero y length" 0 gui_sfnt_simple_glyph_point_stream_y_data_length &stream
                    test_report_push r4 assert_eq_i32 "point stream repeat zero trailing length" 2 gui_sfnt_simple_glyph_point_stream_trailing_data_length &stream
            io_bytebuf_free bytes
            next_report
    let report7 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report6 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 0:
                Result::Err _error:
                    test_report_push report6 assert false
                Result::Ok point:
                    let r1 %TestReport test_report_push report6 assert_eq_i32 "point decode no-repeat point0 x" 0 gui_sfnt_simple_glyph_point_x &point
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point decode no-repeat point0 y" 0 gui_sfnt_simple_glyph_point_y &point
                    let r3 %TestReport test_report_push r2 assert "point decode no-repeat point0 off curve" not gui_sfnt_simple_glyph_point_on_curve &point
                    test_report_push r3 assert "point decode no-repeat point0 not contour end" not gui_sfnt_simple_glyph_point_end_of_contour &point
            io_bytebuf_free bytes
            next_report
    let report8 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report7 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 3:
                Result::Err _error:
                    test_report_push report7 assert false
                Result::Ok point:
                    let r1 %TestReport test_report_push report7 assert_eq_i32 "point decode no-repeat endpoint index" 3 gui_sfnt_simple_glyph_point_index &point
                    test_report_push r1 assert "point decode no-repeat endpoint contour end" gui_sfnt_simple_glyph_point_end_of_contour &point
            io_bytebuf_free bytes
            next_report
    let report9 %TestReport match build_topology_case_sfnt 9 30 15:
        Result::Err _message:
            test_report_push report8 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 2:
                Result::Err _error:
                    test_report_push report8 assert false
                Result::Ok point:
                    let r1 %TestReport test_report_push report8 assert_eq_i32 "point decode repeat cumulative x" 6 gui_sfnt_simple_glyph_point_x &point
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point decode repeat cumulative y" 0 gui_sfnt_simple_glyph_point_y &point
                    let r3 %TestReport test_report_push r2 assert "point decode repeat off curve" not gui_sfnt_simple_glyph_point_on_curve &point
                    test_report_push r3 assert "point decode repeat middle not contour end" not gui_sfnt_simple_glyph_point_end_of_contour &point
            io_bytebuf_free bytes
            next_report
    let report10 %TestReport match build_topology_case_sfnt 10 18 9:
        Result::Err _message:
            test_report_push report9 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 0:
                Result::Err _error:
                    test_report_push report9 assert false
                Result::Ok point:
                    let r1 %TestReport test_report_push report9 assert_eq_i32 "point decode repeat zero x" 0 gui_sfnt_simple_glyph_point_x &point
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point decode repeat zero y" 0 gui_sfnt_simple_glyph_point_y &point
                    test_report_push r2 assert "point decode repeat zero contour end" gui_sfnt_simple_glyph_point_end_of_contour &point
            io_bytebuf_free bytes
            next_report
    let report11 %TestReport match build_topology_case_sfnt 15 22 11:
        Result::Err _message:
            test_report_push report10 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 1:
                Result::Err _error:
                    test_report_push report10 assert false
                Result::Ok point:
                    let r1 %TestReport test_report_push report10 assert_eq_i32 "point decode signed x" 2 gui_sfnt_simple_glyph_point_x &point
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "point decode signed y" -6 gui_sfnt_simple_glyph_point_y &point
                    test_report_push r2 assert "point decode signed on curve" gui_sfnt_simple_glyph_point_on_curve &point
            io_bytebuf_free bytes
            next_report
    let report12 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report11 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_contour_span &bytes none glyph1 0:
                Result::Err _error:
                    test_report_push report11 assert false
                Result::Ok span:
                    let r1 %TestReport test_report_push report11 assert_eq_i32 "contour span first start" 0 gui_sfnt_simple_glyph_contour_span_start_point_index &span
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "contour span first end" 1 gui_sfnt_simple_glyph_contour_span_end_point_index &span
                    test_report_push r2 assert_eq_i32 "contour span first count" 2 gui_sfnt_simple_glyph_contour_span_point_count &span
            io_bytebuf_free bytes
            next_report
    let report13 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report12 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_contour_span &bytes none glyph1 1:
                Result::Err _error:
                    test_report_push report12 assert false
                Result::Ok span:
                    let r1 %TestReport test_report_push report12 assert_eq_i32 "contour span second start" 2 gui_sfnt_simple_glyph_contour_span_start_point_index &span
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "contour span second end" 3 gui_sfnt_simple_glyph_contour_span_end_point_index &span
                    test_report_push r2 assert_eq_i32 "contour span second count" 2 gui_sfnt_simple_glyph_contour_span_point_count &span
            io_bytebuf_free bytes
            next_report
    let report14 %TestReport match build_topology_case_sfnt 15 22 11:
        Result::Err _message:
            test_report_push report13 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_contour_span &bytes none glyph1 0:
                Result::Err _error:
                    test_report_push report13 assert false
                Result::Ok span:
                    let r1 %TestReport test_report_push report13 assert_eq_i32 "contour span single start" 0 gui_sfnt_simple_glyph_contour_span_start_point_index &span
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "contour span single end" 2 gui_sfnt_simple_glyph_contour_span_end_point_index &span
                    test_report_push r2 assert_eq_i32 "contour span single count" 3 gui_sfnt_simple_glyph_contour_span_point_count &span
            io_bytebuf_free bytes
            next_report
    let report15 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report14 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_contour_point &bytes none glyph1 0 0:
                Result::Err _error:
                    test_report_push report14 assert false
                Result::Ok contour_point:
                    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &contour_point
                    let r1 %TestReport test_report_push report14 assert_eq_i32 "contour point first local index" 0 gui_sfnt_simple_glyph_contour_point_local_index &contour_point
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "contour point first absolute index" 0 gui_sfnt_simple_glyph_point_index &point
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "contour point first x" 0 gui_sfnt_simple_glyph_point_x &point
                    let r4 %TestReport test_report_push r3 assert_eq_i32 "contour point first y" 0 gui_sfnt_simple_glyph_point_y &point
                    test_report_push r4 assert "contour point first not contour end" not gui_sfnt_simple_glyph_point_end_of_contour &point
            io_bytebuf_free bytes
            next_report
    let report16 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report15 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_contour_point &bytes none glyph1 1 1:
                Result::Err _error:
                    test_report_push report15 assert false
                Result::Ok contour_point:
                    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &contour_point
                    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_point_span &contour_point
                    let r1 %TestReport test_report_push report15 assert_eq_i32 "contour point second span index" 1 gui_sfnt_simple_glyph_contour_span_index &span
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "contour point second local index" 1 gui_sfnt_simple_glyph_contour_point_local_index &contour_point
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "contour point second absolute index" 3 gui_sfnt_simple_glyph_point_index &point
                    test_report_push r3 assert "contour point second contour end" gui_sfnt_simple_glyph_point_end_of_contour &point
            io_bytebuf_free bytes
            next_report
    match build_topology_case_sfnt 15 22 11:
        Result::Err _message:
            test_report_push report16 assert false
        Result::Ok bytes:
            let next_report %TestReport match gui_sfnt_lookup_simple_glyph_contour_point &bytes none glyph1 0 1:
                Result::Err _error:
                    test_report_push report16 assert false
                Result::Ok contour_point:
                    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &contour_point
                    let r1 %TestReport test_report_push report16 assert_eq_i32 "contour point signed absolute index" 1 gui_sfnt_simple_glyph_point_index &point
                    let r2 %TestReport test_report_push r1 assert_eq_i32 "contour point signed x" 2 gui_sfnt_simple_glyph_point_x &point
                    let r3 %TestReport test_report_push r2 assert_eq_i32 "contour point signed y" -6 gui_sfnt_simple_glyph_point_y &point
                    test_report_push r3 assert "contour point signed on curve" gui_sfnt_simple_glyph_point_on_curve &point
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
    let report10 %TestReport match build_inverted_bounds_sfnt:
        Result::Err _message:
            test_report_push report9 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_error_is gui_sfnt_lookup_glyph_bounds &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report9 assert "inverted glyph bounds" ok
    let report11 %TestReport match build_topology_case_sfnt 1 20 10:
        Result::Err _message:
            test_report_push report10 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat
            io_bytebuf_free bytes
            test_report_push report10 assert "composite glyph unsupported" ok
    let report12 %TestReport match build_topology_case_sfnt 2 20 10:
        Result::Err _message:
            test_report_push report11 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report11 assert "zero contour topology" ok
    let report13 %TestReport match build_topology_case_sfnt 3 20 10:
        Result::Err _message:
            test_report_push report12 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report12 assert "non increasing endpoint" ok
    let report14 %TestReport match build_topology_case_sfnt 4 12 6:
        Result::Err _message:
            test_report_push report13 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report13 assert "short endpoint array" ok
    let report15 %TestReport match build_topology_case_sfnt 5 12 6:
        Result::Err _message:
            test_report_push report14 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report14 assert "short instruction length" ok
    let report16 %TestReport match build_topology_case_sfnt 6 16 8:
        Result::Err _message:
            test_report_push report15 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report15 assert "instruction overrun" ok
    let report17 %TestReport match build_topology_case_sfnt 7 16 8:
        Result::Err _message:
            test_report_push report16 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_topology_error_is gui_sfnt_lookup_simple_glyph_topology &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report16 assert "missing point data" ok
    let report18 %TestReport match build_topology_case_sfnt 11 16 8:
        Result::Err _message:
            test_report_push report17 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_stream_error_is gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report17 assert "point stream repeat overrun" ok
    let report19 %TestReport match build_topology_case_sfnt 12 16 8:
        Result::Err _message:
            test_report_push report18 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_stream_error_is gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report18 assert "point stream missing repeat byte" ok
    let report20 %TestReport match build_topology_case_sfnt 13 16 8:
        Result::Err _message:
            test_report_push report19 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_stream_error_is gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report19 assert "point stream x coordinate overrun" ok
    let report21 %TestReport match build_topology_case_sfnt 14 16 8:
        Result::Err _message:
            test_report_push report20 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_stream_error_is gui_sfnt_lookup_simple_glyph_point_stream &bytes none glyph1 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report20 assert "point stream y coordinate overrun" ok
    let report22 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report21 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_error_is gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 -1 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report21 assert "point decode negative index missing" ok
    let report23 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report22 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_error_is gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 4 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report22 assert "point decode index count missing" ok
    let report24 %TestReport match build_topology_case_sfnt 13 16 8:
        Result::Err _message:
            test_report_push report23 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_error_is gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 0 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report23 assert "point decode x coordinate overrun" ok
    let report25 %TestReport match build_topology_case_sfnt 14 16 8:
        Result::Err _message:
            test_report_push report24 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_point_error_is gui_sfnt_lookup_simple_glyph_point &bytes none glyph1 0 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report24 assert "point decode y coordinate overrun" ok
    let report26 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report25 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_contour_span_error_is gui_sfnt_lookup_simple_glyph_contour_span &bytes none glyph1 -1 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report25 assert "contour span negative index missing" ok
    let report27 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report26 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_contour_span_error_is gui_sfnt_lookup_simple_glyph_contour_span &bytes none glyph1 2 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report26 assert "contour span index count missing" ok
    let report28 %TestReport match build_topology_case_sfnt 3 20 10:
        Result::Err _message:
            test_report_push report27 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_contour_span_error_is gui_sfnt_lookup_simple_glyph_contour_span &bytes none glyph1 0 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report27 assert "contour span malformed endpoint observed" ok
    let report29 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report28 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_contour_point_error_is gui_sfnt_lookup_simple_glyph_contour_point &bytes none glyph1 0 -1 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report28 assert "contour point negative local missing" ok
    let report30 %TestReport match build_topology_case_sfnt 8 34 17:
        Result::Err _message:
            test_report_push report29 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_contour_point_error_is gui_sfnt_lookup_simple_glyph_contour_point &bytes none glyph1 0 2 GuiSfntParseErrorKind::MissingGlyphOutline
            io_bytebuf_free bytes
            test_report_push report29 assert "contour point local count missing" ok
    match build_topology_case_sfnt 13 16 8:
        Result::Err _message:
            test_report_push report30 assert false
        Result::Ok bytes:
            let ok %bool sfnt_glyf_contour_point_error_is gui_sfnt_lookup_simple_glyph_contour_point &bytes none glyph1 0 0 GuiSfntParseErrorKind::MalformedGlyfRecord
            io_bytebuf_free bytes
            test_report_push report30 assert "contour point x coordinate overrun" ok

fn main %impure fn void i32 \void:
    let report0 %TestReport test_report_new "gui_sfnt_glyf_reads_header_bounds_and_typed_errors"
    let report1 %TestReport append_success_cases report0
    let report2 %TestReport append_error_cases report1
    let shown test_report_print_stdout report2
    test_report_exit_code shown
```
