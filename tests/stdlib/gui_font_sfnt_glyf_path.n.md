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
#import "core/gui/font" as *
#import "core/math" as *
#import "core/result" as *
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

## contour cursor step stores explicit next state

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target wasi

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 12
    let cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 2 3 GuiSfntSimpleGlyphPathSinkEventSlot::First
    let next_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 2 3 GuiSfntSimpleGlyphPathSinkEventSlot::Second
    let next %GuiSfntSimpleGlyphPathContourNext GuiSfntSimpleGlyphPathContourNext::Continue next_cursor
    let skip_payload %GuiSfntSimpleGlyphPathSkipNoSegment gui_sfnt_simple_glyph_path_skip_no_segment 2 3 GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart
    let skip_command %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip_payload
    let event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_command_sink_event skip_command
    let kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind &event
    let step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_path_contour_step cursor event kind next
    let stored_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_step_cursor &step
    let stored_next %GuiSfntSimpleGlyphPathContourNext gui_sfnt_simple_glyph_path_contour_step_next &step
    let stored_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_contour_step_kind &step
    let cursor_ok %bool and eq 2 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &stored_cursor eq 3 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &stored_cursor
    let next_ok %bool match stored_next:
        GuiSfntSimpleGlyphPathContourNext::Continue stored_next_cursor:
            match gui_sfnt_simple_glyph_path_contour_cursor_slot &stored_next_cursor:
                GuiSfntSimpleGlyphPathSinkEventSlot::First:
                    false
                GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                    true
        GuiSfntSimpleGlyphPathContourNext::EndContour:
            false
    let end_next %GuiSfntSimpleGlyphPathContourNext GuiSfntSimpleGlyphPathContourNext::EndContour
    let end_ok %bool match end_next:
        GuiSfntSimpleGlyphPathContourNext::Continue unused:
            false
        GuiSfntSimpleGlyphPathContourNext::EndContour:
            true
    let kind_ok %bool match stored_kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            match reason:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                    false
                GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                    true
                GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                    false
    test_assertion_exit_code assert "contour step stores explicit next state" and cursor_ok and next_ok and end_ok kind_ok
