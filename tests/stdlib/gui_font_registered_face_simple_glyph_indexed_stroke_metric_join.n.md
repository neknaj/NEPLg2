# GUI font registered simple glyph indexed stroke metric join

このファイルは、固定 SFNT bytes を registered face owner に登録し、indexed outline、path command stream、sink、stroke metric provenance を production API だけで構築する。

## registered bytes reach indexed contour spans

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_metric_join\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered bytes produce sealed stroke metric provenance\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font" as *
#import "alloc/io" as *
#import "core/cast" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
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

fn join_push_table_records %impure fn ByteBuilder Result ByteBuilder str \builder:
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
                                            match sfnt_push_record b5 sfnt_tag4 'l' 'o' 'c' 'a' 344 82:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    sfnt_push_record b6 sfnt_tag4 'g' 'l' 'y' 'f' 426 24

fn join_push_loca_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_zero_run builder 74:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 12:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 12:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u16_be b3 12:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    sfnt_push_u16_be b4 12

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

fn join_push_simple_glyph_coordinates %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_u8 builder 10:
        Result::Err message: Result::Err message
        Result::Ok b1:
            match sfnt_push_u8 b1 5:
                Result::Err message: Result::Err message
                Result::Ok b2:
                    match sfnt_push_u8 b2 5:
                        Result::Err message: Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 5:
                                Result::Err message: Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 5:
                                        Result::Err message: Result::Err message
                                        Result::Ok b5: sfnt_push_u8 b5 0

fn join_push_simple_glyph %impure fn ByteBuilder Result ByteBuilder str \builder:
    match sfnt_push_glyf_header builder 1 0 0 20 5:
        Result::Err message:
            Result::Err message
        Result::Ok b1:
            match sfnt_push_u16_be b1 3:
                Result::Err message:
                    Result::Err message
                Result::Ok b2:
                    match sfnt_push_u16_be b2 0:
                        Result::Err message:
                            Result::Err message
                        Result::Ok b3:
                            match sfnt_push_u8 b3 49:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok b4:
                                    match sfnt_push_u8 b4 51:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok b5:
                                            match sfnt_push_u8 b5 54:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok b6:
                                                    match sfnt_push_u8 b6 23:
                                                        Result::Err message:
                                                            Result::Err message
                                                        Result::Ok b7:
                                                            join_push_simple_glyph_coordinates b7

fn join_push_glyf_table %impure fn ByteBuilder Result ByteBuilder str \builder:
    join_push_simple_glyph builder

fn join_push_table_data %impure fn ByteBuilder Result ByteBuilder str \builder:
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
                                                                                                                    match join_push_loca_table b14:
                                                                                                                        Result::Err message:
                                                                                                                            Result::Err message
                                                                                                                        Result::Ok b15:
                                                                                                                            join_push_glyf_table b15

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

fn join_fixture_bytes %impure fn void Result ByteBuf str \void:
    match byte_builder_with_capacity 450:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            sfnt_finish:
                match sfnt_push_header b0 7:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match join_push_table_records b1:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                join_push_table_data b2

fn join_path_command_tag_slot_ok %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathSinkActionPathCommandTagSlot fn i32 bool \slot\index:
    let edge_index %i32 div_s index 2
    let event_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_event_slot slot:
        GuiSfntSimpleGlyphPathSinkEventSlot::First: eq rem_s index 2 0
        GuiSfntSimpleGlyphPathSinkEventSlot::Second: eq rem_s index 2 1
    let tag_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_tag slot:
        GuiSfntSimpleGlyphPathCommandTag::MoveTo: or or eq index 0 eq index 2 eq index 6
        GuiSfntSimpleGlyphPathCommandTag::LineTo: or eq index 1 eq index 7
        GuiSfntSimpleGlyphPathCommandTag::QuadraticTo: eq index 3
        GuiSfntSimpleGlyphPathCommandTag::SkipNoSegment: or eq index 4 eq index 5
    let expected_scalar %i32 match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_tag slot:
        GuiSfntSimpleGlyphPathCommandTag::MoveTo: 1
        GuiSfntSimpleGlyphPathCommandTag::LineTo: 2
        GuiSfntSimpleGlyphPathCommandTag::QuadraticTo: 3
        GuiSfntSimpleGlyphPathCommandTag::SkipNoSegment: 4
    and eq index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_path_command_index slot and eq edge_index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_edge_index slot and eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_contour_index slot and eq edge_index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_contour_edge_index slot and eq expected_scalar gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_tag_slot_scalar_value slot and event_ok tag_ok

fn join_stroke_metric_provenance_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricCompletedOwner fn i32 fn i32 fn i32 bool \completed\metric_index\command_index\tag_scalar:
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_read_provenance completed metric_index:
        Result::Err _error: false
        Result::Ok provenance:
            let command gui_font_registered_face_simple_glyph_indexed_stroke_metric_provenance_command &provenance
            let value gui_font_registered_face_simple_glyph_indexed_stroke_source_contour_command_value &command
            and eq metric_index gui_font_registered_face_simple_glyph_indexed_stroke_metric_provenance_metric_index &provenance and eq command_index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_path_command_index &value eq tag_scalar gui_sfnt_simple_glyph_path_command_tag_scalar_value &(gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_stored_tag &value)

fn join_stroke_metric_joined_provenance_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoined fn i32 fn i32 fn i32 bool \joined\metric_index\command_index\tag_scalar:
    let provenance gui_font_registered_face_simple_glyph_indexed_stroke_metric_joined_provenance joined
    let command gui_font_registered_face_simple_glyph_indexed_stroke_metric_provenance_command &provenance
    let value gui_font_registered_face_simple_glyph_indexed_stroke_source_contour_command_value &command
    and eq metric_index gui_font_registered_face_simple_glyph_indexed_stroke_metric_provenance_metric_index &provenance and eq command_index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_path_command_index &value eq tag_scalar gui_sfnt_simple_glyph_path_command_tag_scalar_value &(gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_stored_tag &value)

