# alloc/gui/font registered face

このファイルは `alloc/gui/font/registered_face` が platform font API や browser `FontFace` ではなく、provider bytes owner と SFNT metadata parser を接続することを確認する。

## gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata

[目的/もくてき]:
- `GuiFontResourceBytes` に含まれる byte payload と face index を SFNT metadata parser に渡します。
- 成功時は resource id、face id、selected face index、resource owner、metadata を同じ owner として保持します。
- 失敗時は typed registered face error と parser error を返し、resource owner を回収して解放できます。
- WOFF decode 境界が来るまでは `SfntOnly` 以外の decode policy を parse 前に拒否します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata\" count=32 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"resource id\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"face id\" expected=\"11\" actual=\"11\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"selected face index\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"owner resource len\" expected=\"96\" actual=\"96\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"metadata face index\" expected=\"0\" actual=\"0\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"metadata face count\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"units per em\" expected=\"2048\" actual=\"2048\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"glyph count\" expected=\"321\" actual=\"321\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"invalid face registered kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"invalid face parse kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=eq_i32 label=\"invalid face owner len\" expected=\"96\" actual=\"96\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"malformed registered kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"malformed parse kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=eq_i32 label=\"malformed owner len\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"unsupported decode kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=15 status=ok kind=bool label=\"unsupported decode no parse\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=16 status=ok kind=eq_i32 label=\"unsupported decode owner len\" expected=\"96\" actual=\"96\" message=\"\"\nassertion index=17 status=ok kind=bool label=\"invalid raw face id rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=18 status=ok kind=bool label=\"registered face table success\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=19 status=ok kind=bool label=\"registered face table duplicate recovery\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=20 status=ok kind=bool label=\"registered face table duplicate face recovery\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=21 status=ok kind=bool label=\"registered face glyph lookup success and missing recovery\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=22 status=ok kind=bool label=\"registered face horizontal metric success\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=23 status=ok kind=bool label=\"registered face horizontal metric missing table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=24 status=ok kind=bool label=\"registered face horizontal metric malformed table\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=25 status=ok kind=bool label=\"registered face horizontal metric mapping mismatch\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=26 status=ok kind=bool label=\"registered face simple glyph success and composite rejection\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=27 status=ok kind=bool label=\"registered face simple glyph linear collection and indexed contour endpoint population\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=28 status=ok kind=bool label=\"registered face simple glyph missing loca\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=29 status=ok kind=bool label=\"registered face simple glyph missing glyf\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=30 status=ok kind=bool label=\"registered face simple glyph malformed point data\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=31 status=ok kind=bool label=\"registered face simple glyph mapping mismatch\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font" as *
#import "alloc/io" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *
#import "std/gui/font_resource" as *
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

fn sfnt_push_cmap_hmtx_records %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\hmtx_length:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 92 20:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 112 36:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 148 6:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_record b3 sfnt_tag4 'c' 'm' 'a' 'p' 154 44:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_record b4 sfnt_tag4 'h' 'm' 't' 'x' 198 hmtx_length

fn sfnt_push_valid_hmtx_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u16_be builder 600:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 0:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_zero_run b2 70:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 20:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_zero_run b4 6

fn sfnt_push_format4_ab_segment %impure fn ByteBuilder Result ByteBuilder str \builder:
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
                            match sfnt_push_u16_be b3 4:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_zero_run b4 6:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u16_be b5 'B':
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
                                                                                    match sfnt_push_u16_be b10 65507:
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

fn sfnt_push_valid_cmap_ab_table %impure fn ByteBuilder Result ByteBuilder str \builder:
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
                            sfnt_push_format4_ab_segment b3

fn sfnt_push_comprehensive_records %impure fn ByteBuilder impure fn i32 impure fn i32 impure fn i32 Result ByteBuilder str \builder\glyf_length\loca_tag\glyf_tag:
    match sfnt_push_record builder sfnt_tag4 'h' 'e' 'a' 'd' 124 52:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_record b1 sfnt_tag4 'h' 'h' 'e' 'a' 176 36:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_record b2 sfnt_tag4 'm' 'a' 'x' 'p' 212 6:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_record b3 sfnt_tag4 'c' 'm' 'a' 'p' 218 44:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_record b4 sfnt_tag4 'h' 'm' 't' 'x' 262 82:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_record b5 loca_tag 344 82:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    sfnt_push_record b6 glyf_tag 426 glyf_length

fn sfnt_push_comprehensive_loca %impure fn ByteBuilder impure fn bool Result ByteBuilder str \builder\malformed:
    match sfnt_push_zero_run builder 74:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            let simple_end %i32 if malformed then 15 else 17
            let composite_end %i32 if malformed then 20 else 22
            match sfnt_push_u16_be b1 simple_end:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 composite_end:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 composite_end:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 composite_end

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

fn sfnt_push_comprehensive_simple_glyph %impure fn ByteBuilder impure fn bool Result ByteBuilder str \builder\malformed:
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
                                            match sfnt_push_u8 b5 0:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u8 b6 18:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_u8 b7 36:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    match sfnt_push_u8 b8 48:
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok b9:
                                                                            if:
                                                                                malformed
                                                                                then:
                                                                                    sfnt_push_zero_run b9 9
                                                                                else:
                                                                                    match sfnt_push_zero_run b9 10:
                                                                                        Result::Err message:
                                                                                            Result::Err message
                                                                                        Result::Ok b10:
                                                                                            match sfnt_push_u8 b10 171:
                                                                                                Result::Err message:
                                                                                                    Result::Err message
                                                                                                Result::Ok b11:
                                                                                                    match sfnt_push_u8 b11 205:
                                                                                                        Result::Err message:
                                                                                                            Result::Err message
                                                                                                        Result::Ok b12:
                                                                                                            sfnt_push_u8 b12 239

fn sfnt_push_comprehensive_glyf %impure fn ByteBuilder impure fn bool Result ByteBuilder str \builder\malformed:
    match sfnt_push_comprehensive_simple_glyph builder malformed:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            sfnt_push_glyf_header b1 65535 0 0 10 10

fn sfnt_push_cmap_hmtx_tables %impure fn ByteBuilder Result ByteBuilder str \builder:
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
                                                    match sfnt_push_zero_run b6 24:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            match sfnt_push_u16_be b7 1:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    match sfnt_push_u32_be b8 65536:
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok b9:
                                                                            match sfnt_push_u16_be b9 40:
                                                                                Result::Err message:
                                                                                    Result::Err message
                                                                                Result::Ok b10:
                                                                                    match sfnt_push_valid_cmap_table b10:
                                                                                        Result::Err message:
                                                                                            Result::Err message
                                                                                        Result::Ok b11:
                                                                                            sfnt_push_valid_hmtx_table b11

fn sfnt_push_comprehensive_tables %impure fn ByteBuilder impure fn bool Result ByteBuilder str \builder\malformed:
    match sfnt_push_zero_run builder 18:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 2048:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_zero_run b2 30:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 0:
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
                                                            match sfnt_push_u16_be b7 200:
                                                                Result::Err message:
                                                                    Result::Err message
                                                                Result::Ok b8:
                                                                    match sfnt_push_zero_run b8 24:
                                                                        Result::Err message:
                                                                            Result::Err message
                                                                        Result::Ok b9:
                                                                            match sfnt_push_u16_be b9 1:
                                                                                Result::Err message:
                                                                                    Result::Err message
                                                                                Result::Ok b10:
                                                                                    match sfnt_push_u32_be b10 65536:
                                                                                        Result::Err message:
                                                                                            Result::Err message
                                                                                        Result::Ok b11:
                                                                                            match sfnt_push_u16_be b11 40:
                                                                                                Result::Err message:
                                                                                                    Result::Err message
                                                                                                Result::Ok b12:
                                                                                                    match sfnt_push_valid_cmap_ab_table b12:
                                                                                                        Result::Err message:
                                                                                                            Result::Err message
                                                                                                        Result::Ok b13:
                                                                                                            match sfnt_push_valid_hmtx_table b13:
                                                                                                                Result::Err message:
                                                                                                                    Result::Err message
                                                                                                                Result::Ok b14:
                                                                                                                    match sfnt_push_comprehensive_loca b14 malformed:
                                                                                                                        Result::Err message:
                                                                                                                            Result::Err message
                                                                                                                        Result::Ok b15:
                                                                                                                            sfnt_push_comprehensive_glyf b15 malformed

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

fn build_cmap_hmtx_sfnt %impure fn i32 Result ByteBuf str \hmtx_length:
    match byte_builder_with_capacity 280:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 5:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_cmap_hmtx_records b1 hmtx_length:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_cmap_hmtx_tables b2

fn build_comprehensive_glyph_sfnt %impure fn i32 impure fn i32 impure fn bool Result ByteBuf str \loca_tag\glyf_tag\malformed:
    let glyf_length %i32 if malformed then 40 else 44
    match byte_builder_with_capacity 470:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 7:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match sfnt_push_comprehensive_records b1 glyf_length loca_tag glyf_tag:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                sfnt_push_comprehensive_tables b2 malformed

fn registered_face_resource_from_bytes %fn ByteBuf fn Option i32 fn GuiFontDecodePolicy GuiFontResourceBytes \bytes\face_index\decode_policy:
    let path %GuiFontResourcePath unwrap_ok gui_font_resource_path_result "fonts/Test-Regular.ttf"
    let request %GuiFontResourceRequest unwrap_ok gui_font_resource_request path face_index none decode_policy
    gui_font_resource_bytes_new request GuiFontResourceSource::Vfs bytes

fn registered_face_error_kind_is %fn &GuiFontRegisteredFaceError fn GuiFontRegisteredFaceErrorKind bool \error\expected:
    gui_font_registered_face_error_kind_eq gui_font_registered_face_error_kind error expected

fn sfnt_parse_error_kind_is %fn GuiSfntParseErrorKind fn GuiSfntParseErrorKind bool \actual\expected:
    match actual:
        GuiSfntParseErrorKind::UnexpectedEof:
            match expected:
                GuiSfntParseErrorKind::UnexpectedEof:
                    true
                _:
                    false
        GuiSfntParseErrorKind::InvalidFaceIndex:
            match expected:
                GuiSfntParseErrorKind::InvalidFaceIndex:
                    true
                _:
                    false
        GuiSfntParseErrorKind::MissingGlyphMapping:
            match expected:
                GuiSfntParseErrorKind::MissingGlyphMapping:
                    true
                _:
                    false
        GuiSfntParseErrorKind::MissingTable:
            match expected:
                GuiSfntParseErrorKind::MissingTable:
                    true
                _:
                    false
        GuiSfntParseErrorKind::MalformedHmtxRecord:
            match expected:
                GuiSfntParseErrorKind::MalformedHmtxRecord:
                    true
                _:
                    false
        GuiSfntParseErrorKind::MalformedGlyfRecord:
            match expected:
                GuiSfntParseErrorKind::MalformedGlyfRecord:
                    true
                _:
                    false
        GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
            match expected:
                GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat:
                    true
                _:
                    false
        _:
            false

fn registered_face_parse_error_is %fn &GuiFontRegisteredFaceError fn GuiSfntParseErrorKind bool \error\expected:
    match gui_font_registered_face_error_parse_error error:
        Option::None:
            false
        Option::Some parse_error:
            sfnt_parse_error_kind_is gui_sfnt_parse_error_kind &parse_error expected

fn registered_face_parse_error_absent %fn &GuiFontRegisteredFaceError bool \error:
    match gui_font_registered_face_error_parse_error error:
        Option::None:
            true
        Option::Some _parse_error:
            false

fn invalid_raw_face_id_rejected %fn void bool \void:
    match gui_font_registered_face_request_from_raw 7 0:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _request:
            false

