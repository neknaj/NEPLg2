# GUI font SFNT glyf curve segment public lookup doctests

このファイルは、`gui_sfnt_lookup_simple_glyph_curve_segment` が public SFNT byte lookup 経由で F4l curve segment classifier へ到達することを検査する。
既存の大きい glyf fixture へ追記せず、1 contour / 3 point の最小 byte fixture だけを使う。
現時点の compiler では `alloc/gui/font/sfnt/glyf` import の resource static check が 60 秒制限に近いため、CI の通常 doctest では skip し、source policy で fixture と public lookup 経路の存在を固定する。

## public lookup returns implied quadratic segment

neplg2:test[skip, stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn curve_lookup_fixture_len %fn void i32 \void:
    192

fn curve_lookup_fixture_byte %fn i32 i32 \idx:
    match idx:
        1:
            1
        5:
            5
        12:
            104
        13:
            101
        14:
            97
        15:
            100
        23:
            92
        27:
            52
        28:
            104
        29:
            104
        30:
            101
        31:
            97
        39:
            144
        43:
            10
        44:
            109
        45:
            97
        46:
            120
        47:
            112
        55:
            154
        59:
            6
        60:
            108
        61:
            111
        62:
            99
        63:
            97
        71:
            160
        75:
            10
        76:
            103
        77:
            108
        78:
            121
        79:
            102
        87:
            170
        91:
            22
        110:
            8
        149:
            10
        155:
            1
        159:
            4
        165:
            11
        167:
            11
        169:
            11
        171:
            1
        177:
            10
        179:
            10
        181:
            2
        184:
            49
        185:
            54
        186:
            54
        187:
            1
        188:
            3
        189:
            3
        190:
            5
        _:
            0

fn curve_lookup_push_fixture_bytes %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\idx:
    if:
        ge idx curve_lookup_fixture_len
        then:
            Result::Ok builder
        else:
            match byte_builder_push_u8 builder curve_lookup_fixture_byte idx:
                Result::Err error:
                    byte_builder_error_free error
                    Result::Err "curve lookup fixture byte push failed"
                Result::Ok next:
                    curve_lookup_push_fixture_bytes next add idx 1

fn curve_lookup_fixture_bytes %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity curve_lookup_fixture_len:
        Result::Err _error:
            Result::Err "curve lookup fixture builder allocation failed"
        Result::Ok builder:
            match curve_lookup_push_fixture_bytes builder 0:
                Result::Err message:
                    Result::Err message
                Result::Ok filled:
                    match byte_builder_finish filled:
                        Result::Err error:
                            byte_builder_error_free error
                            Result::Err "curve lookup fixture finish failed"
                        Result::Ok bytes:
                            Result::Ok bytes

fn curve_lookup_segment_is_implied_quadratic %fn GuiSfntSimpleGlyphCurveSegment bool \segment:
    match segment:
        GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment:
            false
        GuiSfntSimpleGlyphCurveSegment::Line line:
            false
        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
            let ok_control_x %bool eq 2 gui_sfnt_simple_glyph_quadratic_segment_control_x2 &quadratic
            let ok_control_y %bool eq 6 gui_sfnt_simple_glyph_quadratic_segment_control_y2 &quadratic
            let ok_end_x %bool eq 5 gui_sfnt_simple_glyph_quadratic_segment_end_x2 &quadratic
            let ok_end_y %bool eq 11 gui_sfnt_simple_glyph_quadratic_segment_end_y2 &quadratic
            let ok_implied %bool gui_sfnt_simple_glyph_quadratic_segment_end_is_implied &quadratic
            and ok_control_x and ok_control_y and ok_end_x and ok_end_y ok_implied

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    match curve_lookup_fixture_bytes:
        Result::Err _error:
            test_assertion_exit_code assert "curve lookup fixture builds" false
        Result::Ok bytes:
            let ok %bool match gui_sfnt_lookup_simple_glyph_curve_segment &bytes none glyph 0 0:
                Result::Err _error:
                    false
                Result::Ok segment:
                    curve_lookup_segment_is_implied_quadratic segment
            io_bytebuf_free bytes
            test_assertion_exit_code assert "curve lookup implied odd midpoint through public lookup" ok
```