fn join_stroke_metric_joined_first_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoined bool \joined:
    let actual gui_font_registered_face_simple_glyph_indexed_stroke_metric_joined_actual joined
    match actual:
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Quadratic _quadratic: false
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Line line:
            and join_stroke_metric_joined_provenance_ok joined 0 1 2 and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_segment_index &line and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_start_x2 &line and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_start_y2 &line and eq 20 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_end_x2 &line and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_end_y2 &line and eq 4 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_stroke_width &line and eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_dx &line %i64 cast 20 and eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_dy &line %i64 cast 0 eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_length_squared &line %i64 cast 400

fn join_stroke_metric_joined_second_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoined bool \joined:
    let actual gui_font_registered_face_simple_glyph_indexed_stroke_metric_joined_actual joined
    match actual:
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Line _line: false
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Quadratic quadratic:
            and join_stroke_metric_joined_provenance_ok joined 1 3 3 and eq 1 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_segment_index &quadratic and eq 20 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_x2 &quadratic and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_y2 &quadratic and eq 30 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_x2 &quadratic and eq 10 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_y2 &quadratic and eq 40 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_end_x2 &quadratic and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_end_y2 &quadratic and eq 4 gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_stroke_width &quadratic and eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_control_dx &quadratic %i64 cast 10 and eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_control_dy &quadratic %i64 cast 10 and eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_end_dx &quadratic %i64 cast 10 and eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_end_dy &quadratic %i64 cast -10 and eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_start_control_length_squared &quadratic %i64 cast 200 eq gui_sfnt_simple_glyph_render_stroke_source_segment_quadratic_metric_projection_control_end_length_squared &quadratic %i64 cast 200

fn join_stroke_metric_joined_third_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoined bool \joined:
    let actual gui_font_registered_face_simple_glyph_indexed_stroke_metric_joined_actual joined
    match actual:
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Quadratic _quadratic: false
        GuiSfntSimpleGlyphRenderStrokeSourceSegmentMetricProjection::Line line:
            and join_stroke_metric_joined_provenance_ok joined 2 7 2 and eq 2 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_segment_index &line and eq 40 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_start_x2 &line and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_start_y2 &line and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_end_x2 &line and eq 0 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_end_y2 &line and eq 4 gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_stroke_width &line and eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_dx &line %i64 cast -40 and eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_dy &line %i64 cast 0 eq gui_sfnt_simple_glyph_render_stroke_source_segment_line_metric_projection_length_squared &line %i64 cast 1600

fn join_stroke_metric_joined_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoined fn i32 bool \joined\metric_index:
    if eq metric_index 0:
        then join_stroke_metric_joined_first_ok joined
        else if eq metric_index 1:
            then join_stroke_metric_joined_second_ok joined
            else join_stroke_metric_joined_third_ok joined

fn join_stroke_metric_join_step_record_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStep fn i32 bool \step\metric_index:
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_status step:
        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStatusKind::Joined:
            match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_joined step:
                Option::None: false
                Option::Some record: join_stroke_metric_joined_ok &record metric_index
        _: false

fn join_stroke_metric_join_step_terminal_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStep bool \step:
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_status step:
        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStatusKind::Completed:
            match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_joined step:
                Option::None: true
                Option::Some _record: false
        _: false

fn join_stroke_metric_join_step_budget_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStep bool \step:
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_status step:
        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStatusKind::StepBudgetExhausted:
            match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_joined step:
                Option::None: true
                Option::Some _record: false
        _: false

fn join_stroke_metric_join_run %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinOwner bool \owner:
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step owner 0:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_owner error
            false
        Result::Ok budget_step:
            let budget_ok %bool join_stroke_metric_join_step_budget_ok &budget_step
            let budget_owner gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_owner budget_step
            let budget_cursor_ok %bool eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &budget_owner
            let forced gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_test_force_metric_index_mismatch budget_owner
            let mismatch_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_kind &forced:
                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStepErrorKind::MetricIndexMismatch: true
                _: false
            let recovered gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_owner forced
            let recovery_cursor_ok %bool eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &recovered
            match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step recovered 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_owner error
                    false
                Result::Ok first_step:
                    let first_ok %bool join_stroke_metric_join_step_record_ok &first_step 0
                    let first_owner gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_owner first_step
                    let first_cursor_ok %bool eq 1 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &first_owner
                    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step first_owner 1:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_owner error
                            false
                        Result::Ok second_step:
                            let second_ok %bool join_stroke_metric_join_step_record_ok &second_step 1
                            let second_owner gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_owner second_step
                            let second_cursor_ok %bool eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &second_owner
                            match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step second_owner 1:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_owner error
                                    false
                                Result::Ok third_step:
                                    let third_ok %bool join_stroke_metric_join_step_record_ok &third_step 2
                                    let third_owner gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_owner third_step
                                    let third_cursor_ok %bool eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &third_owner
                                    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step third_owner 0:
                                        Result::Err error:
                                            gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_error_owner error
                                            false
                                        Result::Ok terminal_step:
                                            let terminal_ok %bool join_stroke_metric_join_step_terminal_ok &terminal_step
                                            let terminal_owner gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_step_owner terminal_step
                                            let terminal_cursor_ok %bool eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &terminal_owner
                                            gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_free terminal_owner
                                            and budget_ok and budget_cursor_ok and mismatch_ok and recovery_cursor_ok and first_ok and first_cursor_ok and second_ok and second_cursor_ok and third_ok and third_cursor_ok and terminal_ok terminal_cursor_ok

fn join_stroke_offset_geometry_ok %fn &GuiSfntSimpleGlyphRenderOffsetGeometryProjection fn i32 bool \geometry\metric_index:
    gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_test_geometry_matches geometry metric_index

fn join_stroke_offset_step_record_ok %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStep fn i32 bool \step\metric_index:
    match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_status step:
        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStatusKind::Projected:
            match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_geometry step:
                Option::None: false
                Option::Some geometry: join_stroke_offset_geometry_ok &geometry metric_index
        _: false

