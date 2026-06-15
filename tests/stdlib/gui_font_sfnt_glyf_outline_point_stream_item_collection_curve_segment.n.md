# GUI font SFNT glyf outline point stream item collection curve segment doctests

このファイルは、F5y の collection-backed curve segment lookup が F5x edge authority と必要時だけの F5w lookahead lookup を通して 1 segment だけを分類することを検査する。

source policy coverage labels:

- curve_segment_line_without_lookahead_ok
- curve_segment_explicit_quadratic_with_lookahead_ok
- curve_segment_implied_midpoint_with_lookahead_ok
- curve_segment_single_point_no_segment_ok
- curve_segment_off_curve_start_no_segment_ok
- curve_segment_edge_failure_wraps_range_ok
- curve_segment_lookahead_wraps_contour_end_ok
- LookaheadPointFailed
- EdgeIndexOutOfRange
- SinglePointContour
- OffCurveStart

## point stream item collection curve segment smoke

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

fn build_collection3 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn i32 impure fn i32 impure fn bool impure fn i32 impure fn i32 impure fn bool impure fn i32 impure fn i32 impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\x0\y0\on0\x1\y1\on1\x2\y2\on2:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match alloc_collection capacity:
        Result::Err message:
            Result::Err message
        Result::Ok collection0:
            match push_item_or_free collection0 make_item glyph 0 x0 y0 on0 false:
                Result::Err message:
                    Result::Err message
                Result::Ok collection1:
                    match push_item_or_free collection1 make_item glyph 1 x1 y1 on1 false:
                        Result::Err message:
                            Result::Err message
                        Result::Ok collection2:
                            push_item_or_free collection2 make_item glyph 2 x2 y2 on2 true

fn quadratic_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 442
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 3
    match build_collection3 &capacity 0 0 true 5 3 false 8 7 true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_curve_segment &collection 0 0:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok segment:
                    let ok %bool match segment:
                        GuiSfntSimpleGlyphCurveSegment::Quadratic quadratic:
                            let lookahead %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_quadratic_segment_lookahead &quadratic
                            let lookahead_ok %bool eq 2 gui_sfnt_simple_glyph_contour_point_local_index &lookahead
                            let control_ok %bool and eq 10 gui_sfnt_simple_glyph_quadratic_segment_control_x2 &quadratic eq 6 gui_sfnt_simple_glyph_quadratic_segment_control_y2 &quadratic
                            let end_ok %bool and eq 16 gui_sfnt_simple_glyph_quadratic_segment_end_x2 &quadratic eq 14 gui_sfnt_simple_glyph_quadratic_segment_end_y2 &quadratic
                            let explicit_ok %bool not gui_sfnt_simple_glyph_quadratic_segment_end_is_implied &quadratic
                            let shape_ok %bool and control_ok end_ok
                            and lookahead_ok and shape_ok explicit_ok
                        _:
                            false
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    ok

fn main %impure fn void i32 \void:
    let ok1 %bool quadratic_contract_ok
    test_assertion_exit_code assert "point stream item collection curve segment smoke" ok1
```
