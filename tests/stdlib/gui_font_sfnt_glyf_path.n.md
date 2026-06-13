# GUI font SFNT glyf path command doctests

このファイルは、F4l の `GuiSfntSimpleGlyphCurveSegment` を F4m/F4o の path command projection へ写す pure helper を検査する。
full outline `Vec`、rasterizer、platform API は使わず、typed value だけで line / quadratic / no-segment の command projection と single-edge command pair を確認する。

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
    let pair %GuiSfntSimpleGlyphPathCommandPair gui_sfnt_simple_glyph_curve_segment_path_command_pair &segment
    let pair_move_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_move_command &pair
    let pair_move_ok %bool match pair_move_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            let ok_x %bool eq 2 gui_sfnt_simple_glyph_path_move_to_x2 &move_to
            let ok_y %bool eq 4 gui_sfnt_simple_glyph_path_move_to_y2 &move_to
            and ok_x ok_y
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let pair_draw_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_draw_command &pair
    let pair_line_ok %bool match pair_draw_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            let ok_x %bool eq 10 gui_sfnt_simple_glyph_path_line_to_x2 &line_to
            let ok_y %bool eq -6 gui_sfnt_simple_glyph_path_line_to_y2 &line_to
            and ok_x ok_y
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    test_assertion_exit_code assert "line path commands" and move_ok and line_ok and pair_move_ok pair_line_ok
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
    let pair %GuiSfntSimpleGlyphPathCommandPair gui_sfnt_simple_glyph_curve_segment_path_command_pair &segment
    let pair_move_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_move_command &pair
    let pair_move_ok %bool match pair_move_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            let ok_x %bool eq 0 gui_sfnt_simple_glyph_path_move_to_x2 &move_to
            let ok_y %bool eq 0 gui_sfnt_simple_glyph_path_move_to_y2 &move_to
            and ok_x ok_y
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let pair_draw_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_draw_command &pair
    let pair_quadratic_ok %bool match pair_draw_command:
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
            and ok_control_x and ok_control_y and ok_end_x and ok_end_y ok_implied
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    test_assertion_exit_code assert "quadratic path command keeps odd doubled midpoint" and pair_move_ok pair_quadratic_ok
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
    let move_skip_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_curve_segment_move_to_command &segment
    let move_skip_ok %bool match move_skip_command:
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
    let pair %GuiSfntSimpleGlyphPathCommandPair gui_sfnt_simple_glyph_curve_segment_path_command_pair &segment
    let pair_move_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_move_command &pair
    let pair_move_ok %bool match pair_move_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            let ok_reason %bool match gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
            ok_reason
    let pair_draw_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_draw_command &pair
    let pair_draw_ok %bool match pair_draw_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            let ok_reason %bool match gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
            ok_reason
    test_assertion_exit_code assert "no segment path command is explicit skip" and move_skip_ok and skip_ok and pair_move_ok pair_draw_ok
```

## sink event pair wraps existing path commands

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target wasi

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/math" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let skip_payload %GuiSfntSimpleGlyphPathSkipNoSegment gui_sfnt_simple_glyph_path_skip_no_segment 4 5 GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour
    let skip_command %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip_payload
    let event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_command_sink_event skip_command
    let event_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &event
    let event_ok %bool match event_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            true
    let skip_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind &event
    let skip_kind_ok %bool match skip_kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            match reason:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
    let move_payload %GuiSfntSimpleGlyphPathMoveTo gui_sfnt_simple_glyph_path_move_to 7 8 10 12
    let line_payload %GuiSfntSimpleGlyphPathLineTo gui_sfnt_simple_glyph_path_line_to 7 8 14 16
    let move_command %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::MoveTo move_payload
    let line_command %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::LineTo line_payload
    let command_pair %GuiSfntSimpleGlyphPathCommandPair gui_sfnt_simple_glyph_path_command_pair move_command line_command
    let event_pair %GuiSfntSimpleGlyphPathSinkEventPair gui_sfnt_simple_glyph_path_command_pair_sink_event_pair &command_pair
    let first_event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_sink_event_pair_first_event &event_pair
    let second_event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_sink_event_pair_second_event &event_pair
    let first_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &first_event
    let second_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &second_event
    let first_ok %bool match first_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            true
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let second_ok %bool match second_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            true
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let kind_pair %GuiSfntSimpleGlyphPathSinkEventKindPair gui_sfnt_simple_glyph_path_sink_event_pair_kind_pair &event_pair
    let first_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind &kind_pair
    let second_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind &kind_pair
    let first_kind_ok %bool match first_kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            false
    let second_kind_ok %bool match second_kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            false
    let slot_first_event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_sink_event_pair_event_at &event_pair GuiSfntSimpleGlyphPathSinkEventSlot::First
    let slot_second_event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_sink_event_pair_event_at &event_pair GuiSfntSimpleGlyphPathSinkEventSlot::Second
    let slot_first_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &slot_first_event
    let slot_second_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &slot_second_event
    let slot_first_event_ok %bool match slot_first_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            true
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            false
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let slot_second_event_ok %bool match slot_second_command:
        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
            false
        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
            true
        GuiSfntSimpleGlyphPathCommand::QuadraticTo quadratic_to:
            false
        GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
            false
    let slot_first_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_pair_kind_at &event_pair GuiSfntSimpleGlyphPathSinkEventSlot::First
    let slot_second_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_pair_kind_at &event_pair GuiSfntSimpleGlyphPathSinkEventSlot::Second
    let slot_first_kind_from_pair %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at &kind_pair GuiSfntSimpleGlyphPathSinkEventSlot::First
    let slot_second_kind_from_pair %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind_pair_kind_at &kind_pair GuiSfntSimpleGlyphPathSinkEventSlot::Second
    let slot_first_kind_ok %bool match slot_first_kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            false
    let slot_second_kind_ok %bool match slot_second_kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            false
    let slot_first_kind_pair_ok %bool match slot_first_kind_from_pair:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            false
    let slot_second_kind_pair_ok %bool match slot_second_kind_from_pair:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            false
    let slot_event_ok %bool and slot_first_event_ok slot_second_event_ok
    let slot_kind_ok %bool and slot_first_kind_ok and slot_second_kind_ok and slot_first_kind_pair_ok slot_second_kind_pair_ok
    test_assertion_exit_code assert "sink event pair wraps path commands" and event_ok and skip_kind_ok and first_ok and second_ok and first_kind_ok and second_kind_ok and slot_event_ok slot_kind_ok
```