fn join_stroke_offset_run %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionOwner impure fn i32 bool \owner\metric_index:
    if lt metric_index 3:
        then:
            match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step owner 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_owner error
                    false
                Result::Ok step:
                    let record_ok %bool join_stroke_offset_step_record_ok &step metric_index
                    let joined gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_joined &step
                    let next gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_owner step
                    let cursor_ok %bool eq add metric_index 1 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_next_metric_index &next
                    if eq metric_index 0:
                        then:
                            match joined:
                                Option::None:
                                    gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_free next
                                    false
                                Option::Some rejected:
                                    let forced gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_test_force_geometry_failure next rejected
                                    let kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_kind &forced:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStepErrorKind::GeometryFailed lower:
                                            match lower:
                                                GuiSfntSimpleGlyphRenderStrokeOffsetGeometryErrorKind::ProvenanceContourSpanInvalid: true
                                                _: false
                                        _: false
                                    let rejected_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_rejected &forced:
                                        Option::Some value: join_stroke_metric_joined_first_ok &value
                                        Option::None: false
                                    let recovered gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_owner forced
                                    let recovered_cursor_ok %bool eq 1 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_next_metric_index &recovered
                                    let remaining_ok %bool join_stroke_offset_run recovered 1
                                    and record_ok (and cursor_ok (and kind_ok (and rejected_ok (and recovered_cursor_ok remaining_ok))))
                        else:
                            let remaining_ok %bool join_stroke_offset_run next (add metric_index 1)
                            and record_ok (and cursor_ok remaining_ok)
        else:
            match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step owner 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_free gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_owner error
                    false
                Result::Ok step:
                    let terminal_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_status &step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStatusKind::Completed:
                            match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_geometry &step:
                                Option::None: true
                                Option::Some _geometry: false
                        _: false
                    let completed gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_owner step
                    gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_free completed
                    terminal_ok

fn join_stroke_side_edge_assert %fn &GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner fn &GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection fn i32 bool \owner\edge\edge_index:
    let status_count_ok %bool eq add edge_index 1 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_side_edge_count owner
    let capacity_ok %bool eq 6 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_edges_cap owner
    let expected_pending %bool or eq edge_index 0 or eq edge_index 2 eq edge_index 4
    let actual_pending %bool gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_has_pending owner
    let pending_ok %bool if expected_pending then actual_pending else not actual_pending
    let geometry_ok %bool gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_test_edge_matches edge edge_index
    and status_count_ok (and capacity_ok (and pending_ok geometry_ok))