fn parse_valid_registered_face %impure fn void TestReport \void:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push test_report_new "gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata" assert false
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
            let report %TestReport match gui_font_registered_face_register_bytes registered_request resource:
                Result::Err error:
                    gui_font_registered_face_error_free error
                    test_report_push test_report_new "gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata" assert false
                Result::Ok face:
                    let resource_id %GuiFontResourceId gui_font_registered_face_resource_id &face
                    let face_id %GuiFontFaceId gui_font_registered_face_face_id &face
                    let owner_resource %&GuiFontResourceBytes gui_font_registered_face_resource_ref &face
                    let metadata %GuiSfntMetadata gui_font_registered_face_metadata &face
                    let metrics %GuiSfntMetrics gui_sfnt_metadata_metrics &metadata
                    let report0 %TestReport test_report_new "gui_font_registered_face_binds_resource_bytes_to_sfnt_metadata"
                    let report1 %TestReport test_report_push report0 assert_eq_i32 "resource id" 7 gui_font_resource_id_raw &resource_id
                    let report2 %TestReport test_report_push report1 assert_eq_i32 "face id" 11 gui_font_face_id_raw &face_id
                    let report3 %TestReport test_report_push report2 assert_eq_i32 "selected face index" 0 gui_font_registered_face_selected_face_index &face
                    let report4 %TestReport test_report_push report3 assert_eq_i32 "owner resource len" 96 gui_font_resource_bytes_len owner_resource
                    let report5 %TestReport test_report_push report4 assert_eq_i32 "metadata face index" 0 gui_sfnt_metadata_face_index &metadata
                    let report6 %TestReport test_report_push report5 assert_eq_i32 "metadata face count" 1 gui_sfnt_metadata_face_count &metadata
                    let report7 %TestReport test_report_push report6 assert_eq_i32 "units per em" 2048 gui_sfnt_metrics_units_per_em &metrics
                    let report8 %TestReport test_report_push report7 assert_eq_i32 "glyph count" 321 gui_sfnt_metrics_num_glyphs &metrics
                    gui_font_registered_face_free face
                    report8
            report

fn append_invalid_face_registered_case %impure fn TestReport TestReport \report0:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes some 1 GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    gui_font_registered_face_free face
                    test_report_push report0 assert false
                Result::Err error:
                    let kind_ok %bool registered_face_error_kind_is &error GuiFontRegisteredFaceErrorKind::InvalidFaceIndex
                    let parse_ok %bool registered_face_parse_error_is &error GuiSfntParseErrorKind::InvalidFaceIndex
                    let owner_len %i32 gui_font_resource_bytes_len gui_font_registered_face_error_resource_ref &error
                    gui_font_registered_face_error_free error
                    let report1 %TestReport test_report_push report0 assert "invalid face registered kind" kind_ok
                    let report2 %TestReport test_report_push report1 assert "invalid face parse kind" parse_ok
                    test_report_push report2 assert_eq_i32 "invalid face owner len" 96 owner_len

fn append_malformed_registered_case %impure fn TestReport TestReport \report0:
    let bytes %ByteBuf unwrap_ok io_bytebuf_from_str_result "AB"
    let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
    let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
    match gui_font_registered_face_register_bytes registered_request resource:
        Result::Ok face:
            gui_font_registered_face_free face
            test_report_push report0 assert false
        Result::Err error:
            let kind_ok %bool registered_face_error_kind_is &error GuiFontRegisteredFaceErrorKind::MalformedFontResource
            let parse_ok %bool registered_face_parse_error_is &error GuiSfntParseErrorKind::UnexpectedEof
            let owner_len %i32 gui_font_resource_bytes_len gui_font_registered_face_error_resource_ref &error
            gui_font_registered_face_error_free error
            let report1 %TestReport test_report_push report0 assert "malformed registered kind" kind_ok
            let report2 %TestReport test_report_push report1 assert "malformed parse kind" parse_ok
            test_report_push report2 assert_eq_i32 "malformed owner len" 2 owner_len

fn append_unsupported_decode_case %impure fn TestReport TestReport \report0:
    match build_valid_sfnt:
        Result::Err _message:
            test_report_push report0 assert false
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntAndWoff
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 7 11
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    gui_font_registered_face_free face
                    test_report_push report0 assert false
                Result::Err error:
                    let kind_ok %bool registered_face_error_kind_is &error GuiFontRegisteredFaceErrorKind::UnsupportedDecodePolicy
                    let parse_absent %bool registered_face_parse_error_absent &error
                    let owner_len %i32 gui_font_resource_bytes_len gui_font_registered_face_error_resource_ref &error
                    gui_font_registered_face_error_free error
                    let report1 %TestReport test_report_push report0 assert "unsupported decode kind" kind_ok
                    let report2 %TestReport test_report_push report1 assert "unsupported decode no parse" parse_absent
                    test_report_push report2 assert_eq_i32 "unsupported decode owner len" 96 owner_len

fn registered_face_table_register_error_kind_is %fn &GuiFontRegisteredFaceTableRegisterError fn GuiFontRegisteredFaceTableRegisterErrorKind bool \error\expected:
    gui_font_registered_face_table_register_error_kind_eq gui_font_registered_face_table_register_error_kind error expected

fn registered_face_table_decode_policy_is_sfnt_only %fn GuiFontDecodePolicy bool \policy:
    match policy:
        GuiFontDecodePolicy::SfntOnly:
            true
        _:
            false

fn build_registered_face_for_table %impure fn i32 impure fn i32 Result GuiFontRegisteredFace str \resource_raw\face_raw:
    match build_valid_sfnt:
        Result::Err message:
            Result::Err message
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw resource_raw face_raw
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    Result::Ok face
                Result::Err error:
                    gui_font_registered_face_error_free error
                    Result::Err "register"

fn build_registered_face_for_glyph_lookup %impure fn i32 impure fn i32 Result GuiFontRegisteredFace str \resource_raw\face_raw:
    match build_valid_cmap_sfnt:
        Result::Err message:
            Result::Err message
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw resource_raw face_raw
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    Result::Ok face
                Result::Err error:
                    gui_font_registered_face_error_free error
                    Result::Err "register"

fn build_registered_face_for_horizontal_metric %impure fn i32 impure fn i32 impure fn i32 Result GuiFontRegisteredFace str \resource_raw\face_raw\hmtx_length:
    match build_cmap_hmtx_sfnt hmtx_length:
        Result::Err message:
            Result::Err message
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw resource_raw face_raw
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    Result::Ok face
                Result::Err error:
                    gui_font_registered_face_error_free error
                    Result::Err "register"

fn build_registered_face_for_simple_glyph %impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn bool Result GuiFontRegisteredFace str \resource_raw\face_raw\loca_tag\glyf_tag\malformed:
    match build_comprehensive_glyph_sfnt loca_tag glyf_tag malformed:
        Result::Err message:
            Result::Err message
        Result::Ok bytes:
            let resource %GuiFontResourceBytes registered_face_resource_from_bytes bytes none GuiFontDecodePolicy::SfntOnly
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw resource_raw face_raw
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Ok face:
                    Result::Ok face
                Result::Err error:
                    gui_font_registered_face_error_free error
                    Result::Err "register"

fn registered_face_table_success_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let resource_id %GuiFontResourceId gui_font_registered_face_record_resource_id &record
    let face_id %GuiFontFaceId gui_font_registered_face_record_face_id &record
    let face_ref %&GuiFontRegisteredFace gui_font_registered_face_table_entry_face_ref &entry
    let owner_resource %&GuiFontResourceBytes gui_font_registered_face_resource_ref face_ref
    let len_ok %bool eq 1 gui_font_registered_face_table_len &table
    let record_ok %bool and eq 13 gui_font_registered_face_record_resource_raw &record eq 17 gui_font_registered_face_record_face_raw &record
    let metadata_ok %bool and eq 0 gui_font_registered_face_record_selected_face_index &record and eq 1 gui_font_registered_face_record_face_count &record and eq 2048 gui_font_registered_face_record_units_per_em &record eq 321 gui_font_registered_face_record_glyph_count &record
    let resource_ok %bool and eq 96 gui_font_registered_face_record_byte_len &record and eq 96 gui_font_resource_bytes_len owner_resource registered_face_table_decode_policy_is_sfnt_only gui_font_registered_face_record_decode_policy &record
    let lookup_resource_ok %bool match gui_font_registered_face_table_lookup_resource_id &table resource_id:
        Option::Some lookup:
            eq 17 gui_font_registered_face_record_face_raw &lookup
        Option::None:
            false
    let lookup_face_ok %bool match gui_font_registered_face_table_lookup_face_id &table face_id:
        Option::Some lookup:
            eq 13 gui_font_registered_face_record_resource_raw &lookup
        Option::None:
            false
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and len_ok and record_ok and metadata_ok and resource_ok and lookup_resource_ok lookup_face_ok

fn registered_face_table_success_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 2:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_table 13 17:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_table_success_callback

fn registered_face_table_duplicate_rejected_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFace bool \table\face:
    let face_resource %&GuiFontResourceBytes gui_font_registered_face_resource_ref &face
    let recovered_ok %bool and eq 1 gui_font_registered_face_table_len &table eq 96 gui_font_resource_bytes_len face_resource
    gui_font_registered_face_table_free table
    gui_font_registered_face_free face
    recovered_ok

fn registered_face_table_duplicate_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let first_record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let resource_raw %i32 gui_font_registered_face_record_resource_raw &first_record
    match build_registered_face_for_table resource_raw 43:
        Result::Err _message:
            gui_font_registered_face_table_free table
            gui_font_registered_face_table_entry_free entry
            false
        Result::Ok duplicate_face:
            match gui_font_registered_face_table_register table duplicate_face:
                Result::Ok registration:
                    gui_font_registered_face_table_registration_free registration
                    gui_font_registered_face_table_entry_free entry
                    false
                Result::Err error:
                    let kind_ok %bool registered_face_table_register_error_kind_is &error GuiFontRegisteredFaceTableRegisterErrorKind::DuplicateResourceId
                    let storage_ok %bool is_none gui_font_registered_face_table_register_error_storage_error &error
                    let rejected %GuiFontRegisteredFaceTableRegisterRejected gui_font_registered_face_table_register_error_rejected error
                    let rejected_ok %bool gui_font_registered_face_table_register_rejected_with rejected @registered_face_table_duplicate_rejected_callback
                    gui_font_registered_face_table_entry_free entry
                    and kind_ok and storage_ok rejected_ok

fn registered_face_table_duplicate_recovery_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 2:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_table 31 41:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_table_duplicate_callback

fn registered_face_table_duplicate_face_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let first_record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let face_raw %i32 gui_font_registered_face_record_face_raw &first_record
    match build_registered_face_for_table 53 face_raw:
        Result::Err _message:
            gui_font_registered_face_table_free table
            gui_font_registered_face_table_entry_free entry
            false
        Result::Ok duplicate_face:
            match gui_font_registered_face_table_register table duplicate_face:
                Result::Ok registration:
                    gui_font_registered_face_table_registration_free registration
                    gui_font_registered_face_table_entry_free entry
                    false
                Result::Err error:
                    let kind_ok %bool registered_face_table_register_error_kind_is &error GuiFontRegisteredFaceTableRegisterErrorKind::DuplicateFaceId
                    let storage_ok %bool is_none gui_font_registered_face_table_register_error_storage_error &error
                    let rejected %GuiFontRegisteredFaceTableRegisterRejected gui_font_registered_face_table_register_error_rejected error
                    let rejected_ok %bool gui_font_registered_face_table_register_rejected_with rejected @registered_face_table_duplicate_rejected_callback
                    gui_font_registered_face_table_entry_free entry
                    and kind_ok and storage_ok rejected_ok

fn registered_face_table_duplicate_face_recovery_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 2:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_table 47 59:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_table_duplicate_face_callback

fn registered_face_glyph_lookup_error_kind_is %fn &GuiFontRegisteredFaceGlyphLookupError fn GuiFontRegisteredFaceGlyphLookupErrorKind bool \error\expected:
    gui_font_registered_face_glyph_lookup_error_kind_eq gui_font_registered_face_glyph_lookup_error_kind error expected

fn registered_face_glyph_lookup_parse_error_is %fn &GuiFontRegisteredFaceGlyphLookupError fn GuiSfntParseErrorKind bool \error\expected:
    match gui_font_registered_face_glyph_lookup_error_parse_error error:
        Option::None:
            false
        Option::Some parse_error:
            sfnt_parse_error_kind_is gui_sfnt_parse_error_kind &parse_error expected

fn registered_face_glyph_lookup_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let success_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            let glyph %GuiGlyphId gui_font_registered_face_glyph_mapping_glyph &mapping
            let record %GuiFontRegisteredFaceRecord gui_font_registered_face_glyph_mapping_record &mapping
            and eq 36 gui_glyph_id_raw &glyph and eq 'A' gui_font_registered_face_glyph_mapping_code_point &mapping and eq 67 gui_font_registered_face_record_resource_raw &record eq 71 gui_font_registered_face_record_face_raw &record
    let missing_ok %bool match gui_font_registered_face_glyph_lookup &entry 'B':
        Result::Ok _mapping:
            false
        Result::Err error:
            let kind_ok %bool registered_face_glyph_lookup_error_kind_is &error GuiFontRegisteredFaceGlyphLookupErrorKind::MissingGlyphMapping
            let parse_ok %bool registered_face_glyph_lookup_parse_error_is &error GuiSfntParseErrorKind::MissingGlyphMapping
            let validation_ok %bool is_none gui_font_registered_face_glyph_lookup_error_validation_error &error
            let code_point_ok %bool eq 'B' gui_font_registered_face_glyph_lookup_error_code_point &error
            let error_record %GuiFontRegisteredFaceRecord gui_font_registered_face_glyph_lookup_error_record &error
            let record_ok %bool and eq 67 gui_font_registered_face_record_resource_raw &error_record eq 71 gui_font_registered_face_record_face_raw &error_record
            and kind_ok and parse_ok and validation_ok and code_point_ok record_ok
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and success_ok missing_ok

