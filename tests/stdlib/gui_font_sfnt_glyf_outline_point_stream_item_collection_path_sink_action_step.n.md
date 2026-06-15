# GUI font SFNT glyf outline point stream item collection path sink action step doctests

このファイルは、F5ag の collection-backed sink action step lookup が F5af sink step lookup と pure action-step projection だけを通ることを固定する。

source policy coverage labels:

- path_sink_action_step_primary_ok
- path_sink_action_step_tail_ok
- path_sink_action_step_error_propagates_ok
- path_sink_action_step_no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path sink action step smoke

neplg2:test[skip]
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

fn make_capacity %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphOutlineStorageCapacity \glyph\contours\points:
    gui_sfnt_simple_glyph_outline_storage_capacity glyph contours points points points mul points 2

fn make_item %fn GuiGlyphId fn i32 fn i32 fn i32 fn bool fn bool GuiSfntSimpleGlyphOutlinePointStreamItem \glyph\point_index\x\y\on_curve\end_of_contour:
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph point_index x y on_curve end_of_contour
    gui_sfnt_simple_glyph_outline_point_stream_item point

fn push_item_or_free %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn GuiSfntSimpleGlyphOutlinePointStreamItem Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \collection\item:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection item:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
            Result::Err "push"

fn alloc_collection %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity:
    let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count capacity
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit point_count
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc capacity &limit:
        Result::Ok collection:
            Result::Ok collection
        Result::Err _error:
            Result::Err "alloc"

fn build_line_collection %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match alloc_collection capacity:
        Result::Err message:
            Result::Err message
        Result::Ok collection0:
            match push_item_or_free collection0 make_item glyph 0 0 0 true false:
                Result::Err message:
                    Result::Err message
                Result::Ok collection1:
                    push_item_or_free collection1 make_item glyph 1 8 4 true true

fn action_is_emit_event %fn &GuiSfntSimpleGlyphPathSinkActionStep bool \step:
    let action %GuiSfntSimpleGlyphPathSinkAction gui_sfnt_simple_glyph_path_sink_action_step_action step
    match action:
        GuiSfntSimpleGlyphPathSinkAction::EmitEvent _event:
            true
        GuiSfntSimpleGlyphPathSinkAction::Reject _reason:
            false
        GuiSfntSimpleGlyphPathSinkAction::CloseContour _close:
            false
        GuiSfntSimpleGlyphPathSinkAction::NoAction:
            false

fn next_is_tail %fn &GuiSfntSimpleGlyphPathSinkActionStep bool \step:
    let next %GuiSfntSimpleGlyphPathSinkActionNext gui_sfnt_simple_glyph_path_sink_action_step_next step
    match next:
        GuiSfntSimpleGlyphPathSinkActionNext::EndContour:
            false
        GuiSfntSimpleGlyphPathSinkActionNext::Continue next_cursor:
            match gui_sfnt_simple_glyph_path_sink_action_cursor_action_slot &next_cursor:
                GuiSfntSimpleGlyphPathSinkActionSlot::Primary:
                    false
                GuiSfntSimpleGlyphPathSinkActionSlot::Tail:
                    true

fn action_step_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 514
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    match build_line_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection:
            let start_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_start_cursor glyph 0
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step &collection start_cursor &policy:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok primary_step:
                    let primary_ok %bool action_is_emit_event &primary_step
                    let primary_next_tail %bool next_is_tail &primary_step
                    let tail_contour_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_sink_action_cursor_contour_cursor &start_cursor
                    let tail_cursor %GuiSfntSimpleGlyphPathSinkActionCursor gui_sfnt_simple_glyph_path_sink_action_cursor tail_contour_cursor GuiSfntSimpleGlyphPathSinkActionSlot::Tail
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_action_step &collection tail_cursor &policy:
                        Result::Err _error:
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            false
                        Result::Ok tail_step:
                            let tail_ok %bool not action_is_emit_event &tail_step
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            and primary_ok and primary_next_tail tail_ok

fn main %impure fn void i32 \void:
    let ok1 %bool action_step_contract_ok
    test_assertion_exit_code assert "point stream item collection path sink action step smoke" ok1
```
