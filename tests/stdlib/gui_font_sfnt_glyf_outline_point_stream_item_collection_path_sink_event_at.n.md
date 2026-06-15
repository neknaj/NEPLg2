# GUI font SFNT glyf outline point stream item collection path sink event at doctests

このファイルは、F5ad の collection-backed path sink event slot lookup が F5aa path sink event pair lookup と既存 pure typed slot projection だけを通ることを固定する。

source policy coverage labels:

- path_sink_event_at_first_line_ok
- path_sink_event_at_second_line_ok
- path_sink_event_at_no_segment_skip_ok
- path_sink_event_at_error_propagates_ok
- no_vec_no_fallback_no_sink_traversal

## point stream item collection path sink event at smoke

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

fn event_is_kind %fn &GuiSfntSimpleGlyphPathSinkEvent fn GuiSfntSimpleGlyphPathSinkEventKind bool \event\expected:
    let kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind event
    match expected:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            match kind:
                GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            match kind:
                GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
                    true
                _:
                    false
        _:
            false

fn line_event_slot_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 512
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match build_line_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at &collection 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok first_event:
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_at &collection 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::Second:
                        Result::Err _error:
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            false
                        Result::Ok second_event:
                            let first_ok %bool event_is_kind &first_event GuiSfntSimpleGlyphPathSinkEventKind::MoveTo
                            let second_ok %bool event_is_kind &second_event GuiSfntSimpleGlyphPathSinkEventKind::LineTo
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            and first_ok second_ok

fn main %impure fn void i32 \void:
    let ok1 %bool line_event_slot_contract_ok
    test_assertion_exit_code assert "point stream item collection path sink event at smoke" ok1
```