fn registered_face_glyph_lookup_success_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_glyph_lookup 67 71:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_glyph_lookup_callback

fn registered_face_horizontal_metric_error_kind_is %fn &GuiFontRegisteredFaceHorizontalMetricLookupError fn GuiFontRegisteredFaceHorizontalMetricLookupErrorKind bool \error\expected:
    gui_font_registered_face_horizontal_metric_lookup_error_kind_eq gui_font_registered_face_horizontal_metric_lookup_error_kind error expected

fn registered_face_horizontal_metric_parse_error_is %fn &GuiFontRegisteredFaceHorizontalMetricLookupError fn GuiSfntParseErrorKind bool \error\expected:
    match gui_font_registered_face_horizontal_metric_lookup_error_parse_error error:
        Option::None:
            false
        Option::Some parse_error:
            sfnt_parse_error_kind_is gui_sfnt_parse_error_kind &parse_error expected

fn registered_face_horizontal_metric_success_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let metric_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_horizontal_metric_lookup &entry &mapping:
                Result::Err _error:
                    false
                Result::Ok evidence:
                    let evidence_mapping %GuiFontRegisteredFaceGlyphMapping gui_font_registered_face_glyph_horizontal_metric_mapping &evidence
                    let metric %GuiSfntHorizontalMetric gui_font_registered_face_glyph_horizontal_metric_metric &evidence
                    let glyph %GuiGlyphId gui_font_registered_face_glyph_mapping_glyph &evidence_mapping
                    let record %GuiFontRegisteredFaceRecord gui_font_registered_face_glyph_mapping_record &evidence_mapping
                    and eq 'A' gui_font_registered_face_glyph_mapping_code_point &evidence_mapping and eq 36 gui_glyph_id_raw &glyph and eq 600 gui_sfnt_horizontal_metric_advance_width &metric and eq 20 gui_sfnt_horizontal_metric_left_side_bearing &metric and eq 101 gui_font_registered_face_record_resource_raw &record eq 103 gui_font_registered_face_record_face_raw &record
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok record:
            and eq 101 gui_font_registered_face_record_resource_raw &record eq 103 gui_font_registered_face_record_face_raw &record
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and metric_ok entry_reusable

fn registered_face_horizontal_metric_success_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_horizontal_metric 101 103 82:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_horizontal_metric_success_callback

fn registered_face_horizontal_metric_missing_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let missing_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_horizontal_metric_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_horizontal_metric_error_kind_is &error GuiFontRegisteredFaceHorizontalMetricLookupErrorKind::SfntParseFailed registered_face_horizontal_metric_parse_error_is &error GuiSfntParseErrorKind::MissingTable
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok _record:
            true
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and missing_ok entry_reusable

fn registered_face_horizontal_metric_missing_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_glyph_lookup 107 109:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_horizontal_metric_missing_callback

fn registered_face_horizontal_metric_malformed_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let malformed_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_horizontal_metric_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_horizontal_metric_error_kind_is &error GuiFontRegisteredFaceHorizontalMetricLookupErrorKind::SfntParseFailed registered_face_horizontal_metric_parse_error_is &error GuiSfntParseErrorKind::MalformedHmtxRecord
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok _record:
            true
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and malformed_ok entry_reusable

fn registered_face_horizontal_metric_malformed_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_horizontal_metric 113 127 80:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_horizontal_metric_malformed_callback

fn registered_face_foreign_mapping_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry Option GuiFontRegisteredFaceGlyphMapping \table\entry:
    let mapping %Option GuiFontRegisteredFaceGlyphMapping match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            none
        Result::Ok value:
            some value
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    mapping

fn registered_face_foreign_mapping %impure fn void Option GuiFontRegisteredFaceGlyphMapping \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            none
        Result::Ok table:
            match build_registered_face_for_glyph_lookup 131 137:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    none
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            none
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_foreign_mapping_callback

fn registered_face_horizontal_metric_mismatch_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let mismatch_ok %bool match registered_face_foreign_mapping:
        Option::None:
            false
        Option::Some mapping:
            match gui_font_registered_face_horizontal_metric_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_horizontal_metric_error_kind_is &error GuiFontRegisteredFaceHorizontalMetricLookupErrorKind::MappingRecordMismatch is_none gui_font_registered_face_horizontal_metric_lookup_error_parse_error &error
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok record:
            and eq 139 gui_font_registered_face_record_resource_raw &record eq 149 gui_font_registered_face_record_face_raw &record
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and mismatch_ok entry_reusable

fn registered_face_horizontal_metric_mismatch_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_horizontal_metric 139 149 82:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_horizontal_metric_mismatch_callback

fn registered_face_simple_glyph_error_kind_is %fn &GuiFontRegisteredFaceSimpleGlyphLookupError fn GuiFontRegisteredFaceSimpleGlyphLookupErrorKind bool \error\expected:
    gui_font_registered_face_simple_glyph_lookup_error_kind_eq gui_font_registered_face_simple_glyph_lookup_error_kind error expected

fn registered_face_simple_glyph_parse_error_is %fn &GuiFontRegisteredFaceSimpleGlyphLookupError fn GuiSfntParseErrorKind bool \error\expected:
    match gui_font_registered_face_simple_glyph_lookup_error_parse_error error:
        Option::None:
            false
        Option::Some parse_error:
            sfnt_parse_error_kind_is gui_sfnt_parse_error_kind &parse_error expected

fn registered_face_simple_glyph_reader_points_ok %fn &GuiFontRegisteredFaceTableEntry fn GuiFontRegisteredFaceSimpleGlyphSequentialReaderCursor fn i32 bool \entry\cursor\expected_index:
    match gui_font_registered_face_simple_glyph_sequential_reader_step entry cursor:
        Result::Err _error:
            false
        Result::Ok terminal:
            if:
                eq expected_index 4
                then:
                    match terminal:
                        GuiFontRegisteredFaceSimpleGlyphSequentialReaderTerminal::Point _step:
                            false
                        GuiFontRegisteredFaceSimpleGlyphSequentialReaderTerminal::End end_cursor:
                            let lower %GuiSfntSimpleGlyphSequentialPointCursor gui_font_registered_face_simple_glyph_sequential_reader_cursor_lower &end_cursor
                            if:
                                ne gui_sfnt_simple_glyph_sequential_point_cursor_logical_index &lower 4
                                then:
                                    false
                                else:
                                    match gui_font_registered_face_simple_glyph_sequential_reader_step entry end_cursor:
                                        Result::Err _error:
                                            false
                                        Result::Ok repeated_terminal:
                                            match repeated_terminal:
                                                GuiFontRegisteredFaceSimpleGlyphSequentialReaderTerminal::Point _step:
                                                    false
                                                GuiFontRegisteredFaceSimpleGlyphSequentialReaderTerminal::End repeated_cursor:
                                                    let repeated_lower %GuiSfntSimpleGlyphSequentialPointCursor gui_font_registered_face_simple_glyph_sequential_reader_cursor_lower &repeated_cursor
                                                    eq gui_sfnt_simple_glyph_sequential_point_cursor_logical_index &repeated_lower 4
                else:
                    match terminal:
                        GuiFontRegisteredFaceSimpleGlyphSequentialReaderTerminal::End _end_cursor:
                            false
                        GuiFontRegisteredFaceSimpleGlyphSequentialReaderTerminal::Point step:
                            let point %GuiSfntSimpleGlyphPoint gui_font_registered_face_simple_glyph_sequential_reader_step_point &step
                            let next_cursor %GuiFontRegisteredFaceSimpleGlyphSequentialReaderCursor gui_font_registered_face_simple_glyph_sequential_reader_step_next_cursor &step
                            let expected_end %bool or eq expected_index 1 eq expected_index 3
                            let actual_end %bool gui_sfnt_simple_glyph_point_end_of_contour &point
                            let end_ok %bool if expected_end actual_end else not actual_end
                            and eq gui_sfnt_simple_glyph_point_index &point expected_index and eq gui_sfnt_simple_glyph_point_x &point 0 and eq gui_sfnt_simple_glyph_point_y &point 0 and not gui_sfnt_simple_glyph_point_on_curve &point and end_ok registered_face_simple_glyph_reader_points_ok entry next_cursor add expected_index 1

fn registered_face_simple_glyph_reader_ok %fn &GuiFontRegisteredFaceTableEntry fn &GuiFontRegisteredFaceSimpleGlyphPointStream bool \entry\evidence:
    match gui_font_registered_face_simple_glyph_sequential_reader_start entry evidence:
        Result::Err _error:
            false
        Result::Ok cursor:
            registered_face_simple_glyph_reader_points_ok entry cursor 0

fn registered_face_simple_glyph_collection_item_ok %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollection fn i32 bool \collection\index:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item collection index:
        Result::Err _error:
            false
        Result::Ok item:
            let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
            let kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
            let kind_ok %bool if:
                or eq index 1 eq index 3
                then gui_sfnt_simple_glyph_outline_point_stream_item_kind_is kind GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
                else gui_sfnt_simple_glyph_outline_point_stream_item_kind_is kind GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve
            and eq gui_sfnt_simple_glyph_point_index &point index kind_ok

fn registered_face_simple_glyph_collection_spans_ok %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollection bool \collection:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span collection 0:
        Result::Err _error:
            false
        Result::Ok first:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span collection 1:
                Result::Err _error:
                    false
                Result::Ok second:
                    and eq gui_sfnt_simple_glyph_contour_span_start_point_index &first 0 and eq gui_sfnt_simple_glyph_contour_span_end_point_index &first 1 and eq gui_sfnt_simple_glyph_contour_span_point_count &first 2 and eq gui_sfnt_simple_glyph_contour_span_start_point_index &second 2 and eq gui_sfnt_simple_glyph_contour_span_end_point_index &second 3 eq gui_sfnt_simple_glyph_contour_span_point_count &second 2

fn registered_face_simple_glyph_indexed_path_drain_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner impure fn i32 impure fn i32 bool \owner\emitted_count\close_count:
    match gui_font_registered_face_simple_glyph_indexed_path_drain_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_path_error_free error
            false
        Result::Ok budget_step:
            let status %GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus gui_font_registered_face_simple_glyph_indexed_path_budget_step_status &budget_step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::StepBudgetExhausted:
                    gui_font_registered_face_simple_glyph_indexed_path_budget_step_free budget_step
                    false
                GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::Completed:
                    let completed %GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner gui_font_registered_face_simple_glyph_indexed_path_budget_step_take_owner budget_step
                    let progress_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_progress_kind &completed:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathProgressKind::Completed:
                            true
                        _:
                            false
                    gui_font_registered_face_simple_glyph_indexed_path_owner_free completed
                    and progress_ok and eq emitted_count 8 eq close_count 2
                GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::Emitted sink_step:
                    let primary %GuiSfntSimpleGlyphPathSinkPrimaryAction gui_sfnt_simple_glyph_path_sink_step_primary_action &sink_step
                    let tail %GuiSfntSimpleGlyphPathSinkTailAction gui_sfnt_simple_glyph_path_sink_step_tail_action &sink_step
                    let primary_ok %bool match primary:
                        GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent _event:
                            true
                        GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject _reason:
                            false
                    let next_close_count %i32 match tail:
                        GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction:
                            close_count
                        GuiSfntSimpleGlyphPathSinkTailAction::CloseContour _close:
                            add close_count 1
                    let next_owner %GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner gui_font_registered_face_simple_glyph_indexed_path_budget_step_take_owner budget_step
                    let next_emitted_count %i32 add emitted_count 1
                    let completed_now %bool match gui_font_registered_face_simple_glyph_indexed_path_progress_kind &next_owner:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathProgressKind::Completed:
                            true
                        _:
                            false
                    if completed_now:
                        then:
                            match gui_font_registered_face_simple_glyph_indexed_path_drain_budget next_owner 0:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_indexed_path_error_free error
                                    false
                                Result::Ok completed_step:
                                    let completed_status %GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus gui_font_registered_face_simple_glyph_indexed_path_budget_step_status &completed_step
                                    let status_ok %bool match completed_status:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::Completed:
                                            true
                                        _:
                                            false
                                    let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner gui_font_registered_face_simple_glyph_indexed_path_budget_step_take_owner completed_step
                                    match gui_font_registered_face_simple_glyph_indexed_path_step completed_owner:
                                        Result::Ok unexpected_step:
                                            gui_font_registered_face_simple_glyph_indexed_path_step_free unexpected_step
                                            false
                                        Result::Err completed_error:
                                            let direct_step_rejected %bool match gui_font_registered_face_simple_glyph_indexed_path_error_kind &completed_error:
                                                GuiFontRegisteredFaceSimpleGlyphIndexedPathErrorKind::AlreadyCompleted:
                                                    true
                                                _:
                                                    false
                                            gui_font_registered_face_simple_glyph_indexed_path_error_free completed_error
                                            and primary_ok and status_ok and direct_step_rejected and eq next_emitted_count 8 eq next_close_count 2
                        else:
                            and primary_ok registered_face_simple_glyph_indexed_path_drain_ok next_owner next_emitted_count next_close_count

