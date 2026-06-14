# GUI font SFNT glyf outline point stream item doctests

このファイルは、F5q の full point stream item classification が on-curve/off-curve と contour endpoint を typed enum として保持し、後続 phase が bool field を重複解釈しなくてよいことを検査する。

## point stream item classifies endpoint and curve state

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_sfnt_glyf_outline_point_stream_item_classification\" count=6 failed=0\nassertion index=0 status=ok kind=bool label=\"point stream item on curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"point stream item off curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"point stream item end on curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"point stream item end off curve\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"point stream item accessor kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"point stream item accessor point index\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/result" as *
#import "std/test" as *

fn kind_is %fn GuiSfntSimpleGlyphOutlinePointStreamItemKind fn GuiSfntSimpleGlyphOutlinePointStreamItemKind bool \observed\expected:
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
                    true
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
                    true
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
                    true
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
                    true

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 100
    let point_on %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 10 20 true false
    let point_off %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 11 21 false false
    let point_end_on %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 2 12 22 true true
    let point_end_off %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 3 13 23 false true
    let kind_on %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point &point_on
    let kind_off %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point &point_off
    let kind_end_on %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point &point_end_on
    let kind_end_off %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind_from_point &point_end_off
    let item %GuiSfntSimpleGlyphOutlinePointStreamItem gui_sfnt_simple_glyph_outline_point_stream_item point_end_off
    let item_kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
    let item_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
    let report %TestReport:
        test_report_new "gui_sfnt_glyf_outline_point_stream_item_classification"
        |> test_report_push assert "point stream item on curve" kind_is kind_on GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve
        |> test_report_push assert "point stream item off curve" kind_is kind_off GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve
        |> test_report_push assert "point stream item end on curve" kind_is kind_end_on GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve
        |> test_report_push assert "point stream item end off curve" kind_is kind_end_off GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
        |> test_report_push assert "point stream item accessor kind" kind_is item_kind GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
        |> test_report_push assert_eq_i32 "point stream item accessor point index" 3 gui_sfnt_simple_glyph_point_index &item_point
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
