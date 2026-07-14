# GUI font SFNT glyf outline point stream item collection render stroke source segment metric owner

このファイルは、F5ku の render stroke source segment metric owner boundary が F5ks fresh cursor を消費し、source segment metric Vec owner だけを作り、stroke offset geometry / stroke edge / coverage / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_source_segment_metric_owner_fresh_cursor_ok
- render_stroke_source_segment_metric_owner_exact_capacity_ok
- render_stroke_source_segment_metric_owner_line_push_ok
- render_stroke_source_segment_metric_owner_quadratic_push_ok
- render_stroke_source_segment_metric_owner_completion_counts_ok
- render_stroke_source_segment_metric_owner_push_recovery_ok
- render_stroke_source_segment_metric_owner_metric_error_context_ok
- render_stroke_source_segment_metric_owner_no_reread_edge_mask_command_platform

## point stream item collection render stroke source segment metric checkpoint runtime

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/cast" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

// render_stroke_source_segment_metric_owner_fresh_cursor_ok
// render_stroke_source_segment_metric_owner_exact_capacity_ok
// render_stroke_source_segment_metric_owner_line_push_ok
// render_stroke_source_segment_metric_owner_quadratic_push_ok
// render_stroke_source_segment_metric_owner_completion_counts_ok
// render_stroke_source_segment_metric_owner_push_recovery_ok
// render_stroke_source_segment_metric_owner_metric_error_context_ok
// render_stroke_source_segment_metric_owner_no_reread_edge_mask_command_platform

fn path_command_value %fn i32 fn i32 fn i32 fn GuiSfntSimpleGlyphPathSinkEventSlot fn GuiSfntSimpleGlyphPathCommandTag fn GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValue \path_command_index\edge_index\contour_edge_index\event_slot\tag\command:
    gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value path_command_index edge_index 0 contour_edge_index event_slot tag tag command

fn push_command %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandValue GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner \writer\value:
    unwrap_ok gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_sink_writer_owner_push_value_continue writer value

fn checkpoint_read_error_is_out_of_range %fn Result GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricCheckpointReadErrorKind bool \result:
    match result:
        Result::Err kind:
            match kind:
                GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricCheckpointReadErrorKind::IndexOutOfRange: true
                _: false
        Result::Ok _projection: false

