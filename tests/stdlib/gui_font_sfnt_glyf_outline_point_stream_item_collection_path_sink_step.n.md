# GUI font SFNT glyf outline point stream item collection path sink step doctests

このファイルは、F5af の collection-backed sink step lookup が F5ae contour step lookup と pure sink-step projection だけを通ることを固定する。

source policy coverage labels:

- path_sink_step_primary_line_ok
- path_sink_step_tail_close_ok
- path_sink_step_error_propagates_ok
- path_sink_step_no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path sink step smoke

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

fn primary_is_emit %fn &GuiSfntSimpleGlyphPathSinkStep bool \sink_step:
    let primary %GuiSfntSimpleGlyphPathSinkPrimaryAction gui_sfnt_simple_glyph_path_sink_step_primary_action sink_step
    match primary:
        GuiSfntSimpleGlyphPathSinkPrimaryAction::EmitEvent _event:
            true
        GuiSfntSimpleGlyphPathSinkPrimaryAction::Reject _reason:
            false

fn tail_is_close %fn &GuiSfntSimpleGlyphPathSinkStep bool \sink_step:
    let tail %GuiSfntSimpleGlyphPathSinkTailAction gui_sfnt_simple_glyph_path_sink_step_tail_action sink_step
    match tail:
        GuiSfntSimpleGlyphPathSinkTailAction::NoTailAction:
            false
        GuiSfntSimpleGlyphPathSinkTailAction::CloseContour _close:
            true

fn sink_step_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 513
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    let policy %GuiSfntSimpleGlyphPathSinkPolicy gui_sfnt_simple_glyph_path_sink_policy GuiSfntSimpleGlyphPathOffCurveStartPolicy::RejectUnsupported GuiSfntSimpleGlyphPathClosurePolicy::EmitCloseAfterFinalEvent
    match build_line_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection:
            let first_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step &collection first_cursor &policy:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok first_sink_step:
                    let first_primary_ok %bool primary_is_emit &first_sink_step
                    let first_tail_ok %bool not tail_is_close &first_sink_step
                    let source_step %GuiSfntSimpleGlyphPathContourStep gui_sfnt_simple_glyph_path_sink_step_source_step &first_sink_step
                    let next %GuiSfntSimpleGlyphPathContourNext gui_sfnt_simple_glyph_path_contour_step_next &source_step
                    match next:
                        GuiSfntSimpleGlyphPathContourNext::EndContour:
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            false
                        GuiSfntSimpleGlyphPathContourNext::Continue second_cursor:
                            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_step &collection second_cursor &policy:
                                Result::Err _error:
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                                    false
                                Result::Ok second_sink_step:
                                    let second_primary_ok %bool primary_is_emit &second_sink_step
                                    let second_tail_ok %bool tail_is_close &second_sink_step
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                                    and first_primary_ok and first_tail_ok and second_primary_ok second_tail_ok

fn main %impure fn void i32 \void:
    let ok1 %bool sink_step_contract_ok
    test_assertion_exit_code assert "point stream item collection path sink step smoke" ok1
```
