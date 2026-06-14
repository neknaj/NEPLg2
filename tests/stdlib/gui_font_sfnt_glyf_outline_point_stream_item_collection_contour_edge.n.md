# GUI font SFNT glyf outline point stream item collection contour edge doctests

このファイルは、F5x の collection-backed contour edge lookup が F5v span authority と F5w point lookup を通して 1 本の edge だけを読むことを検査する。

## point stream item collection contour edge reads checked point pairs

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt/glyf" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn make_capacity %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphOutlineStorageCapacity \glyph\contours\points:
    gui_sfnt_simple_glyph_outline_storage_capacity glyph contours points points points mul points 2

fn make_item %fn GuiGlyphId fn i32 fn bool GuiSfntSimpleGlyphOutlinePointStreamItem \glyph\point_index\end_of_contour:
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph point_index add 10 point_index add 20 point_index true end_of_contour
    gui_sfnt_simple_glyph_outline_point_stream_item point

fn contour_edge_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::ContourSpanFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::ContourSpanFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::EdgeIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::EdgeIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::StartPointFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::StartPointFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::EndPointFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::EndPointFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::ContourEdgeInvariantInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::ContourEdgeInvariantInvalid:
                    true
                _:
                    false

fn contour_span_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::InvalidCapacity:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionLengthMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionLengthMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionCapacityMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionCapacityMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionIncomplete:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionIncomplete:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemGlyphMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemIndexMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemKindMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ItemKindMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::MissingContourEnd:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::MissingContourEnd:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourCountMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourCountMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::FinalContourEndMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::FinalContourEndMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourSpanInvariantInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourSpanInvariantInvalid:
                    true
                _:
                    false

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

fn build_collection1 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match alloc_collection capacity:
        Result::Err message:
            Result::Err message
        Result::Ok collection0:
            push_item_or_free collection0 make_item glyph 0 e0

fn build_collection4 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool impure fn bool impure fn bool impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0\e1\e2\e3:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match alloc_collection capacity:
        Result::Err message:
            Result::Err message
        Result::Ok collection0:
            match push_item_or_free collection0 make_item glyph 0 e0:
                Result::Err message:
                    Result::Err message
                Result::Ok collection1:
                    match push_item_or_free collection1 make_item glyph 1 e1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok collection2:
                            match push_item_or_free collection2 make_item glyph 2 e2:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok collection3:
                                    push_item_or_free collection3 make_item glyph 3 e3

fn edge_success_wraps_end_to_start_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 431
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge &collection 0 1:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok edge:
                    let edge_index_ok %bool eq 1 gui_sfnt_simple_glyph_contour_edge_index &edge
                    let next_ok %bool eq 0 gui_sfnt_simple_glyph_contour_edge_next_local_index &edge
                    let start %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_start &edge
                    let end %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_end &edge
                    let start_local_ok %bool eq 1 gui_sfnt_simple_glyph_contour_point_local_index &start
                    let end_local_ok %bool eq 0 gui_sfnt_simple_glyph_contour_point_local_index &end
                    let start_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &start
                    let end_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &end
                    let start_abs_ok %bool eq 1 gui_sfnt_simple_glyph_point_index &start_point
                    let end_abs_ok %bool eq 0 gui_sfnt_simple_glyph_point_index &end_point
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and and edge_index_ok next_ok and start_local_ok and end_local_ok and start_abs_ok end_abs_ok

fn edge_success_second_contour_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 432
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge &collection 1 0:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok edge:
                    let edge_index_ok %bool eq 0 gui_sfnt_simple_glyph_contour_edge_index &edge
                    let next_ok %bool eq 1 gui_sfnt_simple_glyph_contour_edge_next_local_index &edge
                    let start %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_start &edge
                    let end %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_end &edge
                    let start_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &start
                    let end_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &end
                    let start_abs_ok %bool eq 2 gui_sfnt_simple_glyph_point_index &start_point
                    let end_abs_ok %bool eq 3 gui_sfnt_simple_glyph_point_index &end_point
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and edge_index_ok and next_ok and start_abs_ok end_abs_ok

fn edge_one_point_self_wrap_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 433
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 1
    match build_collection1 &capacity true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge &collection 0 0:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok edge:
                    let edge_index_ok %bool eq 0 gui_sfnt_simple_glyph_contour_edge_index &edge
                    let next_ok %bool eq 0 gui_sfnt_simple_glyph_contour_edge_next_local_index &edge
                    let start %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_start &edge
                    let end %GuiSfntSimpleGlyphContourPoint gui_sfnt_simple_glyph_contour_edge_end &edge
                    let start_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &start
                    let end_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &end
                    let start_abs %i32 gui_sfnt_simple_glyph_point_index &start_point
                    let end_abs %i32 gui_sfnt_simple_glyph_point_index &end_point
                    let abs_ok %bool and eq 0 start_abs eq start_abs end_abs
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and edge_index_ok and next_ok abs_ok

fn edge_span_failure_wraps_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 434
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge &collection 2 0:
                Result::Ok _edge:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_edge_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::ContourSpanFailed
                    let span_error_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge_error_span_error &error:
                        Option::None:
                            false
                        Option::Some span_error:
                            contour_span_error_kind_is &span_error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourIndexOutOfRange
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok span_error_ok

fn edge_index_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 435
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge &collection 0 2:
                Result::Ok _edge:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_edge_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::EdgeIndexOutOfRange
                    let next_ok %bool eq -1 gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge_error_next_local_index &error
                    let span_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge_error_span &error:
                        Option::None:
                            false
                        Option::Some span:
                            eq 2 gui_sfnt_simple_glyph_contour_span_point_count &span
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok and next_ok span_ok

fn edge_topology_failure_wraps_final_endpoint_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 436
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true true false:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge &collection 0 0:
                Result::Ok _edge:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_edge_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourEdgeErrorKind::ContourSpanFailed
                    let span_error_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_edge_error_span_error &error:
                        Option::None:
                            false
                        Option::Some span_error:
                            contour_span_error_kind_is &span_error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::FinalContourEndMismatch
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok span_error_ok

fn main %impure fn void i32 \void:
    let ok0 %bool edge_success_wraps_end_to_start_ok
    let ok1 %bool edge_success_second_contour_ok
    let ok2 %bool edge_one_point_self_wrap_ok
    let ok3 %bool edge_span_failure_wraps_range_ok
    let ok4 %bool edge_index_out_of_range_ok
    let ok5 %bool edge_topology_failure_wraps_final_endpoint_ok
    let success_left %bool and ok0 and ok1 ok2
    let success_right %bool and ok3 and ok4 ok5
    test_assertion_exit_code assert "point stream item collection contour edge contract" and success_left success_right
```
