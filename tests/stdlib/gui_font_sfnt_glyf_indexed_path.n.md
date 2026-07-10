# GUI font SFNT glyf indexed contour path doctests

このファイルは、F5nw の lower indexed contour path が pending contour ごとに一度だけ span を取得し、active contour の point、edge、curve、event へ同じ checked span を渡すことを検査する。

## unequal contours keep one checked span through line, quadratic, and single-point traversal

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn make_capacity %fn GuiGlyphId GuiSfntSimpleGlyphOutlineStorageCapacity \glyph:
    gui_sfnt_simple_glyph_outline_storage_capacity glyph 3 6 6 6 12

fn make_item %fn GuiGlyphId fn i32 fn i32 fn i32 fn bool fn bool GuiSfntSimpleGlyphOutlinePointStreamItem \glyph\point_index\x\y\on_curve\end_of_contour:
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph point_index x y on_curve end_of_contour
    gui_sfnt_simple_glyph_outline_point_stream_item point

fn push_item %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn GuiSfntSimpleGlyphOutlinePointStreamItem Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \collection\item:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection item:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error
            Result::Err "push"

fn build_collection %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 6
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc capacity &limit:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok c0:
            match push_item c0 make_item glyph 0 0 0 true false:
                Result::Err error: Result::Err error
                Result::Ok c1:
                    match push_item c1 make_item glyph 1 8 0 true true:
                        Result::Err error: Result::Err error
                        Result::Ok c2:
                            match push_item c2 make_item glyph 2 10 0 true false:
                                Result::Err error: Result::Err error
                                Result::Ok c3:
                                    match push_item c3 make_item glyph 3 14 6 false false:
                                        Result::Err error: Result::Err error
                                        Result::Ok c4:
                                            match push_item c4 make_item glyph 4 20 0 true true:
                                                Result::Err error: Result::Err error
                                                Result::Ok c5:
                                                    push_item c5 make_item glyph 5 30 4 true true

fn finish_index %impure fn GuiSfntSimpleGlyphContourSpanIndexBuilderOwner Result GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner str \builder:
    let collection %&GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &builder
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity collection
    let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
    let next_point_index %i32 gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &builder
    if eq next_point_index point_count:
        then:
            match gui_sfnt_simple_glyph_contour_span_index_complete builder:
                Result::Ok owner: Result::Ok owner
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_step_error_free error
                    Result::Err "complete"
        else:
            match gui_sfnt_simple_glyph_contour_span_index_step builder:
                Result::Ok next: finish_index next
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_step_error_free error
                    Result::Err "step"

fn build_index %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity Result GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner str \capacity:
    match build_collection capacity:
        Result::Err error: Result::Err error
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 3
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    Result::Err "index start"
                Result::Ok builder:
                    finish_index builder

fn kind_is_move %fn GuiSfntSimpleGlyphPathSinkEventKind bool \kind:
    match kind:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo: true
        _: false

fn kind_is_line %fn GuiSfntSimpleGlyphPathSinkEventKind bool \kind:
    match kind:
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo: true
        _: false

fn kind_is_quadratic %fn GuiSfntSimpleGlyphPathSinkEventKind bool \kind:
    match kind:
        GuiSfntSimpleGlyphPathSinkEventKind::QuadraticTo: true
        _: false

fn kind_is_single_point_skip %fn GuiSfntSimpleGlyphPathSinkEventKind bool \kind:
    match kind:
        GuiSfntSimpleGlyphPathSinkEventKind::SkipNoSegment reason:
            match reason:
                GuiSfntSimpleGlyphCurveNoSegmentReason::SinglePointContour: true
                _: false
        _: false

fn active_span_matches %fn &GuiSfntSimpleGlyphIndexedPathContourState fn i32 fn i32 fn i32 bool \state\contour\start\count:
    match *state:
        GuiSfntSimpleGlyphIndexedPathContourState::ActiveContour active:
            let span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_indexed_path_active_contour_span &active
            let contour_ok %bool eq contour gui_sfnt_simple_glyph_contour_span_index &span
            let start_ok %bool eq start gui_sfnt_simple_glyph_contour_span_start_point_index &span
            let count_ok %bool eq count gui_sfnt_simple_glyph_contour_span_point_count &span
            and contour_ok and start_ok count_ok
        _: false

fn first_two_kinds_ok %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner fn i32 fn i32 fn i32 fn i32 bool \owner\contour\start\count\expected_kind:
    let initial %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_start owner contour
    match gui_sfnt_simple_glyph_indexed_path_contour_step owner initial:
        Result::Err _error: false
        Result::Ok first:
            let first_step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_indexed_path_contour_step_result_step &first
            let first_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_contour_step_kind &first_step
            let first_state %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_step_result_state &first
            let first_ok %bool and kind_is_move first_kind active_span_matches &first_state contour start count
            match gui_sfnt_simple_glyph_indexed_path_contour_step owner first_state:
                Result::Err _error: false
                Result::Ok second:
                    let second_step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_indexed_path_contour_step_result_step &second
                    let second_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_contour_step_kind &second_step
                    let second_ok %bool if eq expected_kind 0:
                        then: kind_is_line second_kind
                        else: kind_is_quadratic second_kind
                    and first_ok second_ok

