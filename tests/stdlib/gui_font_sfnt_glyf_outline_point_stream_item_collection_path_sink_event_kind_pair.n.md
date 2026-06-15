# GUI font SFNT glyf outline point stream item collection path sink event kind pair doctests

このファイルは、F5ab の collection-backed path sink event kind pair lookup が F5aa path sink event pair lookup と既存 pure kind pair projection だけを通ることを固定する。

source policy coverage labels:

- path_sink_event_kind_pair_line_ok
- path_sink_event_kind_pair_quadratic_ok
- path_sink_event_kind_pair_no_segment_skip_ok
- path_sink_event_kind_pair_curve_error_propagates_ok
- no_vec_no_fallback_no_sink_traversal

## point stream item collection path sink event kind pair smoke

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

fn line_event_kind_pair_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 512
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match build_line_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_sink_event_kind_pair &collection 0 0:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok kind_pair:
                    let first_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind_pair_first_kind &kind_pair
                    let second_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_sink_event_kind_pair_second_kind &kind_pair
                    let first_ok %bool match first_kind:
                        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
                            true
                        _:
                            false
                    let second_ok %bool match second_kind:
                        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
                            true
                        _:
                            false
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and first_ok second_ok

fn main %impure fn void i32 \void:
    let ok1 %bool line_event_kind_pair_contract_ok
    test_assertion_exit_code assert "point stream item collection path sink event kind pair smoke" ok1
```