fn registered_face_simple_glyph_indexed_path_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedOwner bool \owner:
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let path %GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner gui_font_registered_face_simple_glyph_indexed_path_start owner &policy
    match gui_font_registered_face_simple_glyph_indexed_path_drain_budget path 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_path_error_free error
            false
        Result::Ok budget_step:
            let status %GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus gui_font_registered_face_simple_glyph_indexed_path_budget_step_status &budget_step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::StepBudgetExhausted:
                    let exhausted %GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner gui_font_registered_face_simple_glyph_indexed_path_budget_step_take_owner budget_step
                    let pending_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_progress_kind &exhausted:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathProgressKind::PendingContour:
                            true
                        _:
                            false
                    and pending_ok registered_face_simple_glyph_indexed_path_drain_ok exhausted 0 0
                GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::Completed:
                    let completed %GuiFontRegisteredFaceSimpleGlyphIndexedPathOwner gui_font_registered_face_simple_glyph_indexed_path_budget_step_take_owner budget_step
                    gui_font_registered_face_simple_glyph_indexed_path_owner_free completed
                    false
                GuiFontRegisteredFaceSimpleGlyphIndexedPathBudgetStatus::Emitted _sink_step:
                    gui_font_registered_face_simple_glyph_indexed_path_budget_step_free budget_step
                    false

fn registered_face_simple_glyph_indexed_action_sink_step_source_same %fn &GuiSfntSimpleGlyphPathSinkStep fn &GuiSfntSimpleGlyphPathSinkStep bool \primary\tail:
    let primary_source %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_path_sink_step_source_step primary
    let tail_source %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_path_sink_step_source_step tail
    let primary_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_step_cursor &primary_source
    let tail_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_step_cursor &tail_source
    let primary_glyph %GuiGlyphId gui_sfnt_simple_glyph_path_contour_cursor_glyph &primary_cursor
    let tail_glyph %GuiGlyphId gui_sfnt_simple_glyph_path_contour_cursor_glyph &tail_cursor
    let slot_same %bool match gui_sfnt_simple_glyph_path_contour_cursor_slot &primary_cursor:
        GuiSfntSimpleGlyphPathSinkEventSlot::First:
            match gui_sfnt_simple_glyph_path_contour_cursor_slot &tail_cursor:
                GuiSfntSimpleGlyphPathSinkEventSlot::First:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPathSinkEventSlot::Second:
            match gui_sfnt_simple_glyph_path_contour_cursor_slot &tail_cursor:
                GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                    true
                _:
                    false
    and eq gui_glyph_id_raw &primary_glyph gui_glyph_id_raw &tail_glyph and eq gui_sfnt_simple_glyph_path_contour_cursor_contour_index &primary_cursor gui_sfnt_simple_glyph_path_contour_cursor_contour_index &tail_cursor and eq gui_sfnt_simple_glyph_path_contour_cursor_edge_index &primary_cursor gui_sfnt_simple_glyph_path_contour_cursor_edge_index &tail_cursor slot_same

fn registered_face_simple_glyph_indexed_action_drain_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner impure fn i32 impure fn i32 impure fn i32 impure fn bool impure fn Option GuiSfntSimpleGlyphPathSinkStep bool \owner\action_count\close_count\no_action_count\expect_primary\previous_primary:
    match gui_font_registered_face_simple_glyph_indexed_action_drain_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_error_free error
            false
        Result::Ok budget_step:
            let status %GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_budget_step_status &budget_step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::StepBudgetExhausted:
                    gui_font_registered_face_simple_glyph_indexed_action_budget_step_free budget_step
                    false
                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::Completed:
                    let completed %GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner gui_font_registered_face_simple_glyph_indexed_action_budget_step_take_owner budget_step
                    match gui_font_registered_face_simple_glyph_indexed_action_drain_budget completed 0:
                        Result::Err completed_budget_error:
                            gui_font_registered_face_simple_glyph_indexed_action_error_free completed_budget_error
                            false
                        Result::Ok completed_budget_step:
                            let completed_budget_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_budget_step_status &completed_budget_step
                            let completed_budget_ok %bool match completed_budget_status:
                                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::Completed:
                                    true
                                _:
                                    false
                            let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner gui_font_registered_face_simple_glyph_indexed_action_budget_step_take_owner completed_budget_step
                            match gui_font_registered_face_simple_glyph_indexed_action_step completed_owner:
                                Result::Ok unexpected_step:
                                    gui_font_registered_face_simple_glyph_indexed_action_step_free unexpected_step
                                    false
                                Result::Err completed_error:
                                    let direct_step_rejected %bool match gui_font_registered_face_simple_glyph_indexed_action_error_kind &completed_error:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedActionErrorKind::AlreadyCompleted:
                                            true
                                        _:
                                            false
                                    gui_font_registered_face_simple_glyph_indexed_action_error_free completed_error
                                    let pair_complete %bool match previous_primary:
                                        Option::None:
                                            true
                                        Option::Some _step:
                                            false
                                    and completed_budget_ok and direct_step_rejected and expect_primary and pair_complete and eq action_count 16 and eq close_count 2 eq no_action_count 6
                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::Emitted packet:
                    let slot %GuiSfntSimpleGlyphPathSinkActionSlot gui_font_registered_face_simple_glyph_indexed_action_packet_slot &packet
                    let action %GuiSfntSimpleGlyphPathSinkAction gui_font_registered_face_simple_glyph_indexed_action_packet_action &packet
                    let sink_step %GuiSfntSimpleGlyphPathSinkStep gui_font_registered_face_simple_glyph_indexed_action_packet_sink_step &packet
                    let slot_ok %bool if expect_primary:
                        then:
                            gui_sfnt_simple_glyph_path_sink_action_slot_is_primary slot
                        else:
                            gui_sfnt_simple_glyph_path_sink_action_slot_is_tail slot
                    let action_ok %bool if expect_primary:
                        then:
                            match action:
                                GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
                                    true
                                _:
                                    false
                        else:
                            match action:
                                GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
                                    true
                                GuiSfntSimpleGlyphPathSinkAction::NoAction:
                                    true
                                _:
                                    false
                    let source_pair_ok %bool if expect_primary:
                        then:
                            match previous_primary:
                                Option::None:
                                    true
                                Option::Some _step:
                                    false
                        else:
                            match previous_primary:
                                Option::None:
                                    false
                                Option::Some primary_step:
                                    registered_face_simple_glyph_indexed_action_sink_step_source_same &primary_step &sink_step
                    let next_close_count %i32 match action:
                        GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
                            add close_count 1
                        _:
                            close_count
                    let next_no_action_count %i32 match action:
                        GuiSfntSimpleGlyphPathSinkAction::NoAction:
                            add no_action_count 1
                        _:
                            no_action_count
                    let next_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner gui_font_registered_face_simple_glyph_indexed_action_budget_step_take_owner budget_step
                    let next_action_count %i32 add action_count 1
                    if expect_primary:
                        then:
                            match gui_font_registered_face_simple_glyph_indexed_action_drain_budget next_owner 0:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_indexed_action_error_free error
                                    false
                                Result::Ok exhausted_step:
                                    let exhausted_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_budget_step_status &exhausted_step
                                    let exhausted_ok %bool match exhausted_status:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::StepBudgetExhausted:
                                            true
                                        _:
                                            false
                                    let exhausted_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner gui_font_registered_face_simple_glyph_indexed_action_budget_step_take_owner exhausted_step
                                    and slot_ok and action_ok and source_pair_ok and exhausted_ok registered_face_simple_glyph_indexed_action_drain_ok exhausted_owner next_action_count next_close_count next_no_action_count false some sink_step
                        else:
                            and slot_ok and action_ok and source_pair_ok registered_face_simple_glyph_indexed_action_drain_ok next_owner next_action_count next_close_count next_no_action_count true none

fn registered_face_simple_glyph_indexed_action_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedOwner bool \owner:
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let action %GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner gui_font_registered_face_simple_glyph_indexed_action_start owner &policy
    match gui_font_registered_face_simple_glyph_indexed_action_drain_budget action 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_error_free error
            false
        Result::Ok budget_step:
            let status %GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_budget_step_status &budget_step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::StepBudgetExhausted:
                    let exhausted %GuiFontRegisteredFaceSimpleGlyphIndexedActionOwner gui_font_registered_face_simple_glyph_indexed_action_budget_step_take_owner budget_step
                    registered_face_simple_glyph_indexed_action_drain_ok exhausted 0 0 0 true none
                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::Completed:
                    gui_font_registered_face_simple_glyph_indexed_action_budget_step_free budget_step
                    false
                GuiFontRegisteredFaceSimpleGlyphIndexedActionBudgetStatus::Emitted _packet:
                    gui_font_registered_face_simple_glyph_indexed_action_budget_step_free budget_step
                    false

enum RegisteredFaceSimpleGlyphTraversalMode:
    IndexedPath
    IndexedAction
    IndexedActionSummaryEmitClose
    IndexedActionSummaryKeepOpen
    IndexedActionSummaryRejectUnsupported
    IndexedContourEndpointSuccess
    IndexedContourEndpointPrematureSeal
    IndexedContourEndpointLookupFailure
    IndexedContourEndpointPushFailure

impl Clone for RegisteredFaceSimpleGlyphTraversalMode:
    fn clone %fn &RegisteredFaceSimpleGlyphTraversalMode RegisteredFaceSimpleGlyphTraversalMode \mode:
        *mode

impl Copy for RegisteredFaceSimpleGlyphTraversalMode:
    fn copy_mark %fn RegisteredFaceSimpleGlyphTraversalMode RegisteredFaceSimpleGlyphTraversalMode \mode:
        mode

fn registered_face_simple_glyph_capacity_same %fn &GuiSfntSimpleGlyphOutlineStorageCapacity fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \expected\actual:
    let expected_glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph expected
    let actual_glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph actual
    and eq gui_glyph_id_raw &expected_glyph gui_glyph_id_raw &actual_glyph and eq gui_sfnt_simple_glyph_outline_storage_capacity_contour_count expected gui_sfnt_simple_glyph_outline_storage_capacity_contour_count actual and eq gui_sfnt_simple_glyph_outline_storage_capacity_point_count expected gui_sfnt_simple_glyph_outline_storage_capacity_point_count actual and eq gui_sfnt_simple_glyph_outline_storage_capacity_edge_count expected gui_sfnt_simple_glyph_outline_storage_capacity_edge_count actual and eq gui_sfnt_simple_glyph_outline_storage_capacity_path_command_pair_count expected gui_sfnt_simple_glyph_outline_storage_capacity_path_command_pair_count actual eq gui_sfnt_simple_glyph_outline_storage_capacity_path_command_count expected gui_sfnt_simple_glyph_outline_storage_capacity_path_command_count actual

fn registered_face_simple_glyph_summary_counts_ok %fn &GuiSfntSimpleGlyphPathSinkActionApplyState fn i32 fn i32 fn i32 fn i32 bool \state\emitted\rejected\closed\no_action:
    and eq emitted gui_sfnt_simple_glyph_path_sink_action_apply_state_emitted_event_count state and eq rejected gui_sfnt_simple_glyph_path_sink_action_apply_state_reject_count state and eq closed gui_sfnt_simple_glyph_path_sink_action_apply_state_close_contour_count state eq no_action gui_sfnt_simple_glyph_path_sink_action_apply_state_no_action_count state