fn join_stroke_side_edge_start %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionOwner bool \offset:
    match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_start offset:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_start_error_free error
            false
        Result::Ok owner0:
            match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step owner0 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                    false
                Result::Ok budget_step:
                    match budget_step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                            gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                            false
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_owner1:
                            let status %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_owner1
                            let edge %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_owner1
                            let owner1 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_owner1
                            let budget_ok %bool match status:
                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::StepBudgetExhausted:
                                    match edge:
                                        Option::None: true
                                        Option::Some _edge: false
                                _: false
                            let cursor_ok %bool and (eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_geometry_count &owner1) (eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_side_edge_count &owner1)
                            let forced_lower gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_test_force_offset_projection_failure owner1
                            let lower_kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_kind &forced_lower:
                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeErrorKind::OffsetProjectionFailed lower:
                                    match lower:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStepErrorKind::GeometryFailed geometry_kind:
                                            match geometry_kind:
                                                GuiSfntSimpleGlyphRenderStrokeOffsetGeometryErrorKind::ProvenanceContourSpanInvalid: true
                                                _: false
                                        _: false
                                _: false
                            let recovered gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_owner forced_lower
                            let recovery_ok %bool and (eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_geometry_count &recovered) (eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_side_edge_count &recovered)
                            let run_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step recovered 1:
                                    Result::Err error:
                                        gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                        false
                                    Result::Ok step0:
                                        match step0:
                                            GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                false
                                            GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_next0:
                                                let status0 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_next0
                                                let edge0 %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_next0
                                                let next0 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_next0
                                                let status_ok0 %bool match status0:
                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::EdgePushed: true
                                                    _: false
                                                let edge_ok0 %bool match edge0:
                                                    Option::None: false
                                                    Option::Some value: join_stroke_side_edge_assert &next0 &value 0
                                                let forced_push gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_test_force_push_failure next0
                                                let push_kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_kind &forced_push:
                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeErrorKind::StorageFailed lower:
                                                        match lower:
                                                            StdErrorKind::CapacityExceeded: true
                                                            _: false
                                                    _: false
                                                let rejected_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_rejected &forced_push:
                                                    Option::Some rejected: not gui_sfnt_simple_glyph_render_stroke_side_edge_projection_is_source_forward &rejected
                                                    Option::None: false
                                                let recovered_push gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_owner forced_push
                                                let push_recovery_ok %bool and eq 1 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_side_edge_count &recovered_push gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_has_pending &recovered_push
                                                let remaining_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step recovered_push 1:
                                                        Result::Err error:
                                                            gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                                            false
                                                        Result::Ok step1:
                                                            match step1:
                                                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                                    gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                                    false
                                                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_next1:
                                                                    let status1 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_next1
                                                                    let edge1 %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_next1
                                                                    let next1 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_next1
                                                                    let status_ok1 %bool match status1:
                                                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::EdgePushed: true
                                                                        _: false
                                                                    let edge_ok1 %bool match edge1:
                                                                        Option::None: false
                                                                        Option::Some value: join_stroke_side_edge_assert &next1 &value 1
                                                                    let remaining_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step next1 1:
                                                                            Result::Err error:
                                                                                gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                                                                false
                                                                            Result::Ok step2:
                                                                                match step2:
                                                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                                                        gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                                                        false
                                                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_next2:
                                                                                        let status2 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_next2
                                                                                        let edge2 %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_next2
                                                                                        let next2 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_next2
                                                                                        let status_ok2 %bool match status2:
                                                                                            GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::EdgePushed: true
                                                                                            _: false
                                                                                        let edge_ok2 %bool match edge2:
                                                                                            Option::None: false
                                                                                            Option::Some value: join_stroke_side_edge_assert &next2 &value 2
                                                                                        let remaining_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step next2 1:
                                                                                                Result::Err error:
                                                                                                    gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                                                                                    false
                                                                                                Result::Ok step3:
                                                                                                    match step3:
                                                                                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                                                                            gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                                                                            false
                                                                                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_next3:
                                                                                                            let status3 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_next3
                                                                                                            let edge3 %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_next3
                                                                                                            let next3 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_next3
                                                                                                            let status_ok3 %bool match status3:
                                                                                                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::EdgePushed: true
                                                                                                                _: false
                                                                                                            let edge_ok3 %bool match edge3:
                                                                                                                Option::None: false
                                                                                                                Option::Some value: join_stroke_side_edge_assert &next3 &value 3
                                                                                                            let remaining_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step next3 1:
                                                                                                                    Result::Err error:
                                                                                                                        gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                                                                                                        false
                                                                                                                    Result::Ok step4:
                                                                                                                        match step4:
                                                                                                                            GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                                                                                                gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                                                                                                false
                                                                                                                            GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_next4:
                                                                                                                                let status4 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_next4
                                                                                                                                let edge4 %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_next4
                                                                                                                                let next4 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_next4
                                                                                                                                let status_ok4 %bool match status4:
                                                                                                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::EdgePushed: true
                                                                                                                                    _: false
                                                                                                                                let edge_ok4 %bool match edge4:
                                                                                                                                    Option::None: false
                                                                                                                                    Option::Some value: join_stroke_side_edge_assert &next4 &value 4
                                                                                                                                let remaining_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step next4 1:
                                                                                                                                        Result::Err error:
                                                                                                                                            gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                                                                                                                            false
                                                                                                                                        Result::Ok step5:
                                                                                                                                            match step5:
                                                                                                                                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                                                                                                                    gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                                                                                                                    false
                                                                                                                                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_next5:
                                                                                                                                                    let status5 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_next5
                                                                                                                                                    let edge5 %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_next5
                                                                                                                                                    let next5 %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_next5
                                                                                                                                                    let status_ok5 %bool match status5:
                                                                                                                                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind::EdgePushed: true
                                                                                                                                                        _: false
                                                                                                                                                    let edge_ok5 %bool match edge5:
                                                                                                                                                        Option::None: false
                                                                                                                                                        Option::Some value: join_stroke_side_edge_assert &next5 &value 5
                                                                                                                                                    let remaining_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step next5 1:
                                                                                                                                                            Result::Err error:
                                                                                                                                                                gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_step_error_free error
                                                                                                                                                                false
                                                                                                                                                            Result::Ok step6:
                                                                                                                                                                match step6:
                                                                                                                                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::Progress progress_returned:
                                                                                                                                                                        let _status %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStatusKind gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_status &progress_returned
                                                                                                                                                                        let _edge %Option GuiSfntSimpleGlyphRenderStrokeSideEdgeProjection gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_edge &progress_returned
                                                                                                                                                                        let returned %GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeOwner gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_progress_owner progress_returned
                                                                                                                                                                        gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_owner_free returned
                                                                                                                                                                        false
                                                                                                                                                                    GuiFontRegisteredFaceSimpleGlyphIndexedStrokeSideEdgeStep::CompletedValue completed:
                                                                                                                                                                        let counts_ok %bool and eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_geometry_count &completed (and eq 6 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_side_edge_count &completed (and eq 4 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_line_side_edge_count &completed (and eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_quadratic_side_edge_count &completed (and eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_left_side_edge_count &completed (and eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_right_side_edge_count &completed eq 6 gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_edges_cap &completed)))))
                                                                                                                                                                        gui_font_registered_face_simple_glyph_indexed_stroke_side_edge_completed_owner_free completed
                                                                                                                                                                        counts_ok
                                                                                                                                                    and status_ok5 (and edge_ok5 remaining_ok)
                                                                                                                                and status_ok4 (and edge_ok4 remaining_ok)
                                                                                                            and status_ok3 (and edge_ok3 remaining_ok)
                                                                                        and status_ok2 (and edge_ok2 remaining_ok)
                                                                    and status_ok1 (and edge_ok1 remaining_ok)
                                                and status_ok0 (and edge_ok0 (and push_kind_ok (and rejected_ok (and push_recovery_ok remaining_ok))))
                            and budget_ok (and cursor_ok (and lower_kind_ok (and recovery_ok run_ok)))

fn join_stroke_metric_join_start %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricCompletedOwner bool \completed:
    let full %u8 cast 255
    let zero %u8 cast 0
    let color %Rgba8888 rgba8888_new full zero zero full
    let stroke %GuiStroke unwrap_ok gui_stroke_new color 4 GuiStrokeCap::Butt GuiStrokeJoin::Miter 4.0 GuiStrokeDash::Solid
    let paint %GuiGlyphPaint unwrap_ok gui_glyph_paint_result none (some stroke) gui_shadow_ref_none GuiBlendMode::SourceOver
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_start completed (gui_point_new 3 sub 0 2) paint:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_start_error_free error
            false
        Result::Ok owner:
            let origin gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_origin &owner
            let summary_ok %bool and eq 8 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_command_count &owner and eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_metric_count &owner and eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_line_count &owner and eq 1 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_quadratic_count &owner and eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_next_metric_index &owner and eq 4 gui_font_registered_face_simple_glyph_indexed_stroke_metric_join_owner_stroke_width &owner and eq 3 gui_point_x &origin eq sub 0 2 gui_point_y &origin
            let offset_owner0 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_start owner
            match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step offset_owner0 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_free error
                    false
                Result::Ok budget_step:
                    let budget_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_status &budget_step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStatusKind::StepBudgetExhausted: true
                        _: false
                    let offset_owner1 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_owner budget_step
                    let budget_cursor_ok %bool eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_next_metric_index &offset_owner1
                    let forced_join gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_test_force_join_failure offset_owner1
                    let join_kind_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_kind &forced_join:
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeOffsetProjectionStepErrorKind::JoinFailed lower:
                            match lower:
                                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricJoinStepErrorKind::MetricIndexMismatch: true
                                _: false
                        _: false
                    let offset_owner2 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_step_error_owner forced_join
                    let join_cursor_ok %bool eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_offset_projection_owner_next_metric_index &offset_owner2
                    let joined_ok %bool join_stroke_side_edge_start offset_owner2
                    and summary_ok (and budget_ok (and budget_cursor_ok (and join_kind_ok (and join_cursor_ok joined_ok))))