fn main %impure fn void i32 \void:
    let plan %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkPlan gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_sink_plan 6 6 2 2 1 1 2 4 2 5
    let owner %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkOwner unwrap_ok gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_sink_owner_alloc &plan
    let writer0 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner unwrap_ok gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_sink_writer_owner_start owner
    let move0 %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::MoveTo gui_sfnt_simple_glyph_path_move_to 0 0 0 0
    let quadratic0 %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::QuadraticTo gui_sfnt_simple_glyph_path_quadratic_to 0 0 10 14 16 8 false
    let skip1a %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::SkipNoSegment gui_sfnt_simple_glyph_path_skip_no_segment 0 1 GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart
    let skip1b %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::SkipNoSegment gui_sfnt_simple_glyph_path_skip_no_segment 0 1 GuiSfntSimpleGlyphCurveNoSegmentReason::OffCurveStart
    let move2 %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::MoveTo gui_sfnt_simple_glyph_path_move_to 0 2 16 8
    let line2 %GuiSfntSimpleGlyphPathCommand GuiSfntSimpleGlyphPathCommand::LineTo gui_sfnt_simple_glyph_path_line_to 0 2 0 0
    let writer1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner push_command writer0 path_command_value 0 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First GuiSfntSimpleGlyphPathCommandTag::MoveTo move0
    let writer2 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner push_command writer1 path_command_value 1 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::Second GuiSfntSimpleGlyphPathCommandTag::QuadraticTo quadratic0
    let writer3 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner push_command writer2 path_command_value 2 1 1 GuiSfntSimpleGlyphPathSinkEventSlot::First GuiSfntSimpleGlyphPathCommandTag::SkipNoSegment skip1a
    let writer4 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner push_command writer3 path_command_value 3 1 1 GuiSfntSimpleGlyphPathSinkEventSlot::Second GuiSfntSimpleGlyphPathCommandTag::SkipNoSegment skip1b
    let writer5 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner push_command writer4 path_command_value 4 2 2 GuiSfntSimpleGlyphPathSinkEventSlot::First GuiSfntSimpleGlyphPathCommandTag::MoveTo move2
    let writer6 %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathCommandStreamSinkWriterOwner push_command writer5 path_command_value 5 2 2 GuiSfntSimpleGlyphPathSinkEventSlot::Second GuiSfntSimpleGlyphPathCommandTag::LineTo line2
    let full %u8 cast 255
    let zero %u8 cast 0
    let color %Rgba8888 rgba8888_new full zero zero full
    let stroke %GuiStroke unwrap_ok gui_stroke_new color 4 GuiStrokeCap::Butt GuiStrokeJoin::Miter 4.0 GuiStrokeDash::Solid
    let fill %Option GuiPaint none
    let stroke_option %Option GuiStroke some stroke
    let paint %GuiGlyphPaint unwrap_ok gui_glyph_paint_result fill stroke_option gui_shadow_ref_none GuiBlendMode::SourceOver
    let checkpoint %GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricCheckpointOwner unwrap_ok gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_build writer6 gui_point_new 0 0 paint
    let summary %GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricCheckpointSummary unwrap_ok gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_summary &checkpoint
    let summary_ok %bool and:
        eq gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_summary_total &summary 2
        and:
            eq gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_summary_line &summary 1
            and:
                eq gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_summary_quadratic &summary 1
                and:
                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_summary_len &summary 2
                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_summary_cap &summary 2
    let first %GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection unwrap_ok gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_read_at &checkpoint 0
    let first_ok %bool match first:
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Quadratic metric:
            and:
                eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_segment_index &metric 0
                and:
                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_x2 &metric 0
                    and:
                        eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_y2 &metric 0
                        and:
                            eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_x2 &metric 10
                            and:
                                eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_y2 &metric 14
                                and:
                                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_end_x2 &metric 16
                                    and:
                                        eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_end_y2 &metric 8
                                        and:
                                            eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_stroke_width &metric 4
                                            and:
                                                eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_control_dx &metric %i64 cast 10
                                                and:
                                                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_control_dy &metric %i64 cast 14
                                                    and:
                                                        eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_end_dx &metric %i64 cast 6
                                                        and:
                                                            eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_end_dy &metric %i64 cast -6
                                                            and:
                                                                eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_control_length_squared &metric %i64 cast 296
                                                                eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_end_length_squared &metric %i64 cast 72
        _: false
    let second %GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection unwrap_ok gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_read_at &checkpoint 1
    let second_ok %bool match second:
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Line metric:
            and:
                eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_segment_index &metric 1
                and:
                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_start_x2 &metric 16
                    and:
                        eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_start_y2 &metric 8
                        and:
                            eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_end_x2 &metric 0
                            and:
                                eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_end_y2 &metric 0
                                and:
                                    eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_stroke_width &metric 4
                                    and:
                                        eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_dx &metric %i64 cast -16
                                        and:
                                            eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_dy &metric %i64 cast -8
                                            eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_length_squared &metric %i64 cast 320
        _: false
    let low_error_ok %bool checkpoint_read_error_is_out_of_range gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_read_at &checkpoint -1
    let high_error_ok %bool checkpoint_read_error_is_out_of_range gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_read_at &checkpoint 2
    let count_ok %bool eq gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_metric_count &checkpoint 2
    let all_groups %bool and summary_ok and first_ok and second_ok and low_error_ok and high_error_ok count_ok
    gui_sfnt_simple_glyph_render_stroke_source_segment_metric_checkpoint_owner_free checkpoint
    test_assertion_exit_code assert "production checkpoint preserves quadratic and line metrics" all_groups
```
