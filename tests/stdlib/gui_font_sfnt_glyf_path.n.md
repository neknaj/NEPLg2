# GUI font SFNT glyf path command doctests

このファイルは、F4l の `GuiSfntSimpleGlyphCurveSegment` を F4m の path command projection へ写す pure helper を検査する。
full outline `Vec`、rasterizer、platform API は使わず、typed value だけで line / quadratic / no-segment の command sequence を確認する。

## line segment emits move and line commands

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
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

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_span glyph 0 0 0 2
    let point0 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 1 2 true false
    let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 5 -3 true true
    let contour0 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point0
    let contour1 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 1 point1
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour0 contour1 0 1
    let line %GuiSfntSimpleGlyphLineSegment gui_sfnt_simple_glyph_line_segment edge 2 4 10 -6
    let segment %GuiSfntSimpleGlyphCurveSegment GuiSfntSimpleGlyphCurveSegment::Line line
    let move_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_curve_segment_move_to_command &segment
    let move_ok %bool match move_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            let ok_x %bool eq 2 gui_sfnt_simple_glyph_path_move_to_x2 &move_to
            let ok_y %bool eq 4 gui_sfnt_simple_glyph_path_move_to_y2 &move_to
            let ok_contour %bool eq 0 gui_sfnt_simple_glyph_path_move_to_contour_index &move_to
            let ok_edge %bool eq 0 gui_sfnt_simple_glyph_path_move_to_edge_index &move_to
            and ok_x and ok_y and ok_contour ok_edge
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let line_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_curve_segment_draw_command &segment
    let line_ok %bool match line_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            let ok_x %bool eq 10 gui_sfnt_simple_glyph_path_line_to_x2 &line_to
            let ok_y %bool eq -6 gui_sfnt_simple_glyph_path_line_to_y2 &line_to
            let ok_contour %bool eq 0 gui_sfnt_simple_glyph_path_line_to_contour_index &line_to
            let ok_edge %bool eq 0 gui_sfnt_simple_glyph_path_line_to_edge_index &line_to
            and ok_x and ok_y and ok_contour ok_edge
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    test_assertion_exit_code assert "line path commands" and move_ok line_ok
```

## implied quadratic keeps doubled coordinates

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
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

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_span glyph 0 0 0 3
    let point0 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 0 0 true false
    let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 1 3 false false
    let point2 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 2 4 8 false true
    let contour0 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point0
    let contour1 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 1 point1
    let contour2 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 2 point2
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour0 contour1 0 1
    let quadratic %GuiSfntSimpleGlyphQuadraticSegment gui_sfnt_simple_glyph_quadratic_segment edge contour2 0 0 2 6 5 11 true
    let segment %GuiSfntSimpleGlyphCurveSegment GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic
    let quadratic_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_curve_segment_draw_command &segment
    let quadratic_ok %bool match quadratic_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            let ok_control_x %bool eq 2 gui_sfnt_simple_glyph_path_quadratic_to_control_x2 &quadratic_to
            let ok_control_y %bool eq 6 gui_sfnt_simple_glyph_path_quadratic_to_control_y2 &quadratic_to
            let ok_end_x %bool eq 5 gui_sfnt_simple_glyph_path_quadratic_to_end_x2 &quadratic_to
            let ok_end_y %bool eq 11 gui_sfnt_simple_glyph_path_quadratic_to_end_y2 &quadratic_to
            let ok_implied %bool gui_sfnt_simple_glyph_path_quadratic_to_end_is_implied &quadratic_to
            let ok_contour %bool eq 0 gui_sfnt_simple_glyph_path_quadratic_to_contour_index &quadratic_to
            let ok_edge %bool eq 0 gui_sfnt_simple_glyph_path_quadratic_to_edge_index &quadratic_to
            and ok_control_x and ok_control_y and ok_end_x and ok_end_y and ok_implied and ok_contour ok_edge
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    test_assertion_exit_code assert "quadratic path command keeps odd doubled midpoint" quadratic_ok
```

## no segment emits explicit skip command

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
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

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_span glyph 0 0 0 1
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 2 2 true true
    let contour %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour contour 0 0
    let no_segment %GuiSfntSimpleGlyphCurveNoSegment gui_sfnt_simple_glyph_curve_no_segment edge GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour
    let segment %GuiSfntSimpleGlyphCurveSegment GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment
    let skip_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_curve_segment_draw_command &segment
    let skip_ok %bool match skip_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            let ok_contour %bool eq 0 gui_sfnt_simple_glyph_path_skip_no_segment_contour_index &skip
            let ok_edge %bool eq 0 gui_sfnt_simple_glyph_path_skip_no_segment_edge_index &skip
            let ok_reason %bool match gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
            and ok_contour and ok_edge ok_reason
    test_assertion_exit_code assert "no segment path command is explicit skip" skip_ok
```