```

## path sink policy keeps reject and close tail exclusive

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target wasi

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn primary_is_reject_off_curve %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_primary_action step:
        GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject reason:
            match reason:
                GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                    true

fn primary_is_emit_off_curve %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_primary_action step:
        GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent event:
            let command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &event
            match command:
                GuiSfntSimpleGlyphPathCommand::MoveTo _move_to:
                    false
                GuiSfntSimpleGlyphPathCommand::LineTo _line_to:
                    false
                GuiSfntSimpleGlyphPathCommand::QuadraticTo _quadratic_to:
                    false
                GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
                    let reason %GuiSfntSimpleGlyphCurveNoSegmentReason gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip
                    match reason:
                        GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                            false
                        GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                            true
                        GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                            false
        GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject _reason:
            false

fn primary_is_emit_single_point %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_primary_action step:
        GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent event:
            let command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &event
            match command:
                GuiSfntSimpleGlyphPathCommand::MoveTo _move_to:
                    false
                GuiSfntSimpleGlyphPathCommand::LineTo _line_to:
                    false
                GuiSfntSimpleGlyphPathCommand::QuadraticTo _quadratic_to:
                    false
                GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
                    let reason %GuiSfntSimpleGlyphCurveNoSegmentReason gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip
                    match reason:
                        GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                            true
                        GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                            false
                        GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                            false
        GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject _reason:
            false

fn tail_is_none %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_tail_action step:
        GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction:
            true
        GuiSfntSimpleGlyphPathSinkTailAction::CloseContour _close:
            false

fn tail_closes_contour %fn &GuiSfntSimpleGlyphPathSinkStep fn i32 bool \step\expected_contour:
    match gui_sfnt_simple_glyph_path_sink_step_tail_action step:
        GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction:
            false
        GuiSfntSimpleGlyphPathSinkTailAction::CloseContour close:
            eq expected_contour gui_sfnt_simple_glyph_path_contour_close_contour_index &close

fn action_is_reject_off_curve %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_action_at step GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
        GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkAction::Reject reason:
            match reason:
                GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                    true
        GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
            false
        GuiSfntSimpleGlyphPathSinkAction::NoAction:
            false

fn action_is_emit_off_curve %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_action_at step GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
        GuiSfntSimpleGlyphPathSinkAction::EmitEvent event:
            let command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &event
            match command:
                GuiSfntSimpleGlyphPathCommand::MoveTo _move_to:
                    false
                GuiSfntSimpleGlyphPathCommand::LineTo _line_to:
                    false
                GuiSfntSimpleGlyphPathCommand::QuadraticTo _quadratic_to:
                    false
                GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
                    let reason %GuiSfntSimpleGlyphCurveNoSegmentReason gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip
                    match reason:
                        GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                            false
                        GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                            true
                        GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                            false
        GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
            false
        GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
            false
        GuiSfntSimpleGlyphPathSinkAction::NoAction:
            false

fn action_closes_contour %fn &GuiSfntSimpleGlyphPathSinkStep fn i32 bool \step\expected_contour:
    match gui_sfnt_simple_glyph_path_sink_step_action_at step GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
        GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
            false
        GuiSfntSimpleGlyphPathSinkAction::CloseContour close:
            eq expected_contour gui_sfnt_simple_glyph_path_contour_close_contour_index &close
        GuiSfntSimpleGlyphPathSinkAction::NoAction:
            false

fn action_is_no_action %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_step_action_at step GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
        GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
            false
        GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
            false
        GuiSfntSimpleGlyphPathSinkAction::NoAction:
            true

fn event_slot_matches %fn GuiSfntSimpleGlyphPathSinkEventSlot fn GuiSfntSimpleGlyphPathSinkEventSlot bool \actual\expected:
    match actual:
        GuiSfntSimpleGlyphPathSinkEventSlot::First:
            match expected:
                GuiSfntSimpleGlyphPathSinkEventSlot::First:
                    true
                GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                    false
        GuiSfntSimpleGlyphPathSinkEventSlot::Second:
            match expected:
                GuiSfntSimpleGlyphPathSinkEventSlot::First:
                    false
                GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                    true

fn action_slot_matches %fn GuiSfntSimpleGlyphPathSinkActionSlot fn GuiSfntSimpleGlyphPathSinkActionSlot bool \actual\expected:
    match actual:
        GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
            match expected:
                GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                    true
                GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                    false
        GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
            match expected:
                GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                    false
                GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                    true

fn action_cursor_matches %fn &GuiSfntSimpleGlyphPathSinkActionCursor fn i32 fn i32 fn GuiSfntSimpleGlyphPathSinkEventSlot fn GuiSfntSimpleGlyphPathSinkActionSlot bool \cursor\expected_contour\expected_edge\expected_event_slot\expected_action_slot:
    let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor cursor
    let contour_ok %bool eq expected_contour gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
    let edge_ok %bool eq expected_edge gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
    let event_slot_ok %bool event_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor expected_event_slot
    let action_slot_ok %bool action_slot_matches gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot cursor expected_action_slot
    and contour_ok and edge_ok and event_slot_ok action_slot_ok

fn action_step_primary_next_is_tail_same_cursor %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    let action_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step step GuiSfntSimpleGlyphPathSinkActionSlot::Primary
    let source_step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_path_sink_step_source_step step
    let source_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_step_cursor &source_step
    let expected_contour %i32 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &source_cursor
    let expected_edge %i32 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &source_cursor
    let expected_event_slot %GuiSfntSimpleGlyphPathSinkEventSlot gui_sfnt_simple_glyph_path_contour_cursor_slot &source_cursor
    match gui_sfnt_simple_glyph_path_sink_action_step_next &action_step:
        GuiSfntSimpleGlyphPathSinkActionNext::Continue cursor:
            action_cursor_matches &cursor expected_contour expected_edge expected_event_slot GuiSfntSimpleGlyphPathSinkActionSlot::Tail
        GuiSfntSimpleGlyphPathSinkActionNext::EndContour:
            false

fn action_step_tail_continues_to_primary %fn &GuiSfntSimpleGlyphPathSinkStep fn i32 fn i32 fn GuiSfntSimpleGlyphPathSinkEventSlot bool \step\expected_contour\expected_edge\expected_event_slot:
    let action_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step step GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    match gui_sfnt_simple_glyph_path_sink_action_step_next &action_step:
        GuiSfntSimpleGlyphPathSinkActionNext::Continue cursor:
            action_cursor_matches &cursor expected_contour expected_edge expected_event_slot GuiSfntSimpleGlyphPathSinkActionSlot::Primary
        GuiSfntSimpleGlyphPathSinkActionNext::EndContour:
            false

fn action_step_tail_ends_contour %fn &GuiSfntSimpleGlyphPathSinkStep bool \step:
    let action_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step step GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    match gui_sfnt_simple_glyph_path_sink_action_step_next &action_step:
        GuiSfntSimpleGlyphPathSinkActionNext::Continue _cursor:
            false
        GuiSfntSimpleGlyphPathSinkActionNext::EndContour:
            true

fn action_step_advance_is_end %fn GuiSfntSimpleGlyphPathSinkActionStepAdvance bool \advance:
    match advance:
        GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue _step:
            false
        GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour:
            true

fn action_step_item_keeps_step_and_end %fn &GuiSfntSimpleGlyphPathSinkActionStepItem fn i32 fn i32 fn GuiSfntSimpleGlyphPathSinkEventSlot fn GuiSfntSimpleGlyphPathSinkActionSlot bool \item\expected_contour\expected_edge\expected_event_slot\expected_action_slot:
    let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step item
    let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &stored_step
    let step_ok %bool action_cursor_matches &action_cursor expected_contour expected_edge expected_event_slot expected_action_slot
    let advance %GuiSfntSimpleGlyphPathSinkActionStepAdvance gui_sfnt_simple_glyph_path_sink_action_step_item_advance item
    let advance_ok %bool action_step_advance_is_end advance
    and step_ok advance_ok

fn apply_status_is_emit_off_curve %fn &GuiSfntSimpleGlyphPathSinkActionApplyStatus bool \status:
    match *status:
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent event:
            let command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_sink_event_command &event
            match command:
                GuiSfntSimpleGlyphPathCommand::MoveTo _move_to:
                    false
                GuiSfntSimpleGlyphPathCommand::LineTo _line_to:
                    false
                GuiSfntSimpleGlyphPathCommand::QuadraticTo _quadratic_to:
                    false
                GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip:
                    let reason %GuiSfntSimpleGlyphCurveNoSegmentReason gui_sfnt_simple_glyph_path_skip_no_segment_reason &skip
                    match reason:
                        GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour:
                            false
                        GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart:
                            true
                        GuiSfntSimpleGlyphCurveNoSegmentReason::MissingLookahead:
                            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected _reason:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
            false

fn apply_status_is_reject_off_curve %fn &GuiSfntSimpleGlyphPathSinkActionApplyStatus bool \status:
    match *status:
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected reason:
            match reason:
                GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                    true
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
            false

fn apply_status_closes_contour %fn &GuiSfntSimpleGlyphPathSinkActionApplyStatus fn i32 bool \status\expected_contour:
    match *status:
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected _reason:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour close:
            eq expected_contour gui_sfnt_simple_glyph_path_contour_close_contour_index &close
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
            false

fn apply_status_is_no_action %fn &GuiSfntSimpleGlyphPathSinkActionApplyStatus bool \status:
    match *status:
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected _reason:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:
            false
        GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
            true

fn apply_state_counts_match %fn &GuiSfntSimpleGlyphPathSinkActionApplyState fn i32 fn i32 fn i32 fn i32 bool \state\expected_emit\expected_reject\expected_close\expected_no_action:
    let emit_ok %bool eq expected_emit gui_sfnt_simple_glyph_path_sink_action_apply_state_emitted_event_count state
    let reject_ok %bool eq expected_reject gui_sfnt_simple_glyph_path_sink_action_apply_state_reject_count state
    let close_ok %bool eq expected_close gui_sfnt_simple_glyph_path_sink_action_apply_state_close_contour_count state
    let no_action_ok %bool eq expected_no_action gui_sfnt_simple_glyph_path_sink_action_apply_state_no_action_count state
    and emit_ok and reject_ok and close_ok no_action_ok

fn consumer_apply_next_is_end %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep bool \step:
    match gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_next step:
        GuiSfntSimpleGlyphPathSinkActionItemNext::Continue _item:
            false
        GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:
            true

fn build_skip_step %fn GuiGlyphId fn i32 fn i32 fn GuiSfntSimpleGlyphCurveNoSegmentReason fn GuiSfntSimpleGlyphPathContourNext GuiSfntSimpleGlyphPathContourStep \glyph\contour\edge\reason\next:
    let cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph contour edge GuiSfntSimpleGlyphPathSinkEventSlot::Second
    let skip_payload %GuiSfntSimpleGlyphPathSkipNoSegment gui_sfnt_simple_glyph_path_skip_no_segment contour edge reason
    let skip_command %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip_payload
    let event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_command_sink_event skip_command
    let kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind &event
    gui_sfnt_simple_glyph_path_contour_step cursor event kind next

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 12
    let close_policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let reject_close_policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let continue_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 1 8 GuiSfntSimpleGlyphPathSinkEventSlot::First
    let continue_next %GuiSfntSimpleGlyphPathContourNext GuiSfntSimpleGlyphPathContourNext::Continue continue_cursor
    let end_next %GuiSfntSimpleGlyphPathContourNext GuiSfntSimpleGlyphPathContourNext::EndContour
    let off_curve_end_step %GuiSfntSimpleGlyphPathContourStep build_skip_step glyph 1 7 GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart end_next
    let off_curve_continue_step %GuiSfntSimpleGlyphPathContourStep build_skip_step glyph 1 7 GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart continue_next
    let single_point_end_step %GuiSfntSimpleGlyphPathContourStep build_skip_step glyph 1 7 GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour end_next
    let keep_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step_from_contour_step &close_policy &off_curve_end_step
    let reject_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step_from_contour_step &reject_close_policy &off_curve_end_step
    let continue_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step_from_contour_step &close_policy &off_curve_continue_step
    let single_point_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step_from_contour_step &reject_close_policy &single_point_end_step
    let keep_ok %bool and primary_is_emit_off_curve &keep_step tail_closes_contour &keep_step 1
    let reject_ok %bool and primary_is_reject_off_curve &reject_step tail_is_none &reject_step
    let continue_ok %bool and primary_is_emit_off_curve &continue_step tail_is_none &continue_step
    let single_point_ok %bool and primary_is_emit_single_point &single_point_step tail_closes_contour &single_point_step 1
    let primary_slot_ok %bool and gui_sfnt_simple_glyph_path_sink_action_slot_is_primary GuiSfntSimpleGlyphPathSinkActionSlot::Primary not gui_sfnt_simple_glyph_path_sink_action_slot_is_tail GuiSfntSimpleGlyphPathSinkActionSlot::Primary
    let tail_slot_ok %bool and gui_sfnt_simple_glyph_path_sink_action_slot_is_tail GuiSfntSimpleGlyphPathSinkActionSlot::Tail not gui_sfnt_simple_glyph_path_sink_action_slot_is_primary GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    let action_slot_ok %bool and primary_slot_ok tail_slot_ok
    let action_keep_ok %bool and action_is_emit_off_curve &keep_step action_closes_contour &keep_step 1
    let action_reject_ok %bool and action_is_reject_off_curve &reject_step action_is_no_action &reject_step
    let action_continue_ok %bool and action_is_emit_off_curve &continue_step action_is_no_action &continue_step
    let action_projection_ok %bool and action_slot_ok and action_keep_ok and action_reject_ok action_continue_ok
    let primary_traversal_ok %bool and action_step_primary_next_is_tail_same_cursor &keep_step action_step_primary_next_is_tail_same_cursor &reject_step
    let continue_tail_traversal_ok %bool action_step_tail_continues_to_primary &continue_step 1 8 GuiSfntSimpleGlyphPathSinkEventSlot::First
    let end_tail_traversal_ok %bool and action_step_tail_ends_contour &keep_step action_step_tail_ends_contour &reject_step
    let action_traversal_ok %bool and primary_traversal_ok and continue_tail_traversal_ok end_tail_traversal_ok
    let start_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph 3
    let start_cursor_ok %bool action_cursor_matches &start_cursor 3 0 GuiSfntSimpleGlyphPathSinkEventSlot::First GuiSfntSimpleGlyphPathSinkActionSlot::Primary
    let action_step_advance_ok %bool action_step_advance_is_end GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour
    let item_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_from_sink_step &keep_step GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    let action_step_item %GuiSfntSimpleGlyphPathSinkActionStepItem gui_sfnt_simple_glyph_path_sink_action_step_item item_step GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour
    let action_step_item_ok %bool action_step_item_keeps_step_and_end &action_step_item 1 7 GuiSfntSimpleGlyphPathSinkEventSlot::Second GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    let apply_state0 %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_state_new
    let emit_action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_step_action_at &keep_step GuiSfntSimpleGlyphPathSinkActionSlot::Primary
    let emit_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action apply_state0 emit_action
    let emit_apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_step_state &emit_apply_step
    let emit_apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_apply_step_status &emit_apply_step
    let emit_apply_ok %bool and apply_status_is_emit_off_curve &emit_apply_status apply_state_counts_match &emit_apply_state 1 0 0 0
    let reject_action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_step_action_at &reject_step GuiSfntSimpleGlyphPathSinkActionSlot::Primary
    let reject_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action emit_apply_state reject_action
    let reject_apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_step_state &reject_apply_step
    let reject_apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_apply_step_status &reject_apply_step
    let reject_apply_ok %bool and apply_status_is_reject_off_curve &reject_apply_status apply_state_counts_match &reject_apply_state 1 1 0 0
    let close_action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_step_action_at &keep_step GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    let close_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action reject_apply_state close_action
    let close_apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_step_state &close_apply_step
    let close_apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_apply_step_status &close_apply_step
    let close_apply_ok %bool and apply_status_closes_contour &close_apply_status 1 apply_state_counts_match &close_apply_state 1 1 1 0
    let no_action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_step_action_at &continue_step GuiSfntSimpleGlyphPathSinkActionSlot::Tail
    let no_action_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_state_apply_action close_apply_state no_action
    let no_action_apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_step_state &no_action_apply_step
    let no_action_apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_apply_step_status &no_action_apply_step
    let no_action_apply_ok %bool and apply_status_is_no_action &no_action_apply_status apply_state_counts_match &no_action_apply_state 1 1 1 1
    let action_apply_ok %bool and emit_apply_ok and reject_apply_ok and close_apply_ok no_action_apply_ok
    let apply_consumer_item %GuiSfntSimpleGlyphPathSinkActionConsumerItem gui_sfnt_simple_glyph_path_sink_action_consumer_item close_action GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let consumer_apply_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_item_apply apply_state0 &apply_consumer_item
    let consumer_inner_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step_apply_step &consumer_apply_step
    let consumer_apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_step_state &consumer_inner_apply_step
    let consumer_apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_apply_step_status &consumer_inner_apply_step
    let consumer_apply_status_ok %bool apply_status_closes_contour &consumer_apply_status 1
    let consumer_apply_count_ok %bool apply_state_counts_match &consumer_apply_state 0 0 1 0
    let consumer_apply_next_ok %bool consumer_apply_next_is_end &consumer_apply_step
    let consumer_apply_ok %bool and consumer_apply_status_ok and consumer_apply_count_ok consumer_apply_next_ok
    test_assertion_exit_code assert "path sink policy keeps reject and close tail exclusive" and keep_ok and reject_ok and continue_ok and single_point_ok and action_projection_ok and action_traversal_ok and start_cursor_ok and action_step_advance_ok and action_step_item_ok and action_apply_ok consumer_apply_ok
```