enum RegisteredFaceSimpleGlyphOutlineLimitFailureCase:
    ContourPositiveLow
    ContourZero
    ContourNegative
    PointPositiveLow
    PointZero
    PointNegative
    EdgePositiveLow
    EdgeZero
    EdgeNegative
    CommandPositiveLow
    CommandZero
    CommandNegative

impl Clone for RegisteredFaceSimpleGlyphOutlineLimitFailureCase:
    fn clone %fn &RegisteredFaceSimpleGlyphOutlineLimitFailureCase RegisteredFaceSimpleGlyphOutlineLimitFailureCase \value:
        *value

impl Copy for RegisteredFaceSimpleGlyphOutlineLimitFailureCase:
    fn copy_mark %fn RegisteredFaceSimpleGlyphOutlineLimitFailureCase RegisteredFaceSimpleGlyphOutlineLimitFailureCase \value:
        value

fn registered_face_simple_glyph_outline_limit_for_case %fn RegisteredFaceSimpleGlyphOutlineLimitFailureCase GuiSfntSimpleGlyphOutlineStorageLimit \case:
    match case:
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourPositiveLow: gui_sfnt_simple_glyph_outline_storage_limit 1 91 0 sub 0 7
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourZero: gui_sfnt_simple_glyph_outline_storage_limit 0 sub 0 11 3 99
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourNegative: gui_sfnt_simple_glyph_outline_storage_limit sub 0 1 0 sub 0 2 1
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointPositiveLow: gui_sfnt_simple_glyph_outline_storage_limit 2 3 71 sub 0 5
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointZero: gui_sfnt_simple_glyph_outline_storage_limit 2 0 sub 0 9 101
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointNegative: gui_sfnt_simple_glyph_outline_storage_limit 2 sub 0 1 0 2
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgePositiveLow: gui_sfnt_simple_glyph_outline_storage_limit 2 4 3 77
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeZero: gui_sfnt_simple_glyph_outline_storage_limit 2 4 0 sub 0 13
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeNegative: gui_sfnt_simple_glyph_outline_storage_limit 2 4 sub 0 1 0
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandPositiveLow: gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 7
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandZero: gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 0
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandNegative: gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 sub 0 1

fn registered_face_simple_glyph_outline_failure_reason_ok %fn RegisteredFaceSimpleGlyphOutlineLimitFailureCase fn GuiSfntSimpleGlyphOutlineCapacityRejectReason bool \case\reason:
    match case:
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourPositiveLow:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourZero:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourNegative:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointPositiveLow:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::PointCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointZero:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::PointCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointNegative:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::PointCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgePositiveLow:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::EdgeCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeZero:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::EdgeCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeNegative:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::EdgeCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandPositiveLow:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::CommandCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandZero:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::CommandCapacityExceeded: true
                _: false
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandNegative:
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::CommandCapacityExceeded: true
                _: false

fn registered_face_simple_glyph_outline_next_failure_case %fn RegisteredFaceSimpleGlyphOutlineLimitFailureCase Option RegisteredFaceSimpleGlyphOutlineLimitFailureCase \case:
    match case:
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourPositiveLow: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourZero
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourZero: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourNegative
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourNegative: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointPositiveLow
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointPositiveLow: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointZero
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointZero: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointNegative
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::PointNegative: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgePositiveLow
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgePositiveLow: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeZero
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeZero: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeNegative
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::EdgeNegative: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandPositiveLow
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandPositiveLow: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandZero
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandZero: some RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandNegative
        RegisteredFaceSimpleGlyphOutlineLimitFailureCase::CommandNegative: none

fn registered_face_simple_glyph_outline_limit_same %fn &GuiSfntSimpleGlyphOutlineStorageLimit fn &GuiSfntSimpleGlyphOutlineStorageLimit bool \expected\actual:
    and eq gui_sfnt_simple_glyph_outline_storage_limit_max_contours expected gui_sfnt_simple_glyph_outline_storage_limit_max_contours actual and eq gui_sfnt_simple_glyph_outline_storage_limit_max_points expected gui_sfnt_simple_glyph_outline_storage_limit_max_points actual and eq gui_sfnt_simple_glyph_outline_storage_limit_max_edges expected gui_sfnt_simple_glyph_outline_storage_limit_max_edges actual eq gui_sfnt_simple_glyph_outline_storage_limit_max_path_commands expected gui_sfnt_simple_glyph_outline_storage_limit_max_path_commands actual

fn registered_face_simple_glyph_outline_capacity_fixture_ok %fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \capacity:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    and eq 36 gui_glyph_id_raw &glyph and eq 2 gui_sfnt_simple_glyph_outline_storage_capacity_contour_count capacity and eq 4 gui_sfnt_simple_glyph_outline_storage_capacity_point_count capacity and eq 4 gui_sfnt_simple_glyph_outline_storage_capacity_edge_count capacity and eq 4 gui_sfnt_simple_glyph_outline_storage_capacity_path_command_pair_count capacity eq 8 gui_sfnt_simple_glyph_outline_storage_capacity_path_command_count capacity

fn registered_face_simple_glyph_outline_rejected_check_ok %fn &Option GuiSfntSimpleGlyphOutlineCapacityCheck fn RegisteredFaceSimpleGlyphOutlineLimitFailureCase fn &GuiSfntSimpleGlyphOutlineStorageCapacity fn &GuiSfntSimpleGlyphOutlineStorageLimit bool \check\case\capacity\limit:
    match *check:
        Option::None: false
        Option::Some lower_check:
            match lower_check:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected rejected:
                    let reason %GuiSfntSimpleGlyphOutlineCapacityRejectReason gui_sfnt_simple_glyph_outline_capacity_rejected_reason &rejected
                    let rejected_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_capacity_rejected_capacity &rejected
                    let rejected_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_capacity_rejected_limit &rejected
                    and registered_face_simple_glyph_outline_failure_reason_ok case reason and registered_face_simple_glyph_capacity_same capacity &rejected_capacity registered_face_simple_glyph_outline_limit_same limit &rejected_limit
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits _capacity: false
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology: false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology: false

fn registered_face_simple_glyph_outline_success_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner bool \completed:
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_font_registered_face_simple_glyph_indexed_outline_storage_start completed &limit:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_free error
            false
        Result::Ok owner:
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_capacity &owner
            let storage_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_storage_capacity &owner
            let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_apply_state &owner
            let last_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_last_status &owner
            let last_ok %bool match last_status:
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close: true
                _: false
            let invariant_ok %bool match gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_invariant_check &owner:
                GuiFontRegisteredFaceSimpleGlyphIndexedOutlineStorageInvariantCheck::Valid: true
                GuiFontRegisteredFaceSimpleGlyphIndexedOutlineStorageInvariantCheck::Invalid _kind: false
            let capacity_ok %bool and registered_face_simple_glyph_outline_capacity_fixture_ok &capacity registered_face_simple_glyph_capacity_same &capacity &storage_capacity
            let storage_ok %bool and eq 22 gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_scalar_slot_count &owner and eq 0 gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_scalar_slots_len &owner eq 22 gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_scalar_slots_cap &owner
            gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_free owner
            and capacity_ok and registered_face_simple_glyph_summary_counts_ok &state 8 0 2 6 and last_ok and storage_ok invariant_ok

fn registered_face_simple_glyph_outline_forced_alloc_failure_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner bool \completed:
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_font_registered_face_simple_glyph_indexed_outline_storage_test_force_scalar_slot_storage_alloc_failed completed:
        Result::Ok owner:
            gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_free owner
            false
        Result::Err error:
            let kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_kind &error:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotStorageAllocFailed: true
                _: false
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_capacity &error
            let supplied_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_limit &error
            let check %Option GuiSfntSimpleGlyphOutlineCapacityCheck gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_capacity_check &error
            let check_ok %bool match check:
                Option::Some lower_check:
                    match lower_check:
                        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits checked_capacity: registered_face_simple_glyph_capacity_same &capacity &checked_capacity
                        _: false
                Option::None: false
            let metadata_ok %bool and kind_ok and registered_face_simple_glyph_outline_capacity_fixture_ok &capacity and registered_face_simple_glyph_outline_limit_same &limit &supplied_limit check_ok
            let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_take_completed_owner error
            and metadata_ok registered_face_simple_glyph_outline_success_ok recovered

fn registered_face_simple_glyph_outline_limit_failures_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner impure fn RegisteredFaceSimpleGlyphOutlineLimitFailureCase bool \completed\case:
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit registered_face_simple_glyph_outline_limit_for_case case
    match gui_font_registered_face_simple_glyph_indexed_outline_storage_start completed &limit:
        Result::Ok owner:
            gui_font_registered_face_simple_glyph_indexed_outline_storage_owner_free owner
            false
        Result::Err error:
            let kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_kind &error:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected: true
                _: false
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_capacity &error
            let supplied_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_limit &error
            let check %Option GuiSfntSimpleGlyphOutlineCapacityCheck gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_capacity_check &error
            let metadata_ok %bool and kind_ok and registered_face_simple_glyph_outline_capacity_fixture_ok &capacity and registered_face_simple_glyph_outline_limit_same &limit &supplied_limit registered_face_simple_glyph_outline_rejected_check_ok &check case &capacity &limit
            let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_take_completed_owner error
            if not metadata_ok:
                then:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_free recovered
                    false
                else:
                    match registered_face_simple_glyph_outline_next_failure_case case:
                        Option::Some next_case: registered_face_simple_glyph_outline_limit_failures_ok recovered next_case
                        Option::None: registered_face_simple_glyph_outline_forced_alloc_failure_ok recovered

fn registered_face_simple_glyph_summary_running_seals_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner Result GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner unit \owner:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_completed owner:
        Result::Ok completed:
            gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_free completed
            Result::Err unit
        Result::Err completed_error:
            let completed_target_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_target &completed_error:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummarySealTargetKind::Completed: true
                _: false
            let completed_actual_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_actual &completed_error:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Running: true
                _: false
            let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_take_owner completed_error
            match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_rejected recovered:
                Result::Ok rejected:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_free rejected
                    Result::Err unit
                Result::Err rejected_error:
                    let rejected_target_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_target &rejected_error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummarySealTargetKind::Rejected: true
                        _: false
                    let rejected_actual_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_actual &rejected_error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Running: true
                        _: false
                    let next_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_take_owner rejected_error
                    if and completed_target_ok and completed_actual_ok and rejected_target_ok rejected_actual_ok:
                        then Result::Ok next_owner
                        else:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                            Result::Err unit

fn registered_face_simple_glyph_summary_running_nonpositive_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner Result GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner unit \owner:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget owner 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
            Result::Err unit
        Result::Ok zero_step:
            let zero_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &zero_step
            let zero_ok %bool match zero_status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::StepBudgetExhausted: true
                _: false
            let zero_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner zero_step
            match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget zero_owner sub 0 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
                    Result::Err unit
                Result::Ok negative_step:
                    let negative_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &negative_step
                    let negative_ok %bool match negative_status:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::StepBudgetExhausted: true
                        _: false
                    let negative_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner negative_step
                    if and zero_ok negative_ok:
                        then Result::Ok negative_owner
                        else:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free negative_owner
                            Result::Err unit

fn registered_face_simple_glyph_summary_completed_nonpositive_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner Result GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner unit \owner:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget owner 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
            Result::Err unit
        Result::Ok zero_step:
            let zero_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &zero_step
            let zero_ok %bool match zero_status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Completed: true
                _: false
            let zero_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner zero_step
            match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget zero_owner sub 0 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
                    Result::Err unit
                Result::Ok negative_step:
                    let negative_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &negative_step
                    let negative_ok %bool match negative_status:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Completed: true
                        _: false
                    let negative_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner negative_step
                    if and zero_ok negative_ok:
                        then Result::Ok negative_owner
                        else:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free negative_owner
                            Result::Err unit

fn registered_face_simple_glyph_summary_rejected_nonpositive_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner Result GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner unit \owner:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget owner 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
            Result::Err unit
        Result::Ok zero_step:
            let zero_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &zero_step
            let zero_ok %bool match zero_status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Rejected reason:
                    match reason:
                        GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart: true
                _: false
            let zero_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner zero_step
            match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget zero_owner sub 0 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
                    Result::Err unit
                Result::Ok negative_step:
                    let negative_status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &negative_step
                    let negative_ok %bool match negative_status:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Rejected reason:
                            match reason:
                                GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart: true
                        _: false
                    let negative_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner negative_step
                    if and zero_ok negative_ok:
                        then Result::Ok negative_owner
                        else:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free negative_owner
                            Result::Err unit