fn join_stroke_metric_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricDrainOwner bool \owner:
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_step_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_metric_step_error_free error
            false
        Result::Ok step:
            let status gui_font_registered_face_simple_glyph_indexed_stroke_metric_budget_step_status &step
            let next gui_font_registered_face_simple_glyph_indexed_stroke_metric_budget_step_take_owner step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricBudgetStatusKind::Progressed: join_stroke_metric_drain next
                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricBudgetStatusKind::Completed:
                    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_seal next:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_stroke_metric_seal_error_free error
                            false
                        Result::Ok completed:
                            let counts_ok %bool and eq 8 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_command_count &completed and eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_metric_count &completed and eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_move_count &completed and eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_line_count &completed and eq 1 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_quadratic_count &completed and eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_skip_count &completed eq 3 gui_font_registered_face_simple_glyph_indexed_stroke_metric_completed_owner_storage_cap &completed
                            let provenance_ok %bool and join_stroke_metric_provenance_ok &completed 0 1 2 and join_stroke_metric_provenance_ok &completed 1 3 3 join_stroke_metric_provenance_ok &completed 2 7 2
                            and counts_ok and provenance_ok join_stroke_metric_join_start completed
                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeMetricBudgetStatusKind::StepBudgetExhausted:
                    gui_font_registered_face_simple_glyph_indexed_stroke_metric_drain_owner_free next
                    false

fn join_stroke_source_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandSinkCompletedOwner bool \completed:
    let source gui_font_registered_face_simple_glyph_indexed_stroke_source_contour_start completed
    match gui_font_registered_face_simple_glyph_indexed_stroke_metric_start source:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_metric_start_error_free error
            false
        Result::Ok owner: join_stroke_metric_drain owner

fn join_path_command_sink_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandSinkWritingOwner impure fn i32 bool \owner\remaining:
    if le remaining 0:
        then:
            match gui_font_registered_face_simple_glyph_indexed_path_command_sink_writer_step_budget owner 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_path_command_sink_step_error_free error
                    false
                Result::Ok terminal:
                    let status_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_command_sink_budget_step_status &terminal:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandSinkBudgetStatus::Completed: true
                        _: false
                    let sealed_owner gui_font_registered_face_simple_glyph_indexed_path_command_sink_budget_step_owner terminal
                    match gui_font_registered_face_simple_glyph_indexed_path_command_sink_writer_seal sealed_owner:
                        Result::Err recovered:
                            gui_font_registered_face_simple_glyph_indexed_path_command_sink_writing_owner_free recovered
                            false
                        Result::Ok completed: and status_ok join_stroke_source_complete completed
        else:
            match gui_font_registered_face_simple_glyph_indexed_path_command_sink_writer_step_budget owner 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_path_command_sink_step_error_free error
                    false
                Result::Ok step:
                    let status_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_command_sink_budget_step_status &step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandSinkBudgetStatus::Written: true
                        _: false
                    let next gui_font_registered_face_simple_glyph_indexed_path_command_sink_budget_step_owner step
                    and status_ok join_path_command_sink_drain next sub remaining 1

fn join_path_command_sink_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandStreamCompletedOwner bool \source:
    match gui_font_registered_face_simple_glyph_indexed_path_command_sink_plan_start source:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_path_command_sink_plan_error_free error
            false
        Result::Ok planned:
            match gui_font_registered_face_simple_glyph_indexed_path_command_sink_allocate planned:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_path_command_sink_alloc_error_free error
                    false
                Result::Ok allocated:
                    match gui_font_registered_face_simple_glyph_indexed_path_command_sink_writer_start allocated:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_path_command_sink_start_error_free error
                            false
                        Result::Ok writer: join_path_command_sink_drain writer 8

fn join_path_command_stream_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandStreamOwner impure fn i32 bool \owner\index:
    match gui_font_registered_face_simple_glyph_indexed_path_command_stream_step_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_path_command_stream_error_free error
            false
        Result::Ok step:
            match gui_font_registered_face_simple_glyph_indexed_path_command_stream_budget_step_status &step:
                GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandStreamBudgetStatus::Prepared value:
                    let stored gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_stored_tag &value
                    let source gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_source_tag &value
                    let expected_scalar %i32 if or or eq index 0 eq index 2 eq index 6 then 1 else if or eq index 1 eq index 7 then 2 else if eq index 3 then 3 else 4
                    let command_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_command &value:
                        GuiSfntSimpleGlyphPathCommand::MoveTo _point: eq expected_scalar 1
                        GuiSfntSimpleGlyphPathCommand::LineTo _point: eq expected_scalar 2
                        GuiSfntSimpleGlyphPathCommand::QuadraticTo _curve: eq expected_scalar 3
                        GuiSfntSimpleGlyphPathCommand::SkipNoSegment _skip: eq expected_scalar 4
                    let prepared_ok %bool and eq index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_path_command_value_path_command_index &value and eq expected_scalar gui_sfnt_simple_glyph_path_command_tag_scalar_value &stored and eq expected_scalar gui_sfnt_simple_glyph_path_command_tag_scalar_value &source command_ok
                    let next gui_font_registered_face_simple_glyph_indexed_path_command_stream_budget_step_owner step
                    and prepared_ok join_path_command_stream_drain next add index 1
                GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandStreamBudgetStatus::Completed:
                    let running gui_font_registered_face_simple_glyph_indexed_path_command_stream_budget_step_owner step
                    match gui_font_registered_face_simple_glyph_indexed_path_command_stream_seal running:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_path_command_stream_seal_error_free error
                            false
                        Result::Ok completed:
                            let summary gui_font_registered_face_simple_glyph_indexed_path_command_stream_completed_owner_summary &completed
                            let summary_ok %bool and eq index 8 and eq 8 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_prepare_summary_total_count &summary and eq 3 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_prepare_summary_move_to_count &summary and eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_prepare_summary_line_to_count &summary and eq 1 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_prepare_summary_quadratic_to_count &summary and eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_prepare_summary_skip_no_segment_count &summary eq 7 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_stream_prepare_summary_last_path_command_index &summary
                            and summary_ok join_path_command_sink_complete completed
                GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandStreamBudgetStatus::StepBudgetExhausted:
                    gui_font_registered_face_simple_glyph_indexed_path_command_stream_budget_step_free step
                    false

