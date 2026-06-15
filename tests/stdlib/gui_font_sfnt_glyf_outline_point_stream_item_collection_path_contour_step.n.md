# GUI font SFNT glyf outline point stream item collection path contour step doctests

このファイルは、F5ae の collection-backed contour step lookup が collection span、cursor glyph identity、F5ad event lookup、pure event-kind projection、cursor next helper だけを通ることを固定する。

source policy coverage labels:

- path_contour_step_first_line_ok
- path_contour_step_second_line_ok
- path_contour_step_end_contour_ok
- path_contour_step_span_error_propagates_ok
- path_contour_step_cursor_glyph_mismatch_ok
- path_contour_step_event_error_propagates_ok
- no_vec_no_fallback_no_byte_backed_traversal

## point stream item collection path contour step smoke

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

fn kind_matches %fn GuiSfntSimpleGlyphPathSinkEventKind fn GuiSfntSimpleGlyphPathSinkEventKind bool \left\right:
    match left:
        GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
            match right:
                GuiSfntSimpleGlyphPathSinkEventKind::MoveTo:
                    true
                _:
                    false
        GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
            match right:
                GuiSfntSimpleGlyphPathSinkEventKind::LineTo:
                    true
                _:
                    false
        _:
            false

fn line_step_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 512
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match build_line_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection:
            let first_cursor %GuiSfntSimpleGlyphPathContourCursor gui_sfnt_simple_glyph_path_contour_cursor glyph 0 0 GuiSfntSimpleGlyphPathSinkEventSlot::First
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step &collection first_cursor:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok first_step:
                    let first_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_contour_step_kind &first_step
                    let first_kind_ok %bool kind_matches first_kind GuiSfntSimpleGlyphPathSinkEventKind::MoveTo
                    let first_next %GuiSfntSimpleGlyphPathContourNext gui_sfnt_simple_glyph_path_contour_step_next &first_step
                    match first_next:
                        GuiSfntSimpleGlyphPathContourNext::EndContour:
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            false
                        GuiSfntSimpleGlyphPathContourNext::Continue second_cursor:
                            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_contour_step &collection second_cursor:
                                Result::Err _error:
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                                    false
                                Result::Ok second_step:
                                    let second_kind %GuiSfntSimpleGlyphPathSinkEventKind gui_sfnt_simple_glyph_path_contour_step_kind &second_step
                                    let second_kind_ok %bool kind_matches second_kind GuiSfntSimpleGlyphPathSinkEventKind::LineTo
                                    let second_next %GuiSfntSimpleGlyphPathContourNext gui_sfnt_simple_glyph_path_contour_step_next &second_step
                                    let ended %bool match second_next:
                                        GuiSfntSimpleGlyphPathContourNext::EndContour:
                                            true
                                        GuiSfntSimpleGlyphPathContourNext::Continue _cursor:
                                            false
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                                    and first_kind_ok and second_kind_ok ended

fn main %impure fn void i32 \void:
    let ok1 %bool line_step_contract_ok
    test_assertion_exit_code assert "point stream item collection path contour step smoke" ok1
```