fn registered_face_simple_glyph_summary_completed_open_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \owner\expected_capacity:
    match registered_face_simple_glyph_summary_completed_nonpositive_ok owner:
        Result::Err _unit:
            false
        Result::Ok terminal_owner:
            match gui_font_registered_face_simple_glyph_indexed_action_summary_step terminal_owner:
                Result::Ok unexpected_step:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_step_free unexpected_step
                    false
                Result::Err completed_error:
                    let direct_step_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_error_kind &completed_error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryErrorKind::AlreadyCompleted: true
                        _: false
                    let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_error_take_owner completed_error
                    match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_rejected recovered:
                        Result::Ok unexpected_rejected:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_free unexpected_rejected
                            false
                        Result::Err seal_error:
                            let target_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_target &seal_error:
                                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummarySealTargetKind::Rejected: true
                                _: false
                            let actual_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_actual &seal_error:
                                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Completed: true
                                _: false
                            let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_take_owner seal_error
                            match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_completed completed_owner:
                                Result::Err final_error:
                                    gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_free final_error
                                    false
                                Result::Ok completed:
                                    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_capacity &completed
                                    let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_apply_state &completed
                                    let last_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_last_status &completed
                                    let last_ok %bool match last_status:
                                        GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction: true
                                        _: false
                                    gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_free completed
                                    and direct_step_ok and target_ok and actual_ok and registered_face_simple_glyph_capacity_same expected_capacity &capacity and registered_face_simple_glyph_summary_counts_ok &state 8 0 0 8 last_ok

fn registered_face_simple_glyph_summary_completed_close_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \owner\expected_capacity:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_completed owner:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_free error
            false
        Result::Ok completed:
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_capacity &completed
            let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_apply_state &completed
            let last_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_last_status &completed
            let last_ok %bool match last_status:
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close: true
                _: false
            let summary_ok %bool and registered_face_simple_glyph_capacity_same expected_capacity &capacity and registered_face_simple_glyph_summary_counts_ok &state 8 0 2 6 last_ok
            if summary_ok:
                then registered_face_simple_glyph_outline_limit_failures_ok completed RegisteredFaceSimpleGlyphOutlineLimitFailureCase::ContourPositiveLow
                else:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_free completed
                    false

fn registered_face_simple_glyph_summary_rejected_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \owner\expected_capacity:
    match registered_face_simple_glyph_summary_rejected_nonpositive_ok owner:
        Result::Err _unit:
            false
        Result::Ok terminal_owner:
            match gui_font_registered_face_simple_glyph_indexed_action_summary_step terminal_owner:
                Result::Ok unexpected_step:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_step_free unexpected_step
                    false
                Result::Err rejected_error:
                    let direct_step_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_error_kind &rejected_error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryErrorKind::AlreadyRejected: true
                        _: false
                    let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_error_take_owner rejected_error
                    match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_completed recovered:
                        Result::Ok unexpected_completed:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_free unexpected_completed
                            false
                        Result::Err seal_error:
                            let target_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_target &seal_error:
                                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummarySealTargetKind::Completed: true
                                _: false
                            let actual_ok %bool match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_actual &seal_error:
                                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Rejected: true
                                _: false
                            let rejected_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_take_owner seal_error
                            match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_rejected rejected_owner:
                                Result::Err final_error:
                                    gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_free final_error
                                    false
                                Result::Ok rejected:
                                    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_capacity &rejected
                                    let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_apply_state &rejected
                                    let last_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_last_status &rejected
                                    let reason %GuiSfntSimpleGlyphPathSinkRejectReason gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_reason &rejected
                                    let last_ok %bool match last_status:
                                        GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected last_reason:
                                            match last_reason:
                                                GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart: true
                                        _: false
                                    let reason_ok %bool match reason:
                                        GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart: true
                                    gui_font_registered_face_simple_glyph_indexed_action_summary_rejected_owner_free rejected
                                    and direct_step_ok and target_ok and actual_ok and registered_face_simple_glyph_capacity_same expected_capacity &capacity and registered_face_simple_glyph_summary_counts_ok &state 0 1 0 0 and last_ok reason_ok

fn registered_face_simple_glyph_summary_drain_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn RegisteredFaceSimpleGlyphTraversalMode impure fn i32 bool \owner\expected_capacity\mode\applied_count:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
            false
        Result::Ok budget_step:
            let status %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &budget_step
            let next_owner %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner budget_step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::StepBudgetExhausted:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                    false
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Completed:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                    false
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Rejected reason:
                    let reason_ok %bool match reason:
                        GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart: true
                    if and reason_ok eq applied_count 0:
                        then registered_face_simple_glyph_summary_rejected_ok next_owner expected_capacity
                        else:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                            false
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Applied _apply_status:
                    let next_count %i32 add applied_count 1
                    let progress %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind gui_font_registered_face_simple_glyph_indexed_action_summary_owner_progress_kind &next_owner
                    match progress:
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Rejected:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                            false
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Completed:
                            if ne next_count 16:
                                then:
                                    gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                                    false
                                else:
                                    match mode:
                                        RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryEmitClose:
                                            registered_face_simple_glyph_summary_completed_close_ok next_owner expected_capacity
                                        RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryKeepOpen:
                                            registered_face_simple_glyph_summary_completed_open_ok next_owner expected_capacity
                                        _:
                                            gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next_owner
                                            false
                        GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Running:
                            match registered_face_simple_glyph_summary_running_nonpositive_ok next_owner:
                                Result::Err _unit:
                                    false
                                Result::Ok exhausted_owner:
                                    let final_primary_ok %bool if eq next_count 15:
                                        then:
                                            match gui_font_registered_face_simple_glyph_indexed_action_summary_owner_progress_kind &exhausted_owner:
                                                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryProgressKind::Running: true
                                                _: false
                                        else true
                                    and final_primary_ok registered_face_simple_glyph_summary_drain_ok exhausted_owner expected_capacity mode next_count

fn registered_face_simple_glyph_indexed_action_summary_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedOwner impure fn RegisteredFaceSimpleGlyphTraversalMode bool \owner\mode:
    let collection %&GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_font_registered_face_simple_glyph_indexed_owner_collection_ref &owner
    let expected_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection
    let policy %GuiSfntSimpleGlyphPathSinkPolicy match mode:
        RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryEmitClose:
            gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
        RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryKeepOpen:
            gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::KeepOpen
        RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryRejectUnsupported:
            gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
        _:
            gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let summary %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_start owner &policy
    let summary_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_font_registered_face_simple_glyph_indexed_action_summary_owner_capacity &summary
    let initial_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_font_registered_face_simple_glyph_indexed_action_summary_owner_apply_state &summary
    match registered_face_simple_glyph_summary_running_seals_ok summary:
        Result::Err _unit:
            false
        Result::Ok recovered:
            match registered_face_simple_glyph_summary_running_nonpositive_ok recovered:
                Result::Err _unit:
                    false
                Result::Ok exhausted:
                    and registered_face_simple_glyph_capacity_same &expected_capacity &summary_capacity and registered_face_simple_glyph_summary_counts_ok &initial_state 0 0 0 0 registered_face_simple_glyph_summary_drain_ok exhausted &expected_capacity mode 0

fn registered_face_contour_endpoint_phase_valid %fn &GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner bool \owner:
    match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_phase_invariant_check owner:
        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointPhaseInvariantCheck::Valid: true
        _: false

fn registered_face_contour_endpoint_success_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner bool \owner:
    let start_cursor %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_cursor &owner
    let start_ok %bool and eq 0 gui_sfnt_simple_glyph_outline_scalar_region_cursor_start &start_cursor and eq 2 gui_sfnt_simple_glyph_outline_scalar_region_cursor_end &start_cursor and eq 0 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &start_cursor and is_none gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_previous_endpoint &owner and eq 0 gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_storage_len &owner registered_face_contour_endpoint_phase_valid &owner
    match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_drain_budget owner 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_free error
            false
        Result::Ok zero_step:
            let zero_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_status &zero_step:
                GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointBudgetStatus::StepBudgetExhausted: true
                _: false
            let zero_owner %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_take_owner zero_step
            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_drain_budget zero_owner sub 0 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_free error
                    false
                Result::Ok negative_step:
                    let negative_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_status &negative_step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointBudgetStatus::StepBudgetExhausted: true
                        _: false
                    let negative_owner %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_take_owner negative_step
                    match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_drain_budget negative_owner 99:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_free error
                            false
                        Result::Ok first_step:
                            let first_status_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_status &first_step:
                                GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointBudgetStatus::Pushed endpoint: and eq 0 gui_sfnt_simple_glyph_contour_endpoint_slot_contour_index &endpoint eq 1 gui_sfnt_simple_glyph_contour_endpoint_slot_end_point_index &endpoint
                                _: false
                            let first_owner %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_take_owner first_step
                            let first_cursor gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_cursor &first_owner
                            let first_previous_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_previous_endpoint &first_owner:
                                Option::Some endpoint: eq 1 endpoint
                                Option::None: false
                            let first_ok %bool and first_status_ok and eq 1 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &first_cursor and first_previous_ok and eq 1 gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_storage_len &first_owner registered_face_contour_endpoint_phase_valid &first_owner
                            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step first_owner:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_free error
                                    false
                                Result::Ok second_step:
                                    let second_status_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_status &second_step:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointBudgetStatus::Pushed endpoint: and eq 1 gui_sfnt_simple_glyph_contour_endpoint_slot_contour_index &endpoint eq 3 gui_sfnt_simple_glyph_contour_endpoint_slot_end_point_index &endpoint
                                        _: false
                                    let second_owner %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_take_owner second_step
                                    let second_cursor gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_cursor &second_owner
                                    let completed_state_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_progress_kind &second_owner:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointProgressKind::Completed: true
                                        _: false
                                    let second_previous_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_previous_endpoint &second_owner:
                                        Option::Some endpoint: eq 3 endpoint
                                        Option::None: false
                                    let second_ok %bool and second_status_ok and completed_state_ok and eq 2 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &second_cursor and second_previous_ok and eq 2 gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_storage_len &second_owner registered_face_contour_endpoint_phase_valid &second_owner
                                    match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_drain_budget second_owner sub 0 100:
                                        Result::Err error:
                                            gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_free error
                                            false
                                        Result::Ok completed_step:
                                            let completed_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_status &completed_step:
                                                GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointBudgetStatus::Completed: true
                                                _: false
                                            let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_take_owner completed_step
                                            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step completed_owner:
                                                Result::Ok unexpected:
                                                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_free unexpected
                                                    false
                                                Result::Err error:
                                                    let direct_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_kind &error:
                                                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointStepErrorKind::AlreadyCompleted: true
                                                        _: false
                                                    let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_take_owner error
                                                    match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_completed recovered:
                                                        Result::Err seal_error:
                                                            gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_error_free seal_error
                                                            false
                                                        Result::Ok sealed:
                                                            gui_font_registered_face_simple_glyph_indexed_contour_endpoint_completed_owner_free sealed
                                                            and start_ok and zero_ok and negative_ok and first_ok and second_ok and completed_ok direct_ok

fn registered_face_contour_endpoint_failure_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner impure fn RegisteredFaceSimpleGlyphTraversalMode bool \owner\mode:
    match mode:
        RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointPrematureSeal:
            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_completed owner:
                Result::Ok completed:
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_completed_owner_free completed
                    false
                Result::Err error:
                    let actual_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_error_actual &error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointProgressKind::Active: true
                        _: false
                    let invariant_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_error_invariant &error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointPhaseInvariantCheck::Valid: true
                        _: false
                    let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_error_take_owner error
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_free recovered
                    and actual_ok invariant_ok
        RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointLookupFailure:
            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_test_force_lookup_failure owner:
                Result::Ok step:
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_free step
                    false
                Result::Err error:
                    let kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_kind &error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointStepErrorKind::SpanLookupFailed: true
                        _: false
                    let contour_ok %bool eq 0 gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_contour_index &error
                    let lookup_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_lookup_error &error:
                        Option::Some lookup: and eq -1 gui_sfnt_simple_glyph_contour_span_index_lookup_error_requested_index &lookup eq 2 gui_sfnt_simple_glyph_contour_span_index_lookup_error_span_count &lookup
                        Option::None: false
                    let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_take_owner error
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_free recovered
                    and kind_ok and contour_ok lookup_ok
        _:
            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_test_force_push_failure owner:
                Result::Ok step:
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_free step
                    false
                Result::Err error:
                    let kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_kind &error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointStepErrorKind::EndpointPushFailed: true
                        _: false
                    let endpoint_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_endpoint &error:
                        Option::Some endpoint: and eq 0 gui_sfnt_simple_glyph_contour_endpoint_slot_contour_index &endpoint eq -1 gui_sfnt_simple_glyph_contour_endpoint_slot_end_point_index &endpoint
                        Option::None: false
                    let push_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_push_kind &error:
                        Option::Some kind:
                            match kind:
                                GuiSfntSimpleGlyphContourEndpointPushErrorKind::EndpointOutOfRange: true
                                _: false
                        Option::None: false
                    let lower_ok %bool and is_none gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_region_error_kind &error is_none gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_storage_push_error_kind &error
                    let recovered %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_take_owner error
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_free recovered
                    and kind_ok and endpoint_ok and push_ok lower_ok

