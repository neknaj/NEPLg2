# GUI font SFNT glyf curve segment doctests

このファイルは、SFNT simple glyph contour edge から line / quadratic / no-segment を分類する pure classifier を検査する。
byte parser の大きい fixture ではなく typed value を直接組み立て、implied midpoint の doubled coordinate contract と enum state を確認する。

## line segment doubled coordinates

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
    let point0 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 0 0 true false
    let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 5 -3 true true
    let contour0 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point0
    let contour1 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 1 point1
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour0 contour1 0 1
    let no_lookahead %Option GuiSfntSimpleGlyphContourPoint Option::None
    let segment %GuiSfntSimpleGlyphCurveSegment gui_sfnt_classify_simple_glyph_curve_segment edge no_lookahead
    let ok %bool match segment:
        GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment:
            false
        GuiSfntSimpleGlyphCurveSegment::Line line:
            let ok_start_x %bool eq 0 gui_sfnt_simple_glyph_line_segment_start_x2 &line
            let ok_start_y %bool eq 0 gui_sfnt_simple_glyph_line_segment_start_y2 &line
            let ok_end_x %bool eq 10 gui_sfnt_simple_glyph_line_segment_end_x2 &line
            let ok_end_y %bool eq -6 gui_sfnt_simple_glyph_line_segment_end_y2 &line
            and ok_start_x and ok_start_y and ok_end_x ok_end_y
        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
            false
    test_assertion_exit_code assert "line segment doubled coordinates" ok
```

## explicit quadratic segment

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
    let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 5 3 false false
    let point2 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 2 8 7 true true
    let contour0 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point0
    let contour1 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 1 point1
    let contour2 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 2 point2
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour0 contour1 0 1
    let lookahead %Option GuiSfntSimpleGlyphContourPoint Option::Some contour2
    let segment %GuiSfntSimpleGlyphCurveSegment gui_sfnt_classify_simple_glyph_curve_segment edge lookahead
    let ok %bool match segment:
        GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment:
            false
        GuiSfntSimpleGlyphCurveSegment::Line line:
            false
        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
            let ok_start_x %bool eq 0 gui_sfnt_simple_glyph_quadratic_segment_start_x2 &quadratic
            let ok_start_y %bool eq 0 gui_sfnt_simple_glyph_quadratic_segment_start_y2 &quadratic
            let ok_control_x %bool eq 10 gui_sfnt_simple_glyph_quadratic_segment_control_x2 &quadratic
            let ok_control_y %bool eq 6 gui_sfnt_simple_glyph_quadratic_segment_control_y2 &quadratic
            let ok_end_x %bool eq 16 gui_sfnt_simple_glyph_quadratic_segment_end_x2 &quadratic
            let ok_end_y %bool eq 14 gui_sfnt_simple_glyph_quadratic_segment_end_y2 &quadratic
            let ok_implied %bool not gui_sfnt_simple_glyph_quadratic_segment_end_is_implied &quadratic
            and ok_start_x and ok_start_y and ok_control_x and ok_control_y and ok_end_x and ok_end_y ok_implied
    test_assertion_exit_code assert "quadratic explicit end" ok
```

## implied midpoint keeps odd doubled coordinates

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
    let lookahead %Option GuiSfntSimpleGlyphContourPoint Option::Some contour2
    let segment %GuiSfntSimpleGlyphCurveSegment gui_sfnt_classify_simple_glyph_curve_segment edge lookahead
    let ok %bool match segment:
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
    test_assertion_exit_code assert "quadratic implied odd midpoint" ok
```

## single point contour produces no segment

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_span glyph 0 0 0 1
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 2 2 true true
    let contour %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour contour 0 0
    let no_lookahead %Option GuiSfntSimpleGlyphContourPoint Option::None
    let segment %GuiSfntSimpleGlyphCurveSegment gui_sfnt_classify_simple_glyph_curve_segment edge no_lookahead
    let ok %bool match segment:
        GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment:
            match gui_sfnt_simple_glyph_curve_no_segment_reason &no_segment:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
        GuiSfntSimpleGlyphCurveSegment::Line line:
            false
        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
            false
    test_assertion_exit_code assert "single point contour is no segment" ok
```

## off curve start produces no segment

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_span glyph 0 0 0 2
    let point0 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 1 1 false false
    let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 3 5 true true
    let contour0 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point0
    let contour1 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 1 point1
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour0 contour1 0 1
    let no_lookahead %Option GuiSfntSimpleGlyphContourPoint Option::None
    let segment %GuiSfntSimpleGlyphCurveSegment gui_sfnt_classify_simple_glyph_curve_segment edge no_lookahead
    let ok %bool match segment:
        GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment:
            match gui_sfnt_simple_glyph_curve_no_segment_reason &no_segment:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
        GuiSfntSimpleGlyphCurveSegment::Line line:
            false
        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
            false
    test_assertion_exit_code assert "off curve start is no segment" ok
```

## missing lookahead produces no segment

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_span glyph 0 0 0 3
    let point0 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 0 0 true false
    let point1 %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 1 5 3 false false
    let contour0 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 0 point0
    let contour1 %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_point span 1 point1
    let edge %GuiSfntSimpleGlyphContourEdge gui_sfnt_simple_glyph_contour_edge contour0 contour1 0 1
    let no_lookahead %Option GuiSfntSimpleGlyphContourPoint Option::None
    let segment %GuiSfntSimpleGlyphCurveSegment gui_sfnt_classify_simple_glyph_curve_segment edge no_lookahead
    let ok %bool match segment:
        GuiSfntSimpleGlyphCurveSegment::NoSegment no_segment:
            match gui_sfnt_simple_glyph_curve_no_segment_reason &no_segment:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    true
        GuiSfntSimpleGlyphCurveSegment::Line line:
            false
        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
            false
    test_assertion_exit_code assert "missing lookahead is no segment" ok
```