fn join_path_command_stream_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagCompletedOwner bool \completed:
    match gui_font_registered_face_simple_glyph_indexed_path_command_stream_start completed:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_path_command_stream_start_error_free error
            false
        Result::Ok owner: join_path_command_stream_drain owner 0

fn join_path_command_tag_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagOwner impure fn i32 bool \owner\index:
    if lt index 8:
        then:
            match gui_font_registered_face_simple_glyph_indexed_path_command_tag_drain_budget owner 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_path_command_tag_error_free error
                    false
                Result::Ok step:
                    let pushed_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_command_tag_budget_step_status &step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagBudgetStatus::Pushed slot: join_path_command_tag_slot_ok &slot index
                        _: false
                    let next %GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagOwner gui_font_registered_face_simple_glyph_indexed_path_command_tag_budget_step_take_owner step
                    let invariant_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_command_tag_owner_phase_invariant_check &next:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagPhaseInvariantCheck::Valid: true
                        _: false
                    and pushed_ok and invariant_ok join_path_command_tag_drain next add index 1
        else:
            match gui_font_registered_face_simple_glyph_indexed_path_command_tag_drain_budget owner 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_path_command_tag_error_free error
                    false
                Result::Ok terminal:
                    let terminal_ok %bool match gui_font_registered_face_simple_glyph_indexed_path_command_tag_budget_step_status &terminal:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagBudgetStatus::Completed: true
                        _: false
                    let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedPathCommandTagOwner gui_font_registered_face_simple_glyph_indexed_path_command_tag_budget_step_take_owner terminal
                    match gui_font_registered_face_simple_glyph_indexed_path_command_tag_seal_completed completed_owner:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_path_command_tag_seal_error_free error
                            false
                        Result::Ok completed:
                            and terminal_ok join_path_command_stream_complete completed

fn join_path_command_tag_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedEdgeCompletedOwner bool \edge:
    match gui_font_registered_face_simple_glyph_indexed_path_command_tag_start edge:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_path_command_tag_start_error_free error
            false
        Result::Ok owner:
            join_path_command_tag_drain owner 0

fn join_edge_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedEdgeOwner impure fn i32 bool \owner\index:
    if lt index 4:
        then:
            match gui_font_registered_face_simple_glyph_indexed_edge_drain_budget owner 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_edge_owner_free gui_font_registered_face_simple_glyph_indexed_edge_error_take_owner error
                    false
                Result::Ok step:
                    let pushed_ok %bool match gui_font_registered_face_simple_glyph_indexed_edge_budget_step_status &step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedEdgeBudgetStatus::Pushed edge:
                            and eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_slot_contour_index &edge and eq index gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_slot_contour_edge_index &edge eq if eq index 3 then 0 else add index 1 gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_edge_slot_next_contour_point_index &edge
                        _: false
                    let next %GuiFontRegisteredFaceSimpleGlyphIndexedEdgeOwner gui_font_registered_face_simple_glyph_indexed_edge_budget_step_take_owner step
                    let invariant_ok %bool match gui_font_registered_face_simple_glyph_indexed_edge_owner_phase_invariant_check &next:
                        GuiFontRegisteredFaceSimpleGlyphIndexedEdgePhaseInvariantCheck::Valid: true
                        _: false
                    and pushed_ok and invariant_ok join_edge_drain next add index 1
        else:
            match gui_font_registered_face_simple_glyph_indexed_edge_drain_budget owner 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_edge_owner_free gui_font_registered_face_simple_glyph_indexed_edge_error_take_owner error
                    false
                Result::Ok terminal:
                    let terminal_ok %bool match gui_font_registered_face_simple_glyph_indexed_edge_budget_step_status &terminal:
                        GuiFontRegisteredFaceSimpleGlyphIndexedEdgeBudgetStatus::Completed: true
                        _: false
                    let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedEdgeOwner gui_font_registered_face_simple_glyph_indexed_edge_budget_step_take_owner terminal
                    match gui_font_registered_face_simple_glyph_indexed_edge_seal_completed completed_owner:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_edge_owner_free gui_font_registered_face_simple_glyph_indexed_edge_seal_error_take_owner error
                            false
                        Result::Ok completed:
                            and terminal_ok and eq 13 gui_font_registered_face_simple_glyph_indexed_edge_completed_owner_storage_len &completed join_path_command_tag_complete completed

fn join_edge_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPointYCompletedOwner bool \point_y:
    match gui_font_registered_face_simple_glyph_indexed_edge_start point_y:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_point_y_completed_owner_free gui_font_registered_face_simple_glyph_indexed_edge_start_error_take_owner error
            false
        Result::Ok owner:
            join_edge_drain owner 0

fn join_point_y_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPointYOwner impure fn i32 bool \owner\index:
    if lt index 4:
        then:
            match gui_font_registered_face_simple_glyph_indexed_point_y_drain_budget owner 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_point_y_step_error_free error
                    false
                Result::Ok step:
                    let pushed_ok %bool match gui_font_registered_face_simple_glyph_indexed_point_y_budget_step_status &step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPointYBudgetStatus::Pushed point:
                            and eq index gui_sfnt_simple_glyph_point_y_slot_point_index &point eq if eq index 2 then 5 else 0 gui_sfnt_simple_glyph_point_y_slot_y &point
                        _: false
                    let next %GuiFontRegisteredFaceSimpleGlyphIndexedPointYOwner gui_font_registered_face_simple_glyph_indexed_point_y_budget_step_take_owner step
                    let cursor %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_font_registered_face_simple_glyph_indexed_point_y_owner_cursor &next
                    let progress_ok %bool and eq add 5 add index 1 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &cursor and eq add index 1 gui_font_registered_face_simple_glyph_indexed_point_y_owner_logical_item_index &next eq add 5 add index 1 gui_font_registered_face_simple_glyph_indexed_point_y_owner_storage_len &next
                    let invariant_ok %bool match gui_font_registered_face_simple_glyph_indexed_point_y_owner_phase_invariant_check &next:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPointYPhaseInvariantCheck::Valid: true
                        _: false
                    and pushed_ok and progress_ok and invariant_ok join_point_y_drain next add index 1
        else:
            match gui_font_registered_face_simple_glyph_indexed_point_y_drain_budget owner 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_point_y_step_error_free error
                    false
                Result::Ok terminal:
                    let terminal_ok %bool match gui_font_registered_face_simple_glyph_indexed_point_y_budget_step_status &terminal:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPointYBudgetStatus::Completed: true
                        _: false
                    let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedPointYOwner gui_font_registered_face_simple_glyph_indexed_point_y_budget_step_take_owner terminal
                    match gui_font_registered_face_simple_glyph_indexed_point_y_seal_completed completed_owner:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_point_y_seal_error_free error
                            false
                        Result::Ok completed:
                            and terminal_ok join_edge_complete completed

