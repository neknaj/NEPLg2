# GUI font SFNT glyf outline point stream item collection path command pair doctests

このファイルは、F5z の collection-backed path command pair lookup が F5y curve segment lookup と既存 pure path command pair projection だけを通ることを固定する。

source policy coverage labels:

- path_command_pair_line_ok
- path_command_pair_quadratic_ok
- path_command_pair_no_segment_skip_ok
- path_command_pair_curve_error_propagates_ok
- no_vec_no_fallback_no_sink_traversal

## point stream item collection path command pair smoke

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

fn line_pair_contract_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 510
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match build_line_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_path_command_pair &collection 0 0:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok pair:
                    let move_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_move_command &pair
                    let draw_command %GuiSfntSimpleGlyphPathCommand gui_sfnt_simple_glyph_path_command_pair_draw_command &pair
                    let move_ok %bool match move_command:
                        GuiSfntSimpleGlyphPathCommand::MoveTo move_to:
                            let ok_x %bool eq 0 gui_sfnt_simple_glyph_path_move_to_x2 &move_to
                            let ok_y %bool eq 0 gui_sfnt_simple_glyph_path_move_to_y2 &move_to
                            and ok_x ok_y
                        _:
                            false
                    let draw_ok %bool match draw_command:
                        GuiSfntSimpleGlyphPathCommand::LineTo line_to:
                            let ok_x %bool eq 16 gui_sfnt_simple_glyph_path_line_to_x2 &line_to
                            let ok_y %bool eq 8 gui_sfnt_simple_glyph_path_line_to_y2 &line_to
                            and ok_x ok_y
                        _:
                            false
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and move_ok draw_ok

fn main %impure fn void i32 \void:
    let ok1 %bool line_pair_contract_ok
    test_assertion_exit_code assert "point stream item collection path command pair smoke" ok1
```
