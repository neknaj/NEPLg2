# GUI font SFNT glyf outline point stream item collection contour span doctests

このファイルは、F5v の collection-backed contour span lookup が partial collection や forged endpoint topology を成功値にしないことを検査する。

## point stream item collection contour span validates full endpoint topology

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

fn build_collection3 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool impure fn bool impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0\e1\e2:
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
                            push_item_or_free collection2 make_item glyph 2 e2

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

fn span_success_two_contours_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 401
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection 0:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok span0:
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection 1:
                        Result::Err _error:
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            false
                        Result::Ok span1:
                            let span0_start_ok %bool eq 0 gui_sfnt_simple_glyph_contour_span_start_point_index &span0
                            let span0_end_ok %bool eq 1 gui_sfnt_simple_glyph_contour_span_end_point_index &span0
                            let span0_count_ok %bool eq 2 gui_sfnt_simple_glyph_contour_span_point_count &span0
                            let span1_start_ok %bool eq 2 gui_sfnt_simple_glyph_contour_span_start_point_index &span1
                            let span1_end_ok %bool eq 3 gui_sfnt_simple_glyph_contour_span_end_point_index &span1
                            let span1_count_ok %bool eq 2 gui_sfnt_simple_glyph_contour_span_point_count &span1
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            and and span0_start_ok span0_end_ok and span0_count_ok and span1_start_ok and span1_end_ok span1_count_ok

fn span_partial_collection_rejected_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 402
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match alloc_collection &capacity:
        Result::Err _message:
            false
        Result::Ok collection0:
            match push_item_or_free collection0 make_item glyph 0 false:
                Result::Err _message:
                    false
                Result::Ok collection1:
                    match push_item_or_free collection1 make_item glyph 1 true:
                        Result::Err _message:
                            false
                        Result::Ok collection2:
                            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection2 0:
                                Result::Ok _span:
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection2
                                    false
                                Result::Err error:
                                    let kind_ok %bool contour_span_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::CollectionIncomplete
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection2
                                    kind_ok

fn span_contour_index_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 403
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection 2:
                Result::Ok _span:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_span_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourIndexOutOfRange
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    kind_ok

fn span_contour_count_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 404
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 3
    match build_collection3 &capacity false true true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection 0:
                Result::Ok _span:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_span_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourCountMismatch
                    let observed_ok %bool eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span_error_observed_contour_count &error
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok observed_ok

fn span_final_endpoint_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 405
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true true false:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection 0:
                Result::Ok _span:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_span_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::FinalContourEndMismatch
                    let last_ok %bool eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span_error_last_endpoint_index &error
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok last_ok

fn span_missing_contour_end_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 406
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false false false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_span &collection 1:
                Result::Ok _span:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_span_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::MissingContourEnd
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    kind_ok

fn main %impure fn void i32 \void:
    let ok0 %bool span_success_two_contours_ok
    let ok1 %bool span_partial_collection_rejected_ok
    let ok2 %bool span_contour_index_out_of_range_ok
    let ok3 %bool span_contour_count_mismatch_ok
    let ok4 %bool span_final_endpoint_mismatch_ok
    let ok5 %bool span_missing_contour_end_ok
    let all0 %bool and ok0 and ok1 ok2
    let all1 %bool and ok3 and ok4 ok5
    test_assertion_exit_code assert "point stream item collection contour span contract" and all0 all1
```