fn join_point_y_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPointXCompletedOwner bool \point_x:
    match gui_font_registered_face_simple_glyph_indexed_point_y_start point_x:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_point_y_start_error_free error
            false
        Result::Ok owner:
            join_point_y_drain owner 0

fn join_point_x_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedPointXOwner impure fn i32 bool \owner\index:
    if lt index 4:
        then:
            match gui_font_registered_face_simple_glyph_indexed_point_x_drain_budget owner 1:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_point_x_step_error_free error
                    false
                Result::Ok step:
                    let pushed_ok %bool match gui_font_registered_face_simple_glyph_indexed_point_x_budget_step_status &step:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPointXBudgetStatus::Pushed point:
                            let expected_x %i32 if eq index 0 then 0 else add 5 mul index 5
                            and eq index gui_sfnt_simple_glyph_point_x_slot_point_index &point eq expected_x gui_sfnt_simple_glyph_point_x_slot_x &point
                        _: false
                    let next %GuiFontRegisteredFaceSimpleGlyphIndexedPointXOwner gui_font_registered_face_simple_glyph_indexed_point_x_budget_step_take_owner step
                    let cursor %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_font_registered_face_simple_glyph_indexed_point_x_owner_cursor &next
                    let progress_ok %bool and eq add 1 add index 1 gui_sfnt_simple_glyph_outline_scalar_region_cursor_next_index &cursor and eq add index 1 gui_font_registered_face_simple_glyph_indexed_point_x_owner_logical_item_index &next eq add 1 add index 1 gui_font_registered_face_simple_glyph_indexed_point_x_owner_storage_len &next
                    let invariant_ok %bool match gui_font_registered_face_simple_glyph_indexed_point_x_owner_phase_invariant_check &next:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPointXPhaseInvariantCheck::Valid: true
                        _: false
                    and pushed_ok and progress_ok and invariant_ok join_point_x_drain next add index 1
        else:
            match gui_font_registered_face_simple_glyph_indexed_point_x_drain_budget owner 0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_point_x_step_error_free error
                    false
                Result::Ok terminal:
                    let terminal_ok %bool match gui_font_registered_face_simple_glyph_indexed_point_x_budget_step_status &terminal:
                        GuiFontRegisteredFaceSimpleGlyphIndexedPointXBudgetStatus::Completed: true
                        _: false
                    let completed_owner %GuiFontRegisteredFaceSimpleGlyphIndexedPointXOwner gui_font_registered_face_simple_glyph_indexed_point_x_budget_step_take_owner terminal
                    match gui_font_registered_face_simple_glyph_indexed_point_x_seal_completed completed_owner:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_point_x_seal_error_free error
                            false
                        Result::Ok completed:
                            and terminal_ok join_point_y_complete completed

fn join_point_x_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointCompletedOwner bool \endpoint:
    match gui_font_registered_face_simple_glyph_indexed_point_x_start endpoint:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_point_x_start_error_free error
            false
        Result::Ok owner:
            join_point_x_drain owner 0

fn join_contour_endpoint_complete %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryCompletedOwner bool \completed:
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 4 4 8
    match gui_font_registered_face_simple_glyph_indexed_outline_storage_start completed &limit:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_outline_storage_alloc_error_free error
            false
        Result::Ok storage:
            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_start storage:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_start_error_free error
                    false
                Result::Ok owner0:
                    match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step owner0:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_contour_endpoint_step_error_free error
                            false
                        Result::Ok step:
                            let pushed_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_status &step:
                                GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointBudgetStatus::Pushed endpoint:
                                    and eq 0 gui_sfnt_simple_glyph_contour_endpoint_slot_contour_index &endpoint eq 3 gui_sfnt_simple_glyph_contour_endpoint_slot_end_point_index &endpoint
                                _: false
                            let owner1 %GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointOwner gui_font_registered_face_simple_glyph_indexed_contour_endpoint_budget_step_take_owner step
                            let endpoint_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_previous_endpoint &owner1:
                                Option::Some endpoint: eq 3 endpoint
                                Option::None: false
                            let progress_ok %bool match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_progress_kind &owner1:
                                GuiFontRegisteredFaceSimpleGlyphIndexedContourEndpointProgressKind::Completed: true
                                _: false
                            let storage_ok %bool eq 1 gui_font_registered_face_simple_glyph_indexed_contour_endpoint_owner_storage_len &owner1
                            match gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_completed owner1:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_seal_error_free error
                                    false
                                Result::Ok sealed:
                                    and pushed_ok and endpoint_ok and progress_ok and storage_ok join_point_x_complete sealed