## path sink consumer apply terminal classifies domain terminal states

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
#import "core/result" as *
#import "std/test" as *

fn consumer_apply_terminal_rejects_off_curve %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal bool \terminal:
    match *terminal:
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Continue _step:
            false
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Rejected reason:
            match reason:
                GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                    true
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::EndContour _step:
            false

fn consumer_apply_terminal_ends_contour %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal bool \terminal:
    match *terminal:
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Continue _step:
            false
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Rejected _reason:
            false
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::EndContour _step:
            true

fn consumer_apply_terminal_continues %fn &GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal bool \terminal:
    match *terminal:
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Continue _step:
            true
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::Rejected _reason:
            false
        GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal::EndContour _step:
            false

fn build_action_step_item %fn GuiGlyphId GuiSfntSimpleGlyphPathSinkActionStepItem \glyph:
    let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First
    let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_cursor contour_cursor GuiSfntSimpleGlyphPathSinkActionSlot::Primary
    let skip_payload %GuiSfntSimpleGlyphPathSkipNoSegment gui_sfnt_simple_glyph_path_skip_no_segment 0 0 GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour
    let skip_command %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::SkipNoSegment skip_payload
    let event %GuiSfntSimpleGlyphPathSinkEvent gui_sfnt_simple_glyph_path_command_sink_event skip_command
    let kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind &event
    let contour_next %GuiSfntSimpleGlyphPathContourNext GuiSfntSimpleGlyphPathContourNext::EndContour
    let source_step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_path_contour_step contour_cursor event kind contour_next
    let primary_action %GuiSfntSimpleGlyphPathSinkPrimaryAction GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart
    let tail_action %GuiSfntSimpleGlyphPathSinkTailAction GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction
    let sink_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step source_step primary_action tail_action
    let action %GuiSfntSimpleGlyphPathSinkAction GuiSfntSimpleGlyphPathSinkAction::NoAction
    let action_next %GuiSfntSimpleGlyphPathSinkActionNext GuiSfntSimpleGlyphPathSinkActionNext::EndContour
    let action_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step action_cursor sink_step action action_next
    gui_sfnt_simple_glyph_path_sink_action_step_item action_step GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 12
    let item %GuiSfntSimpleGlyphPathSinkActionStepItem build_action_step_item glyph
    let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_state_new
    let no_action_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_step state GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction
    let continue_consumer_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step no_action_step GuiSfntSimpleGlyphPathSinkActionItemNext::Continue item
    let continue_terminal %GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step &continue_consumer_step
    let reject_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart
    let reject_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_step state reject_status
    let reject_consumer_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step reject_step GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let reject_terminal %GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step &reject_consumer_step
    let end_consumer_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step no_action_step GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let end_terminal %GuiSfntSimpleGlyphPathSinkActionConsumerApplyTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_apply_terminal_from_step &end_consumer_step
    let continue_ok %bool consumer_apply_terminal_continues &continue_terminal
    let reject_ok %bool consumer_apply_terminal_rejects_off_curve &reject_terminal
    let end_ok %bool consumer_apply_terminal_ends_contour &end_terminal
    let consumer_apply_terminal_ok %bool and continue_ok and reject_ok end_ok
    test_assertion_exit_code assert "path sink consumer apply terminal keeps typed domain states" consumer_apply_terminal_ok
