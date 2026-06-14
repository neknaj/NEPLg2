# GUI font SFNT glyf outline point stream item step doctests

このファイルは、F5r の point stream item step 変換が F5o の成功値 shape を再検査し、classified item と terminal end だけを typed status として後続 phase に渡すことを検査する。

## point stream item step validates F5o step shape

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_sfnt_glyf_outline_point_stream_item_step\" count=10 failed=0\nassertion index=0 status=ok kind=bool label=\"point stream item step point status\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"point stream item step point next cursor\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"point stream item step item kind\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"point stream item step item point index\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"point stream item step end status\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"point stream item step end item none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"point stream item step point none invariant\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"point stream item step end some invariant\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"point stream item step point cursor invariant\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"point stream item step end cursor invariant\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn item_step_status_is %fn GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus fn GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus bool \observed\expected:
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::Item:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::Item:
                    true
                GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::End:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::End:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::Item:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::End:
                    true

fn item_kind_is %fn GuiSfntSimpleGlyphOutlinePointStreamItemKind fn GuiSfntSimpleGlyphOutlinePointStreamItemKind bool \observed\expected:
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
                    true
                _:
                    false

fn make_point_step %fn GuiSfntSimpleGlyphPoint fn i32 fn i32 GuiSfntSimpleGlyphOutlinePointReadStep \point\cursor_index\next_index:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor cursor_index
    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor next_index
    let point_option %Option GuiSfntSimpleGlyphPoint some point
    gui_sfnt_simple_glyph_outline_point_read_step GuiSfntSimpleGlyphOutlinePointReadStepStatus::Point cursor next_cursor point_option

fn make_point_none_step %fn i32 fn i32 GuiSfntSimpleGlyphOutlinePointReadStep \cursor_index\next_index:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor cursor_index
    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor next_index
    let point_option %Option GuiSfntSimpleGlyphPoint none
    gui_sfnt_simple_glyph_outline_point_read_step GuiSfntSimpleGlyphOutlinePointReadStepStatus::Point cursor next_cursor point_option

fn make_end_step %fn i32 fn i32 GuiSfntSimpleGlyphOutlinePointReadStep \cursor_index\next_index:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor cursor_index
    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor next_index
    let point_option %Option GuiSfntSimpleGlyphPoint none
    gui_sfnt_simple_glyph_outline_point_read_step GuiSfntSimpleGlyphOutlinePointReadStepStatus::End cursor next_cursor point_option

fn make_end_some_step %fn GuiSfntSimpleGlyphPoint fn i32 fn i32 GuiSfntSimpleGlyphOutlinePointReadStep \point\cursor_index\next_index:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor cursor_index
    let next_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor next_index
    let point_option %Option GuiSfntSimpleGlyphPoint some point
    gui_sfnt_simple_glyph_outline_point_read_step GuiSfntSimpleGlyphOutlinePointReadStepStatus::End cursor next_cursor point_option

fn converted_status_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus bool \source\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step source:
        Result::Err _error:
            false
        Result::Ok step:
            item_step_status_is gui_sfnt_simple_glyph_outline_point_stream_item_step_status &step expected

fn converted_next_cursor_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn i32 bool \source\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step source:
        Result::Err _error:
            false
        Result::Ok step:
            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_stream_item_step_next_cursor &step
            eq expected gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &cursor

fn converted_item_kind_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn GuiSfntSimpleGlyphOutlinePointStreamItemKind bool \source\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step source:
        Result::Err _error:
            false
        Result::Ok step:
            match gui_sfnt_simple_glyph_outline_point_stream_item_step_item &step:
                Option::None:
                    false
                Option::Some item:
                    let kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
                    item_kind_is kind expected

fn converted_item_point_index_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn i32 bool \source\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step source:
        Result::Err _error:
            false
        Result::Ok step:
            match gui_sfnt_simple_glyph_outline_point_stream_item_step_item &step:
                Option::None:
                    false
                Option::Some item:
                    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
                    eq expected gui_sfnt_simple_glyph_point_index &point

fn converted_item_none_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn bool bool \source\expected:
    let observed %bool match gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step source:
        Result::Err _error:
            false
        Result::Ok step:
            match gui_sfnt_simple_glyph_outline_point_stream_item_step_item &step:
                Option::None:
                    true
                Option::Some _item:
                    false
    match expected:
        true:
            observed
        false:
            not observed

fn conversion_error_invalid_is %fn &GuiSfntSimpleGlyphOutlinePointReadStep fn bool bool \source\expected:
    let observed %bool match gui_sfnt_simple_glyph_outline_point_stream_item_step_from_point_step source:
        Result::Ok _step:
            false
        Result::Err error:
            match gui_sfnt_simple_glyph_outline_point_stream_item_step_error_kind &error:
                GuiSfntSimpleGlyphOutlinePointStreamItemStepErrorKind::PointStepInvariantInvalid:
                    true
    match expected:
        true:
            observed
        false:
            not observed

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 100
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 3 13 23 false true
    let point_step %GuiSfntSimpleGlyphOutlinePointReadStep make_point_step point 0 1
    let end_step %GuiSfntSimpleGlyphOutlinePointReadStep make_end_step 2 2
    let point_none_step %GuiSfntSimpleGlyphOutlinePointReadStep make_point_none_step 0 1
    let end_some_step %GuiSfntSimpleGlyphOutlinePointReadStep make_end_some_step point 0 0
    let point_bad_cursor_step %GuiSfntSimpleGlyphOutlinePointReadStep make_point_step point 0 2
    let end_bad_cursor_step %GuiSfntSimpleGlyphOutlinePointReadStep make_end_step 2 3
    let report %TestReport:
        test_report_new "gui_sfnt_glyf_outline_point_stream_item_step"
        |> test_report_push assert "point stream item step point status" converted_status_is &point_step GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::Item
        |> test_report_push assert "point stream item step point next cursor" converted_next_cursor_is &point_step 1
        |> test_report_push assert "point stream item step item kind" converted_item_kind_is &point_step GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
        |> test_report_push assert "point stream item step item point index" converted_item_point_index_is &point_step 3
        |> test_report_push assert "point stream item step end status" converted_status_is &end_step GuiSfntSimpleGlyphOutlinePointStreamItemStepStatus::End
        |> test_report_push assert "point stream item step end item none" converted_item_none_is &end_step true
        |> test_report_push assert "point stream item step point none invariant" conversion_error_invalid_is &point_none_step true
        |> test_report_push assert "point stream item step end some invariant" conversion_error_invalid_is &end_some_step true
        |> test_report_push assert "point stream item step point cursor invariant" conversion_error_invalid_is &point_bad_cursor_step true
        |> test_report_push assert "point stream item step end cursor invariant" conversion_error_invalid_is &end_bad_cursor_step true
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