fn registered_face_contour_endpoint_from_completed %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner impure fn RegisteredFaceSimpleGlyphTraversalMode bool \completed\mode:
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_font_registered_face_simple_glyph_indexed_outline_storage_start completed &limit:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_free error
            false
        Result::Ok storage:
            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_start storage:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_start_error_free error
                    false
                Result::Ok owner:
                    match mode:
                        RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointSuccess: registered_face_contour_endpoint_success_ok owner
                        _: registered_face_contour_endpoint_failure_ok owner mode

fn registered_face_contour_endpoint_summary_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner impure fn i32 impure fn RegisteredFaceSimpleGlyphTraversalMode bool \owner\count\mode:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
            false
        Result::Ok step:
            let status gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &step
            let next %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Applied _status:
                    if eq add count 1 16:
                        then:
                            match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_completed next:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_free error
                                    false
                                Result::Ok completed: registered_face_contour_endpoint_from_completed completed mode
                        else registered_face_contour_endpoint_summary_drain next add count 1 mode
                _:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next
                    false

fn registered_face_contour_endpoint_indexed_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedOwner impure fn RegisteredFaceSimpleGlyphTraversalMode bool \owner\mode:
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let summary %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_start owner &policy
    registered_face_contour_endpoint_summary_drain summary 0 mode

fn registered_face_simple_glyph_span_index_completed_ok %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedOwner impure fn RegisteredFaceSimpleGlyphTraversalMode bool \owner\mode:
    let evidence %GuiFontRegisteredFaceSimpleGlyphPointStream gui_font_registered_face_simple_glyph_indexed_owner_evidence &owner
    let mapping %GuiFontRegisteredFaceGlyphMapping gui_font_registered_face_simple_glyph_point_stream_mapping &evidence
    let glyph %GuiGlyphId gui_font_registered_face_glyph_mapping_glyph &mapping
    let first_ok %bool match gui_font_registered_face_simple_glyph_indexed_owner_span_lookup &owner 0:
        Result::Err _error:
            false
        Result::Ok span:
            and eq 0 gui_sfnt_simple_glyph_contour_span_start_point_index &span and eq 1 gui_sfnt_simple_glyph_contour_span_end_point_index &span eq 2 gui_sfnt_simple_glyph_contour_span_point_count &span
    let second_ok %bool match gui_font_registered_face_simple_glyph_indexed_owner_span_lookup &owner 1:
        Result::Err _error:
            false
        Result::Ok span:
            and eq 2 gui_sfnt_simple_glyph_contour_span_start_point_index &span and eq 3 gui_sfnt_simple_glyph_contour_span_end_point_index &span eq 2 gui_sfnt_simple_glyph_contour_span_point_count &span
    let shape_ok %bool and eq 36 gui_glyph_id_raw &glyph eq 2 gui_font_registered_face_simple_glyph_indexed_owner_span_count &owner
    let traversal_ok %bool match mode:
        RegisteredFaceSimpleGlyphTraversalMode::IndexedPath:
            registered_face_simple_glyph_indexed_path_ok owner
        RegisteredFaceSimpleGlyphTraversalMode::IndexedAction:
            registered_face_simple_glyph_indexed_action_ok owner
        _:
            match mode:
                RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryEmitClose: registered_face_simple_glyph_indexed_action_summary_ok owner mode
                RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryKeepOpen: registered_face_simple_glyph_indexed_action_summary_ok owner mode
                RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryRejectUnsupported: registered_face_simple_glyph_indexed_action_summary_ok owner mode
                _: registered_face_contour_endpoint_indexed_ok owner mode
    and shape_ok and first_ok and second_ok traversal_ok

fn registered_face_simple_glyph_span_index_drain_ok %impure fn GuiFontRegisteredFaceSimpleGlyphSpanIndexBuilderOwner impure fn i32 impure fn RegisteredFaceSimpleGlyphTraversalMode bool \owner\remaining\mode:
    if le remaining 0:
        then:
            match gui_font_registered_face_simple_glyph_span_index_complete owner:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_span_index_step_error_free error
                    false
                Result::Ok indexed:
                    registered_face_simple_glyph_span_index_completed_ok indexed mode
        else:
            match gui_font_registered_face_simple_glyph_span_index_step owner:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_span_index_step_error_free error
                    false
                Result::Ok next_owner:
                    registered_face_simple_glyph_span_index_drain_ok next_owner sub remaining 1 mode

fn registered_face_simple_glyph_span_index_ok %impure fn &GuiFontRegisteredFaceTableEntry impure fn GuiFontRegisteredFaceSimpleGlyphCollectedOwner impure fn RegisteredFaceSimpleGlyphTraversalMode bool \entry\owner\mode:
    let rejected_limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 1
    match gui_font_registered_face_simple_glyph_span_index_start entry owner &rejected_limit:
        Result::Ok unexpected_builder:
            gui_font_registered_face_simple_glyph_span_index_builder_free unexpected_builder
            false
        Result::Err start_error:
            let start_kind_ok %bool match gui_font_registered_face_simple_glyph_span_index_start_error_kind &start_error:
                GuiFontRegisteredFaceSimpleGlyphSpanIndexStartErrorKind::LowerStartRejected:
                    true
                _:
                    false
            let lower_kind_ok %bool match gui_font_registered_face_simple_glyph_span_index_start_error_lower_kind &start_error:
                Option::Some lower_kind:
                    match lower_kind:
                        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CapacityRejected:
                            true
                        _:
                            false
                _:
                    false
            let recovered %GuiFontRegisteredFaceSimpleGlyphCollectedOwner gui_font_registered_face_simple_glyph_span_index_start_error_take_owner start_error
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 2
            match gui_font_registered_face_simple_glyph_span_index_start entry recovered &limit:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_span_index_start_error_free error
                    false
                Result::Ok builder:
                    let initial_ok %bool and eq 0 gui_font_registered_face_simple_glyph_span_index_builder_next_point_index &builder eq 0 gui_font_registered_face_simple_glyph_span_index_builder_span_count &builder
                    match gui_font_registered_face_simple_glyph_span_index_complete builder:
                        Result::Ok unexpected_indexed:
                            gui_font_registered_face_simple_glyph_indexed_owner_free unexpected_indexed
                            false
                        Result::Err complete_error:
                            let complete_kind_ok %bool match gui_font_registered_face_simple_glyph_span_index_step_error_kind &complete_error:
                                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::CompletionInvariantInvalid:
                                    true
                                _:
                                    false
                            let recovered_builder %GuiFontRegisteredFaceSimpleGlyphSpanIndexBuilderOwner gui_font_registered_face_simple_glyph_span_index_step_error_take_owner complete_error
                            and start_kind_ok and lower_kind_ok and initial_ok and complete_kind_ok registered_face_simple_glyph_span_index_drain_ok recovered_builder 4 mode

fn registered_face_simple_glyph_collection_drain_ok %impure fn &GuiFontRegisteredFaceTableEntry impure fn GuiFontRegisteredFaceSimpleGlyphCollectionOwner impure fn i32 impure fn RegisteredFaceSimpleGlyphTraversalMode bool \entry\owner\expected_index\mode:
    match gui_font_registered_face_simple_glyph_collection_step entry owner:
        Result::Err error:
            gui_font_registered_face_simple_glyph_collection_step_error_free error
            false
        Result::Ok terminal:
            match terminal:
                GuiFontRegisteredFaceSimpleGlyphCollectionTerminal::Collecting next_owner:
                    if:
                        ge expected_index 4
                        then:
                            gui_font_registered_face_simple_glyph_collection_owner_free next_owner
                            false
                        else:
                            let item_ok %bool registered_face_simple_glyph_collection_item_ok gui_font_registered_face_simple_glyph_collection_owner_collection_ref &next_owner expected_index
                            if:
                                item_ok
                                then registered_face_simple_glyph_collection_drain_ok entry next_owner add expected_index 1 mode
                                else:
                                    gui_font_registered_face_simple_glyph_collection_owner_free next_owner
                                    false
                GuiFontRegisteredFaceSimpleGlyphCollectionTerminal::Completed completed:
                    if:
                        ne expected_index 4
                        then:
                            gui_font_registered_face_simple_glyph_collected_owner_free completed
                            false
                        else:
                            let collection %&GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_font_registered_face_simple_glyph_collected_owner_collection_ref &completed
                            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection
                            let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph &capacity
                            let invariant_ok %bool and eq gui_glyph_id_raw &glyph 36 and eq gui_sfnt_simple_glyph_outline_storage_capacity_contour_count &capacity 2 and eq gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity 4 and eq gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count collection 4 and eq gui_sfnt_simple_glyph_outline_point_stream_item_collection_items_len collection 4 eq gui_sfnt_simple_glyph_outline_point_stream_item_collection_items_cap collection 4
                            let spans_ok %bool registered_face_simple_glyph_collection_spans_ok collection
                            let index_ok %bool registered_face_simple_glyph_span_index_ok entry completed mode
                            and invariant_ok and spans_ok index_ok

fn registered_face_simple_glyph_collection_limit_rejected %impure fn &GuiFontRegisteredFaceTableEntry impure fn &GuiFontRegisteredFaceSimpleGlyphPointStream bool \entry\evidence:
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 3
    match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Ok owner:
            gui_font_registered_face_simple_glyph_collection_owner_free owner
            false
        Result::Err error:
            let kind_ok %bool match gui_font_registered_face_simple_glyph_collection_start_error_kind &error:
                GuiFontRegisteredFaceSimpleGlyphCollectionStartErrorKind::CollectionAllocFailed:
                    true
                _:
                    false
            let alloc_ok %bool match gui_font_registered_face_simple_glyph_collection_start_error_alloc_error &error:
                Option::None:
                    false
                Option::Some alloc_error:
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc_error_kind &alloc_error:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::CapacityRejected:
                            true
                        _:
                            false
            and kind_ok alloc_ok

fn registered_face_simple_glyph_collection_ok %impure fn &GuiFontRegisteredFaceTableEntry impure fn &GuiFontRegisteredFaceSimpleGlyphPointStream bool \entry\evidence:
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 4
    let path_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedPath
    let action_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedAction
    // registered face indexed action summary emit close 8/0/2/6
    // registered face indexed outline storage contour first failure positive-low zero negative arbitrary later limits
    // registered face indexed outline storage point first failure positive-low zero negative arbitrary later limits
    // registered face indexed outline storage edge first failure positive-low zero negative arbitrary later limits
    // registered face indexed outline storage command first failure positive-low zero negative
    // registered face indexed outline storage lower kind capacity limit capacity check read before take
    // registered face indexed outline storage forced ScalarSlotStorageAllocFailed owner recovery
    // registered face indexed outline storage exact 2/4/4/8 success
    // registered face indexed outline storage six-field capacity and storage capacity
    // registered face indexed outline storage apply counts 8/0/2/6 ClosedContour
    // registered face indexed outline storage scalar count 22 len 0 cap 22 invariant Valid free
    // registered face indexed outline storage allocation success 2/4/4/8
    // registered face indexed outline storage limit precedence
    // registered face indexed outline storage recovery retry
    // registered face indexed outline storage forced allocation failure
    // registered face indexed outline storage free order
    let summary_close_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryEmitClose
    // registered face indexed action summary keep open 8/0/0/8
    let summary_open_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryKeepOpen
    // registered face indexed action summary reject unsupported 0/1/0/0
    // registered face indexed action summary budget boundaries
    // registered face indexed action summary checked seals
    // registered face indexed action summary capacity preservation
    // registered face indexed action summary terminal misuse
    let summary_reject_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedActionSummaryRejectUnsupported
    let endpoint_success_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointSuccess
    let endpoint_seal_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointPrematureSeal
    let endpoint_lookup_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointLookupFailure
    let endpoint_push_ok %bool match gui_font_registered_face_simple_glyph_collection_start entry evidence &limit:
        Result::Err _error:
            false
        Result::Ok owner:
            registered_face_simple_glyph_collection_drain_ok entry owner 0 RegisteredFaceSimpleGlyphTraversalMode::IndexedContourEndpointPushFailure
    and path_ok and action_ok and summary_close_ok and summary_open_ok and summary_reject_ok and endpoint_success_ok and endpoint_seal_ok and endpoint_lookup_ok and endpoint_push_ok registered_face_simple_glyph_collection_limit_rejected entry evidence