```

## path sink consumer apply advance keeps domain terminals as ok values

neplg2:test[skip, stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "alloc/io" as *
#import "core/gui/font" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn apply_advance_rejects_off_curve %fn Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntParseError bool \result:
    match result:
        Result::Err _error:
            false
        Result::Ok advance:
            match advance:
                GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue _item:
                    false
                GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected reason:
                    match reason:
                        GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                            true
                GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour:
                    false

fn apply_advance_ends_contour %fn Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntParseError bool \result:
    match result:
        Result::Err _error:
            false
        Result::Ok advance:
            match advance:
                GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Continue _item:
                    false
                GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::Rejected _reason:
                    false
                GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance::EndContour:
                    true

fn main %impure fn void i32 \void:
    let bytes %ByteBuf io_bytebuf_empty
    let none_face %Option i32 none
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_state_new
    let reject_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart
    let reject_apply_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_step state reject_status
    let reject_consumer_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step reject_apply_step GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let reject_result %Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntParseError gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance &bytes none_face &reject_consumer_step &policy
    let no_action_step %GuiSfntSimpleGlyphPathSinkActionApplyStep gui_sfnt_simple_glyph_path_sink_action_apply_step state GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction
    let end_consumer_step %GuiSfntSimpleGlyphPathSinkActionConsumerApplyStep gui_sfnt_simple_glyph_path_sink_action_consumer_apply_step no_action_step GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let end_result %Result GuiSfntSimpleGlyphPathSinkActionConsumerApplyAdvance GuiSfntParseError gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_apply_advance &bytes none_face &end_consumer_step &policy
    let reject_ok %bool apply_advance_rejects_off_curve reject_result
    let end_ok %bool apply_advance_ends_contour end_result
    let apply_advance_ok %bool if reject_ok end_ok false
    test_assertion_exit_code assert "path sink consumer apply advance keeps domain terminals as ok values" apply_advance_ok
```