fn join_action_summary_drain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner bool \owner:
    match gui_font_registered_face_simple_glyph_indexed_action_summary_drain_budget owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_action_summary_error_free error
            false
        Result::Ok step:
            let status gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_status &step
            let next %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_budget_step_take_owner step
            match status:
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Applied _apply_status:
                    join_action_summary_drain next
                GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryBudgetStatus::Completed:
                    match gui_font_registered_face_simple_glyph_indexed_action_summary_seal_completed next:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_action_summary_seal_error_free error
                            false
                        Result::Ok completed:
                            let apply_state %GuiSfntSimpleGlyphPathSinkActionApplyState gui_font_registered_face_simple_glyph_indexed_action_summary_completed_owner_apply_state &completed
                            let counts_ok %bool and eq 8 gui_sfnt_simple_glyph_path_sink_action_apply_state_emitted_event_count &apply_state and eq 0 gui_sfnt_simple_glyph_path_sink_action_apply_state_reject_count &apply_state and eq 1 gui_sfnt_simple_glyph_path_sink_action_apply_state_close_contour_count &apply_state eq 7 gui_sfnt_simple_glyph_path_sink_action_apply_state_no_action_count &apply_state
                            and counts_ok join_contour_endpoint_complete completed
                _:
                    gui_font_registered_face_simple_glyph_indexed_action_summary_owner_free next
                    false

fn join_indexed_phase_chain %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedOwner bool \indexed:
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    let summary %GuiFontRegisteredFaceSimpleGlyphIndexedActionSummaryOwner gui_font_registered_face_simple_glyph_indexed_action_summary_start indexed &policy
    join_action_summary_drain summary

fn join_span_index_complete %impure fn &GuiFontRegisteredFaceTableEntry impure fn GuiFontRegisteredFaceSimpleGlyphCollectedOwner bool \entry\collected:
    let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 1
    match gui_font_registered_face_simple_glyph_span_index_start entry collected &limit:
        Result::Err error:
            gui_font_registered_face_simple_glyph_span_index_start_error_free error
            false
        Result::Ok builder0:
            match gui_font_registered_face_simple_glyph_span_index_step builder0:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_span_index_step_error_free error
                    false
                Result::Ok builder1:
                    match gui_font_registered_face_simple_glyph_span_index_step builder1:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_span_index_step_error_free error
                            false
                        Result::Ok builder2:
                            match gui_font_registered_face_simple_glyph_span_index_step builder2:
                                Result::Err error:
                                    gui_font_registered_face_simple_glyph_span_index_step_error_free error
                                    false
                                Result::Ok builder3:
                                    match gui_font_registered_face_simple_glyph_span_index_step builder3:
                                        Result::Err error:
                                            gui_font_registered_face_simple_glyph_span_index_step_error_free error
                                            false
                                        Result::Ok builder4:
                                            match gui_font_registered_face_simple_glyph_span_index_complete builder4:
                                                Result::Err error:
                                                    gui_font_registered_face_simple_glyph_span_index_step_error_free error
                                                    false
                                                Result::Ok indexed:
                                                    let count_ok %bool eq 1 gui_font_registered_face_simple_glyph_indexed_owner_span_count &indexed
                                                    let first_ok %bool match gui_font_registered_face_simple_glyph_indexed_owner_span_lookup &indexed 0:
                                                        Result::Err _error: false
                                                        Result::Ok span:
                                                            and eq 0 gui_sfnt_simple_glyph_contour_span_start_point_index &span and eq 3 gui_sfnt_simple_glyph_contour_span_end_point_index &span eq 4 gui_sfnt_simple_glyph_contour_span_point_count &span
                                                    let phase_ok %bool join_indexed_phase_chain indexed
                                                    and count_ok and first_ok phase_ok

fn join_collection_drain %impure fn &GuiFontRegisteredFaceTableEntry impure fn GuiFontRegisteredFaceSimpleGlyphCollectionOwner impure fn i32 bool \entry\owner\collecting_step_count:
    match gui_font_registered_face_simple_glyph_collection_step entry owner:
        Result::Err error:
            gui_font_registered_face_simple_glyph_collection_step_error_free error
            false
        Result::Ok terminal:
            match terminal:
                GuiFontRegisteredFaceSimpleGlyphCollectionTerminal::Collecting next:
                    join_collection_drain entry next add collecting_step_count 1
                GuiFontRegisteredFaceSimpleGlyphCollectionTerminal::Completed completed:
                    and eq collecting_step_count 4 join_span_index_complete entry completed

fn join_registered_callback %impure fn GuiFontRegisteredFaceTable impure fn GuiFontRegisteredFaceTableEntry bool \table\entry:
    let record %GuiFontRegisteredFaceRecord gui_font_registered_face_table_entry_record &entry
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 36
    let mapping %GuiFontRegisteredFaceGlyphMapping GuiFontRegisteredFaceGlyphMapping record 'A' glyph
    let ok %bool match gui_font_registered_face_simple_glyph_lookup &entry &mapping:
        Result::Err _error:
            false
        Result::Ok evidence:
            let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 4
            match gui_font_registered_face_simple_glyph_collection_start &entry &evidence &limit:
                Result::Err _error:
                    false
                Result::Ok owner:
                    join_collection_drain &entry owner 0
    gui_font_registered_face_table_free table
    gui_font_registered_face_table_entry_free entry
    ok

fn join_fixture_run %impure fn void bool \void:
    match join_fixture_bytes:
        Result::Err _message:
            false
        Result::Ok bytes:
            let path %GuiFontResourcePath unwrap_ok gui_font_resource_path_result "fonts/JoinFixture.ttf"
            let request %GuiFontResourceRequest unwrap_ok gui_font_resource_request path none none GuiFontDecodePolicy::SfntOnly
            let resource %GuiFontResourceBytes gui_font_resource_bytes_new request GuiFontResourceSource::Vfs bytes
            let registered_request %GuiFontRegisteredFaceRequest unwrap_ok gui_font_registered_face_request_from_raw 401 409
            match gui_font_registered_face_register_bytes registered_request resource:
                Result::Err error:
                    gui_font_registered_face_error_free error
                    false
                Result::Ok face:
                    match gui_font_registered_face_table_new 1:
                        Result::Err _error:
                            gui_font_registered_face_free face
                            false
                        Result::Ok table:
                            match gui_font_registered_face_table_register table face:
                                Result::Err error:
                                    gui_font_registered_face_table_register_error_free error
                                    false
                                Result::Ok registration:
                                    gui_font_registered_face_table_registration_with registration @join_registered_callback

fn main %impure fn void i32 \void:
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_metric_join"
    let checked %TestReport test_report_push report assert "registered bytes produce sealed stroke metric provenance" join_fixture_run
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