fn registered_face_foreign_simple_glyph_evidence_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry Option GuiFontRegisteredFaceSimpleGlyphPointStream \table\entry:
    let evidence %Option GuiFontRegisteredFaceSimpleGlyphPointStream match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            none
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Err _error:
                    none
                Result::Ok value:
                    some value
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    evidence

fn registered_face_foreign_simple_glyph_evidence %impure fn void Option GuiFontRegisteredFaceSimpleGlyphPointStream \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            none
        Result::Ok table:
            match build_registered_face_for_simple_glyph 181 191 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f' false:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    none
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            none
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_foreign_simple_glyph_evidence_callback

fn registered_face_simple_glyph_foreign_evidence_rejected %impure fn &GuiFontRegisteredFaceTableEntry bool \entry:
    match registered_face_foreign_simple_glyph_evidence:
        Option::None:
            false
        Option::Some evidence:
            let reader_rejected %bool match gui_font_registered_face_simple_glyph_sequential_reader_start entry &evidence:
                Result::Ok _cursor:
                    false
                Result::Err error:
                    let kind %GuiFontRegisteredFaceSimpleGlyphSequentialReaderErrorKind gui_font_registered_face_simple_glyph_sequential_reader_error_kind &error
                    let record %GuiFontRegisteredFaceRecord gui_font_registered_face_simple_glyph_sequential_reader_error_record &error
                    let error_evidence %GuiFontRegisteredFaceSimpleGlyphPointStream gui_font_registered_face_simple_glyph_sequential_reader_error_evidence &error
                    let error_mapping %GuiFontRegisteredFaceGlyphMapping gui_font_registered_face_simple_glyph_point_stream_mapping &error_evidence
                    let foreign_record %GuiFontRegisteredFaceRecord gui_font_registered_face_glyph_mapping_record &error_mapping
                    and gui_font_registered_face_simple_glyph_sequential_reader_error_kind_eq kind GuiFontRegisteredFaceSimpleGlyphSequentialReaderErrorKind::EvidenceRecordMismatch and eq 151 gui_font_registered_face_record_resource_raw &record and eq 157 gui_font_registered_face_record_face_raw &record and eq 181 gui_font_registered_face_record_resource_raw &foreign_record and eq 191 gui_font_registered_face_record_face_raw &foreign_record and is_none gui_font_registered_face_simple_glyph_sequential_reader_error_cursor &error and is_none gui_font_registered_face_simple_glyph_sequential_reader_error_validation_error &error is_none gui_font_registered_face_simple_glyph_sequential_reader_error_parse_error &error
            let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 4
            let collection_rejected %bool match gui_font_registered_face_simple_glyph_collection_start entry &evidence &limit:
                Result::Ok owner:
                    gui_font_registered_face_simple_glyph_collection_owner_free owner
                    false
                Result::Err error:
                    let kind_ok %bool match gui_font_registered_face_simple_glyph_collection_start_error_kind &error:
                        GuiFontRegisteredFaceSimpleGlyphCollectionStartErrorKind::ReaderStartFailed:
                            true
                        _:
                            false
                    and kind_ok and is_some gui_font_registered_face_simple_glyph_collection_start_error_reader_error &error and is_none gui_font_registered_face_simple_glyph_collection_start_error_capacity_check &error is_none gui_font_registered_face_simple_glyph_collection_start_error_alloc_error &error
            and reader_rejected collection_rejected

fn registered_face_simple_glyph_success_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let success_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Err _error:
                    false
                Result::Ok evidence:
                    let evidence_mapping %GuiFontRegisteredFaceGlyphMapping gui_font_registered_face_simple_glyph_point_stream_mapping &evidence
                    let stream %GuiSfntSimpleGlyphPointStream gui_font_registered_face_simple_glyph_point_stream_stream &evidence
                    let topology %GuiSfntSimpleGlyphTopology gui_sfnt_simple_glyph_point_stream_topology &stream
                    let bounds %GuiSfntGlyphBounds gui_sfnt_simple_glyph_topology_bounds &topology
                    let glyph %GuiGlyphId gui_font_registered_face_glyph_mapping_glyph &evidence_mapping
                    and eq 36 gui_glyph_id_raw &glyph and eq 2 gui_sfnt_simple_glyph_topology_contour_count &topology and eq 4 gui_sfnt_simple_glyph_topology_point_count &topology and eq 1 gui_sfnt_simple_glyph_topology_instruction_length &topology and eq -10 gui_sfnt_glyph_bounds_x_min &bounds and eq -20 gui_sfnt_glyph_bounds_y_min &bounds and eq 100 gui_sfnt_glyph_bounds_x_max &bounds and eq 200 gui_sfnt_glyph_bounds_y_max &bounds and eq 17 gui_sfnt_simple_glyph_point_stream_flag_data_offset &stream and eq 4 gui_sfnt_simple_glyph_point_stream_flag_data_length &stream and eq 21 gui_sfnt_simple_glyph_point_stream_x_data_offset &stream and eq 5 gui_sfnt_simple_glyph_point_stream_x_data_length &stream and eq 26 gui_sfnt_simple_glyph_point_stream_y_data_offset &stream and eq 5 gui_sfnt_simple_glyph_point_stream_y_data_length &stream and eq 31 gui_sfnt_simple_glyph_point_stream_trailing_data_offset &stream and eq 3 gui_sfnt_simple_glyph_point_stream_trailing_data_length &stream registered_face_simple_glyph_reader_ok &entry &evidence
    let composite_ok %bool match gui_font_registered_face_glyph_lookup &entry 'B':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_simple_glyph_error_kind_is &error GuiFontRegisteredFaceSimpleGlyphLookupErrorKind::SfntParseFailed registered_face_simple_glyph_parse_error_is &error GuiSfntParseErrorKind::UnsupportedGlyphOutlineFormat
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok record:
            and eq 151 gui_font_registered_face_record_resource_raw &record eq 157 gui_font_registered_face_record_face_raw &record
    let foreign_evidence_rejected %bool registered_face_simple_glyph_foreign_evidence_rejected &entry
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and success_ok and composite_ok and foreign_evidence_rejected entry_reusable

fn registered_face_simple_glyph_collection_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let collection_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Err _error:
                    false
                Result::Ok evidence:
                    registered_face_simple_glyph_collection_ok &entry &evidence
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok _record:
            true
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and collection_ok entry_reusable

fn registered_face_simple_glyph_collection_success_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_simple_glyph 151 157 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f' false:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_simple_glyph_collection_callback

fn registered_face_simple_glyph_success_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_simple_glyph 151 157 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f' false:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_simple_glyph_success_callback

fn registered_face_simple_glyph_missing_loca_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let missing_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_simple_glyph_error_kind_is &error GuiFontRegisteredFaceSimpleGlyphLookupErrorKind::SfntParseFailed registered_face_simple_glyph_parse_error_is &error GuiSfntParseErrorKind::MissingTable
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok _record:
            true
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and missing_ok entry_reusable

fn registered_face_simple_glyph_missing_loca_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_simple_glyph 163 167 sfnt_tag4 'z' 'z' 'z' 'z' sfnt_tag4 'g' 'l' 'y' 'f' false:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_simple_glyph_missing_loca_callback

fn registered_face_simple_glyph_missing_glyf_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let missing_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_simple_glyph_error_kind_is &error GuiFontRegisteredFaceSimpleGlyphLookupErrorKind::SfntParseFailed registered_face_simple_glyph_parse_error_is &error GuiSfntParseErrorKind::MissingTable
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok _record:
            true
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and missing_ok entry_reusable

fn registered_face_simple_glyph_missing_glyf_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_simple_glyph 173 179 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'z' 'z' 'z' 'z' false:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_simple_glyph_missing_glyf_callback

fn registered_face_simple_glyph_malformed_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let malformed_ok %bool match gui_font_registered_face_glyph_lookup &entry 'A':
        Result::Err _error:
            false
        Result::Ok mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_simple_glyph_error_kind_is &error GuiFontRegisteredFaceSimpleGlyphLookupErrorKind::SfntParseFailed registered_face_simple_glyph_parse_error_is &error GuiSfntParseErrorKind::MalformedGlyfRecord
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok _record:
            true
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and malformed_ok entry_reusable

fn registered_face_simple_glyph_malformed_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_simple_glyph 181 191 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f' true:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_simple_glyph_malformed_callback

fn registered_face_simple_glyph_mismatch_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let mismatch_ok %bool match registered_face_foreign_mapping:
        Option::None:
            false
        Option::Some mapping:
            match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
                Result::Ok _evidence:
                    false
                Result::Err error:
                    and registered_face_simple_glyph_error_kind_is &error GuiFontRegisteredFaceSimpleGlyphLookupErrorKind::MappingRecordMismatch is_none gui_font_registered_face_simple_glyph_lookup_error_parse_error &error
    let entry_reusable %bool match gui_font_registered_face_table_entry_validate &entry:
        Result::Err _error:
            false
        Result::Ok record:
            and eq 193 gui_font_registered_face_record_resource_raw &record eq 197 gui_font_registered_face_record_face_raw &record
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    and mismatch_ok entry_reusable

fn registered_face_simple_glyph_mismatch_ok %impure fn void bool \void:
    match gui_font_registered_face_table_new 1:
        Result::Err _error:
            false
        Result::Ok table:
            match build_registered_face_for_simple_glyph 193 197 sfnt_tag4 'l' 'o' 'c' 'a' sfnt_tag4 'g' 'l' 'y' 'f' false:
                Result::Err _message:
                    gui_font_registered_face_table_free table
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_register table face:
                        Result::Err error:
                            gui_font_registered_face_table_register_error_free error
                            false
                        Result::Ok registration:
                            gui_font_registered_face_table_registration_with registration @registered_face_simple_glyph_mismatch_callback

fn main %impure fn void i32 \void:
    let report0 %TestReport parse_valid_registered_face
    let report1 %TestReport append_invalid_face_registered_case report0
    let report2 %TestReport append_malformed_registered_case report1
    let report3 %TestReport append_unsupported_decode_case report2
    let report4 %TestReport test_report_push report3 assert "invalid raw face id rejected" invalid_raw_face_id_rejected
    let report5 %TestReport test_report_push report4 assert "registered face table success" registered_face_table_success_ok
    let report6 %TestReport test_report_push report5 assert "registered face table duplicate recovery" registered_face_table_duplicate_recovery_ok
    let report7 %TestReport test_report_push report6 assert "registered face table duplicate face recovery" registered_face_table_duplicate_face_recovery_ok
    let report8 %TestReport test_report_push report7 assert "registered face glyph lookup success and missing recovery" registered_face_glyph_lookup_success_ok
    let report9 %TestReport test_report_push report8 assert "registered face horizontal metric success" registered_face_horizontal_metric_success_ok
    let report10 %TestReport test_report_push report9 assert "registered face horizontal metric missing table" registered_face_horizontal_metric_missing_ok
    let report11 %TestReport test_report_push report10 assert "registered face horizontal metric malformed table" registered_face_horizontal_metric_malformed_ok
    let report12 %TestReport test_report_push report11 assert "registered face horizontal metric mapping mismatch" registered_face_horizontal_metric_mismatch_ok
    let report13 %TestReport test_report_push report12 assert "registered face simple glyph success and composite rejection" registered_face_simple_glyph_success_ok
    let report14 %TestReport test_report_push report13 assert "registered face simple glyph linear collection and indexed contour endpoint population" registered_face_simple_glyph_collection_success_ok
    let report15 %TestReport test_report_push report14 assert "registered face simple glyph missing loca" registered_face_simple_glyph_missing_loca_ok
    let report16 %TestReport test_report_push report15 assert "registered face simple glyph missing glyf" registered_face_simple_glyph_missing_glyf_ok
    let report17 %TestReport test_report_push report16 assert "registered face simple glyph malformed point data" registered_face_simple_glyph_malformed_ok
    let report18 %TestReport test_report_push report17 assert "registered face simple glyph mapping mismatch" registered_face_simple_glyph_mismatch_ok
    let shown test_report_print_stdout report18
    test_report_exit_code shown
```