## path sink consumer consume once preserves apply result and advance

neplg2:test[skip, stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "alloc/io" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn consume_once_reject_ok %fn Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError bool \result:
    match result:
        Result::Err _error:
            false
        Result::Ok consume_step:
            let summary %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step
            let apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state &summary
            let apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status &summary
            let status_ok %bool match apply_status:
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:
                    false
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected reason:
                    match reason:
                        GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                            true
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:
                    false
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
                    false
            let count_ok %bool eq 1 gui_sfnt_simple_glyph_path_sink_action_apply_state_reject_count &apply_state
            let terminal %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal &summary
            let terminal_ok %bool match terminal:
                GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue _item:
                    false
                GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected reason:
                    match reason:
                        GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart:
                            true
                GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour:
                    false
            and status_ok and count_ok terminal_ok

fn consume_once_end_ok %fn Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError bool \result:
    match result:
        Result::Err _error:
            false
        Result::Ok consume_step:
            let summary %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step
            let apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state &summary
            let apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status &summary
            let status_ok %bool match apply_status:
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:
                    false
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected _reason:
                    false
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:
                    false
                GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
                    true
            let count_ok %bool eq 1 gui_sfnt_simple_glyph_path_sink_action_apply_state_no_action_count &apply_state
            let terminal %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal &summary
            let terminal_ok %bool match terminal:
                GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue _item:
                    false
                GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected _reason:
                    false
                GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour:
                    true
            and status_ok and count_ok terminal_ok

fn main %impure fn void i32 \void:
    let bytes %ByteBuf io_bytebuf_empty
    let none_face %Option i32 none
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_state_new
    let reject_action %GuiSfntSimpleGlyphPathSinkAction GuiSfntSimpleGlyphPathSinkAction::Reject GuiSfntSimpleGlyphPathSinkRejectReason::UnsupportedOffCurveStart
    let reject_item %GuiSfntSimpleGlyphPathSinkActionConsumerItem gui_sfnt_simple_glyph_path_sink_action_consumer_item reject_action GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let reject_result %Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once &bytes none_face state &reject_item &policy
    let no_action_item %GuiSfntSimpleGlyphPathSinkActionConsumerItem gui_sfnt_simple_glyph_path_sink_action_consumer_item GuiSfntSimpleGlyphPathSinkAction::NoAction GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
    let end_result %Result GuiSfntSimpleGlyphPathSinkActionConsumerConsumeStep GuiSfntParseError gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_consume_once &bytes none_face state &no_action_item &policy
    let reject_ok %bool consume_once_reject_ok reject_result
    let end_ok %bool consume_once_end_ok end_result
    let consume_once_ok %bool and reject_ok end_ok
    test_assertion_exit_code assert "path sink consumer consume once preserves apply result and advance" consume_once_ok
```

## path contour step public lookup follows cursor next contract

neplg2:test[skip, stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "alloc/gui/font/sfnt/metadata" as *
#import "alloc/io" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn build_sfnt_step_fixture %impure fn void Result ByteBuf str \void:
    match io_bytebuf_from_str_result "\x00\x01\x00\x00\x00\x05\x00\x00\x00\x00\x00\x00\x68\x65\x61\x64\x00\x00\x00\x00\x00\x00\x00\x5c\x00\x00\x00\x34\x68\x68\x65\x61\x00\x00\x00\x00\x00\x00\x00\x90\x00\x00\x00\x24\x6d\x61\x78\x70\x00\x00\x00\x00\x00\x00\x00\xb4\x00\x00\x00\x06\x6c\x6f\x63\x61\x00\x00\x00\x00\x00\x00\x00\xba\x00\x00\x00\x08\x67\x6c\x79\x66\x00\x00\x00\x00\x00\x00\x00\xc2\x00\x00\x00\x14\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x01\x00\x00\x00\x03\x00\x00\x00\x00\x00\x0a\x00\x0a\x00\x01\x00\x00\x00\x00\x00\x14\x00\x14\x00\x02\x00\x00\x31\x33\x35\x0a\x0a\x00":
        Result::Err _error:
            Result::Err "fixture bytes"
        Result::Ok bytes:
            Result::Ok bytes

fn sfnt_step_slot_matches %fn GuiSfntSimpleGlyphPathSinkEventSlot fn GuiSfntSimpleGlyphPathSinkEventSlot bool \actual\expected:
    match actual:
        GuiSfntSimpleGlyphPathSinkEventSlot::First:
            match expected:
                GuiSfntSimpleGlyphPathSinkEventSlot::First:
                    true
                GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                    false
        GuiSfntSimpleGlyphPathSinkEventSlot::Second:
            match expected:
                GuiSfntSimpleGlyphPathSinkEventSlot::First:
                    false
                GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                    true

fn sfnt_step_next_is_edge_slot %fn &GuiSfntSimpleGlyphPathContourStep fn i32 fn GuiSfntSimpleGlyphPathSinkEventSlot bool \step\expected_edge\expected_slot:
    let next %GuiSfntSimpleGlyphPathContourNext gui_sfnt_simple_glyph_path_contour_step_next step
    match next:
        GuiSfntSimpleGlyphPathContourNext::Continue cursor:
            let edge_ok %bool eq expected_edge gui_sfnt_simple_glyph_path_contour_cursor_edge_index &cursor
            let slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &cursor expected_slot
            and edge_ok slot_ok
        GuiSfntSimpleGlyphPathContourNext::EndContour:
            false

fn sfnt_step_next_is_end %fn &GuiSfntSimpleGlyphPathContourStep bool \step:
    let next %GuiSfntSimpleGlyphPathContourNext gui_sfnt_simple_glyph_path_contour_step_next step
    match next:
        GuiSfntSimpleGlyphPathContourNext::Continue _cursor:
            false
        GuiSfntSimpleGlyphPathContourNext::EndContour:
            true

fn sfnt_step_kind_is_move %fn &GuiSfntSimpleGlyphPathContourStep bool \step:
    match gui_sfnt_simple_glyph_path_contour_step_kind step:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment _reason:
            false

fn sfnt_step_kind_is_line %fn &GuiSfntSimpleGlyphPathContourStep bool \step:
    match gui_sfnt_simple_glyph_path_contour_step_kind step:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            true
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo:
            false
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment _reason:
            false

fn sfnt_step_result_is_missing_outline %fn Result GuiSfntSimpleGlyphPathContourStep GuiSfntParseError bool \result:
    match result:
        Result::Ok _step:
            false
        Result::Err error:
            match gui_sfnt_parse_error_kind &error:
                GuiSfntParseErrorKind::MissingGlyphOutline:
                    true
                _:
                    false

fn sfnt_action_slot_is_primary %fn GuiSfntSimpleGlyphPathSinkActionSlot bool \slot:
    match slot:
        GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
            true
        GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
            false

fn main %impure fn void i32 \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    match build_sfnt_step_fixture:
        Result::Err _message:
            test_assertion_exit_code assert "sfnt fixture builds" false
        Result::Ok bytes:
            let first_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First
            let second_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::Second
            let final_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 2 GuiSfntSimpleGlyphPathSinkEventSlot::Second
            let out_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 3 GuiSfntSimpleGlyphPathSinkEventSlot::First
            let sink_policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
            let first_ok %bool match gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none first_cursor:
                Result::Err _error:
                    false
                Result::Ok step:
                    let kind_ok %bool sfnt_step_kind_is_move &step
                    let next_ok %bool sfnt_step_next_is_edge_slot &step 0 GuiSfntSimpleGlyphPathSinkEventSlot::Second
                    and kind_ok next_ok
            let second_ok %bool match gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none second_cursor:
                Result::Err _error:
                    false
                Result::Ok step:
                    let kind_ok %bool sfnt_step_kind_is_line &step
                    let next_ok %bool sfnt_step_next_is_edge_slot &step 1 GuiSfntSimpleGlyphPathSinkEventSlot::First
                    and kind_ok next_ok
            let final_ok %bool match gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none final_cursor:
                Result::Err _error:
                    false
                Result::Ok step:
                    let kind_ok %bool sfnt_step_kind_is_line &step
                    let next_ok %bool sfnt_step_next_is_end &step
                    and kind_ok next_ok
            let out_ok %bool sfnt_step_result_is_missing_outline gui_sfnt_lookup_simple_glyph_path_contour_step &bytes none out_cursor
            let sink_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_step &bytes none final_cursor &sink_policy:
                Result::Err _error:
                    false
                Result::Ok sink_step:
                    match gui_sfnt_simple_glyph_path_sink_step_tail_action &sink_step:
                        GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction:
                            false
                        GuiSfntSimpleGlyphPathSinkTailAction::CloseContour close:
                            eq 0 gui_sfnt_simple_glyph_path_contour_close_contour_index &close
            let start_step_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_step &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok action_step:
                    let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &action_step
                    let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &action_cursor
                    let contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
                    let edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
                    let event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor GuiSfntSimpleGlyphPathSinkEventSlot::First
                    let action_slot_ok %bool sfnt_action_slot_is_primary gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &action_cursor
                    and contour_ok and edge_ok and event_slot_ok action_slot_ok
            let start_advance_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_step &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok action_step:
                    match gui_sfnt_lookup_simple_glyph_path_sink_action_step_advance &bytes none &action_step &sink_policy:
                        Result::Err _error:
                            false
                        Result::Ok advance:
                            match advance:
                                GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step:
                                    let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &next_step
                                    let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &action_cursor
                                    let contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
                                    let edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
                                    let event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor GuiSfntSimpleGlyphPathSinkEventSlot::First
                                    let action_slot_ok %bool match gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &action_cursor:
                                        GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                                            false
                                        GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                                            true
                                    and contour_ok and edge_ok and event_slot_ok action_slot_ok
                                GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour:
                                    false
            let start_item_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok item:
                    let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step &item
                    let stored_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &stored_step
                    let stored_contour %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &stored_cursor
                    let stored_contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &stored_contour
                    let stored_edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &stored_contour
                    let stored_event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &stored_contour GuiSfntSimpleGlyphPathSinkEventSlot::First
                    let stored_action_slot_ok %bool sfnt_action_slot_is_primary gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &stored_cursor
                    let advance %GuiSfntSimpleGlyphPathSinkActionStepAdvance gui_sfnt_simple_glyph_path_sink_action_step_item_advance &item
                    let advance_ok %bool match advance:
                        GuiSfntSimpleGlyphPathSinkActionStepAdvance::Continue next_step:
                            let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &next_step
                            let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &action_cursor
                            let contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
                            let edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
                            let event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor GuiSfntSimpleGlyphPathSinkEventSlot::First
                            let action_slot_ok %bool match gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &action_cursor:
                                GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                                    false
                                GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                                    true
                            and contour_ok and edge_ok and event_slot_ok action_slot_ok
                        GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour:
                            false
                    and stored_contour_ok and stored_edge_ok and stored_event_slot_ok and stored_action_slot_ok advance_ok
            let terminal_item_next_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_step &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok action_step:
                    let terminal_item %GuiSfntSimpleGlyphPathSinkActionStepItem gui_sfnt_simple_glyph_path_sink_action_step_item action_step GuiSfntSimpleGlyphPathSinkActionStepAdvance::EndContour
                    match gui_sfnt_lookup_simple_glyph_path_sink_action_item_next &bytes none &terminal_item &sink_policy:
                        Result::Err _error:
                            false
                        Result::Ok item_next:
                            match item_next:
                                GuiSfntSimpleGlyphPathSinkActionItemNext::Continue _next_item:
                                    false
                                GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:
                                    true
            let start_item_next_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok item:
                    match gui_sfnt_lookup_simple_glyph_path_sink_action_item_next &bytes none &item &sink_policy:
                        Result::Err _error:
                            false
                        Result::Ok item_next:
                            match item_next:
                                GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item:
                                    let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step &next_item
                                    let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &stored_step
                                    let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &action_cursor
                                    let contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
                                    let edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
                                    let event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor GuiSfntSimpleGlyphPathSinkEventSlot::First
                                    let action_slot_ok %bool match gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &action_cursor:
                                        GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                                            false
                                        GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                                            true
                                    and contour_ok and edge_ok and event_slot_ok action_slot_ok
                                GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:
                                    false
            let start_consumer_item_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok item:
                    match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item &bytes none &item &sink_policy:
                        Result::Err _error:
                            false
                        Result::Ok consumer:
                            let action_ok %bool match gui_sfnt_simple_glyph_path_sink_action_consumer_item_action &consumer:
                                GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
                                    true
                                GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
                                    false
                                GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
                                    false
                                GuiSfntSimpleGlyphPathSinkAction::NoAction:
                                    false
                            let next_ok %bool match gui_sfnt_simple_glyph_path_sink_action_consumer_item_next &consumer:
                                GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item:
                                    let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step &next_item
                                    let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &stored_step
                                    let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &action_cursor
                                    let contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
                                    let edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
                                    let event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor GuiSfntSimpleGlyphPathSinkEventSlot::First
                                    let action_slot_ok %bool match gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &action_cursor:
                                        GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                                            false
                                        GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                                            true
                                    and contour_ok and edge_ok and event_slot_ok action_slot_ok
                                GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:
                                    false
                            and action_ok next_ok
            let start_consumer_item_direct_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_consumer_item &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok consumer:
                    let action_ok %bool match gui_sfnt_simple_glyph_path_sink_action_consumer_item_action &consumer:
                        GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
                            true
                        GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
                            false
                        GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
                            false
                        GuiSfntSimpleGlyphPathSinkAction::NoAction:
                            false
                    let next_ok %bool match gui_sfnt_simple_glyph_path_sink_action_consumer_item_next &consumer:
                        GuiSfntSimpleGlyphPathSinkActionItemNext::Continue next_item:
                            let stored_step %GuiSfntSimpleGlyphPathSinkActionStep gui_sfnt_simple_glyph_path_sink_action_step_item_step &next_item
                            let action_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_step_cursor &stored_step
                            let contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &action_cursor
                            let contour_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_contour_index &contour_cursor
                            let edge_ok %bool eq 0 gui_sfnt_simple_glyph_path_contour_cursor_edge_index &contour_cursor
                            let event_slot_ok %bool sfnt_step_slot_matches gui_sfnt_simple_glyph_path_contour_cursor_slot &contour_cursor GuiSfntSimpleGlyphPathSinkEventSlot::First
                            let action_slot_ok %bool match gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &action_cursor:
                                GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                                    false
                                GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                                    true
                            and contour_ok and edge_ok and event_slot_ok action_slot_ok
                        GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour:
                            false
                    and action_ok next_ok
            let start_consume_once_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_apply_state_new
            let start_consume_once_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_consume_once &bytes none start_consume_once_state glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok consume_step:
                    let summary %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummary gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_from_step &consume_step
                    let apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_state &summary
                    let apply_status %GuiSfntSimpleGlyphPathSinkActionApplyStatus gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_status &summary
                    let status_ok %bool match apply_status:
                        GuiSfntSimpleGlyphPathSinkActionApplyStatus::EmittedEvent _event:
                            true
                        GuiSfntSimpleGlyphPathSinkActionApplyStatus::Rejected _reason:
                            false
                        GuiSfntSimpleGlyphPathSinkActionApplyStatus::ClosedContour _close:
                            false
                        GuiSfntSimpleGlyphPathSinkActionApplyStatus::NoAction:
                            false
                    let count_ok %bool eq 1 gui_sfnt_simple_glyph_path_sink_action_apply_state_emitted_event_count &apply_state
                    let terminal %GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal gui_sfnt_simple_glyph_path_sink_action_consumer_consume_summary_terminal &summary
                    let terminal_ok %bool match terminal:
                        GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Continue next_consumer:
                            match gui_sfnt_simple_glyph_path_sink_action_consumer_item_action &next_consumer:
                                GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
                                    false
                                GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
                                    false
                                GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
                                    false
                                GuiSfntSimpleGlyphPathSinkAction::NoAction:
                                    true
                        GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::Rejected _reason:
                            false
                        GuiSfntSimpleGlyphPathSinkActionConsumerConsumeSummaryTerminal::EndContour:
                            false
                    and status_ok and count_ok terminal_ok
            let terminal_consumer_item_next_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok item:
                    match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item &bytes none &item &sink_policy:
                        Result::Err _error:
                            false
                        Result::Ok consumer:
                            let action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_action_consumer_item_action &consumer
                            let terminal_consumer %GuiSfntSimpleGlyphPathSinkActionConsumerItem gui_sfnt_simple_glyph_path_sink_action_consumer_item action GuiSfntSimpleGlyphPathSinkActionItemNext::EndContour
                            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next &bytes none &terminal_consumer &sink_policy:
                                Result::Err _error:
                                    false
                                Result::Ok consumer_next:
                                    match consumer_next:
                                        GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue _next_consumer:
                                            false
                                        GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour:
                                            true
            let start_consumer_item_next_ok %bool match gui_sfnt_lookup_simple_glyph_path_sink_action_start_item &bytes none glyph 0 &sink_policy:
                Result::Err _error:
                    false
                Result::Ok item:
                    match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item &bytes none &item &sink_policy:
                        Result::Err _error:
                            false
                        Result::Ok consumer:
                            match gui_sfnt_lookup_simple_glyph_path_sink_action_consumer_item_next &bytes none &consumer &sink_policy:
                                Result::Err _error:
                                    false
                                Result::Ok consumer_next:
                                    match consumer_next:
                                        GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::Continue next_consumer:
                                            match gui_sfnt_simple_glyph_path_sink_action_consumer_item_action &next_consumer:
                                                GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
                                                    false
                                                GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
                                                    false
                                                GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
                                                    false
                                                GuiSfntSimpleGlyphPathSinkAction::NoAction:
                                                    true
                                        GuiSfntSimpleGlyphPathSinkActionConsumerItemNext::EndContour:
                                            false
            io_bytebuf_free bytes
            test_assertion_exit_code assert "path contour step public lookup follows cursor next contract" and first_ok and second_ok and final_ok and out_ok and sink_ok and start_step_ok and start_advance_ok and start_item_ok and terminal_item_next_ok and start_item_next_ok and start_consumer_item_ok and start_consumer_item_direct_ok and start_consume_once_ok and terminal_consumer_item_next_ok start_consumer_item_next_ok
```
