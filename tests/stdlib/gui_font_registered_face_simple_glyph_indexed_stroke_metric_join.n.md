# GUI font registered simple glyph indexed stroke metric join

このファイルは、固定 SFNT bytes を registered face owner に登録し、simple glyph collection と contour span index を production API だけで構築できることを確認する。

## registered bytes reach indexed contour spans

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_metric_join\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered bytes produce indexed contour spans\" expected=\"true\" actual=\"true\" message=\"\"\n"
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
                                    gui_font_registered_face_simple_glyph_indexed_contour_endpoint_completed_owner_free sealed
                                    and pushed_ok and endpoint_ok and progress_ok storage_ok

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
    let checked %TestReport test_report_push report assert "registered bytes produce indexed contour spans" join_fixture_run
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