fn single_point_ok %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner bool \owner:
    let initial %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_start owner 2
    match gui_sfnt_simple_glyph_indexed_path_contour_step owner initial:
        Result::Err _error: false
        Result::Ok first:
            let step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_indexed_path_contour_step_result_step &first
            let kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_contour_step_kind &step
            let state %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_step_result_state &first
            and kind_is_single_point_skip kind active_span_matches &state 2 5 1

fn checked_span_propagation_ok %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner bool \owner:
    match gui_sfnt_simple_glyph_contour_span_index_lookup owner 1:
        Result::Err _error: false
        Result::Ok span:
            let collection %&GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_contour_span_indexed_collection_ref owner
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point_checked_span collection span 2:
                Result::Err _error: false
                Result::Ok contour_point:
                    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &contour_point
                    let point_span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_point_span &contour_point
                    let point_ok %bool eq 4 gui_sfnt_simple_glyph_point_index &point
                    let span_ok %bool and eq 1 gui_sfnt_simple_glyph_contour_span_index &point_span eq 3 gui_sfnt_simple_glyph_contour_span_point_count &point_span
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge_checked_span collection span 0:
                        Result::Err _error: false
                        Result::Ok edge:
                            let start %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_start &edge
                            let end %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_end &edge
                            let start_span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_point_span &start
                            let end_span %GuiSfntSimpleGlyphContourSpan gui_sfnt_simple_glyph_contour_point_span &end
                            let edge_span_ok %bool and eq 1 gui_sfnt_simple_glyph_contour_span_index &start_span eq 1 gui_sfnt_simple_glyph_contour_span_index &end_span
                            and point_ok and span_ok edge_span_ok

fn lookup_error_ok %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner bool \owner:
    let initial %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_start owner 3
    match gui_sfnt_simple_glyph_indexed_path_contour_step owner initial:
        Result::Ok _result: false
        Result::Err error:
            let kind_ok %bool match gui_sfnt_simple_glyph_indexed_path_contour_step_error_kind &error:
                GuiSfntSimpleGlyphIndexedPathContourStepErrorKind::SpanIndexLookupFailed: true
                _: false
            let lookup_ok %bool match gui_sfnt_simple_glyph_indexed_path_contour_step_error_lookup_error &error:
                Option::None: false
                Option::Some lookup_error:
                    match gui_sfnt_simple_glyph_contour_span_index_lookup_error_kind &lookup_error:
                        GuiSfntSimpleGlyphContourSpanIndexLookupErrorKind::ContourIndexOutOfRange: true
                        _: false
            and kind_ok lookup_ok

fn cursor_span_mismatch_rejected %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner bool \owner:
    match gui_sfnt_simple_glyph_contour_span_index_lookup owner 1:
        Result::Err _error:
            false
        Result::Ok span:
            let collection %&GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_contour_span_indexed_collection_ref owner
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_contour_span_indexed_collection_capacity owner
            let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph &capacity
            let cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step_checked_span collection span cursor:
                Result::Ok _step:
                    false
                Result::Err error:
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step_error_kind &error:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPathContourStepErrorKind::CursorContourMismatch:
                            true
                        _:
                            false

fn closure_policies_from_state_ok %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner fn GuiSfntSimpleGlyphIndexedPathContourState bool \owner\state:
    match gui_sfnt_simple_glyph_indexed_path_contour_step owner state:
        Result::Err _error:
            false
        Result::Ok result:
            let contour_step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_indexed_path_contour_step_result_step &result
            let next_state %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_step_result_state &result
            match gui_sfnt_simple_glyph_path_contour_step_next &contour_step:
                GuiSfntSimpleGlyphPathContourNext::Continue _cursor:
                    closure_policies_from_state_ok owner next_state
                GuiSfntSimpleGlyphPathContourNext::EndContour:
                    let keep_policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::KeepOpen
                    let close_policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::KeepTypedSkip GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
                    let keep_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step_from_contour_step &keep_policy &contour_step
                    let close_step %GuiSfntSimpleGlyphPathSinkStep gui_sfnt_simple_glyph_path_sink_step_from_contour_step &close_policy &contour_step
                    let keep_ok %bool match gui_sfnt_simple_glyph_path_sink_step_tail_action &keep_step:
                        GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction: true
                        _: false
                    let close_ok %bool match gui_sfnt_simple_glyph_path_sink_step_tail_action &close_step:
                        GuiSfntSimpleGlyphPathSinkTailAction::CloseContour _close: true
                        _: false
                    and keep_ok close_ok

fn closure_policies_ok %fn &GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner bool \owner:
    let initial %GuiSfntSimpleGlyphIndexedPathContourState gui_sfnt_simple_glyph_indexed_path_contour_start owner 0
    closure_policies_from_state_ok owner initial

fn contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 931
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph
    match build_index &capacity:
        Result::Err _error: false
        Result::Ok owner:
            let unequal_ok %bool and eq 3 gui_sfnt_simple_glyph_contour_span_indexed_collection_span_count &owner and first_two_kinds_ok &owner 0 0 2 0 first_two_kinds_ok &owner 1 2 3 1
            let topology_ok %bool and single_point_ok &owner and checked_span_propagation_ok &owner closure_policies_ok &owner
            let error_ok %bool and lookup_error_ok &owner cursor_span_mismatch_rejected &owner
            gui_sfnt_simple_glyph_contour_span_indexed_collection_free owner
            and unequal_ok and topology_ok error_ok

fn main %impure fn void i32 \void:
    test_assertion_exit_code assert "indexed contour path uses one checked span" contract_ok
```
