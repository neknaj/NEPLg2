# GUI font SFNT glyf outline point stream item collection contour point doctests

このファイルは、F5w の collection-backed contour point lookup が F5v span authority と local index validation を通して 1 点だけ読むことを検査する。

## point stream item collection contour point reads checked local points

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

fn contour_point_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourSpanFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourSpanFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourPointIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourPointIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemReadFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemReadFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemGlyphMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemIndexMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemKindMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ItemKindMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourPointInvariantInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourPointInvariantInvalid:
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

fn point_success_two_contours_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 421
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point &collection 0 1:
                Result::Err _error:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Ok first_end:
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point &collection 1 0:
                        Result::Err _error:
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            false
                        Result::Ok second_start:
                            let first_local_ok %bool eq 1 gui_sfnt_simple_glyph_contour_point_local_index &first_end
                            let first_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &first_end
                            let first_abs_ok %bool eq 1 gui_sfnt_simple_glyph_point_index &first_point
                            let first_end_ok %bool gui_sfnt_simple_glyph_point_end_of_contour &first_point
                            let second_local_ok %bool eq 0 gui_sfnt_simple_glyph_contour_point_local_index &second_start
                            let second_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_contour_point_point &second_start
                            let second_abs_ok %bool eq 2 gui_sfnt_simple_glyph_point_index &second_point
                            let second_end_ok %bool not gui_sfnt_simple_glyph_point_end_of_contour &second_point
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                            and and first_local_ok first_abs_ok and first_end_ok and second_local_ok and second_abs_ok second_end_ok

fn point_span_failure_wraps_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 422
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point &collection 2 0:
                Result::Ok _point:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_point_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourSpanFailed
                    let span_error_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point_error_span_error &error:
                        Option::None:
                            false
                        Option::Some span_error:
                            contour_span_error_kind_is &span_error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::ContourIndexOutOfRange
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok span_error_ok

fn point_local_index_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 423
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true false true:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point &collection 0 2:
                Result::Ok _point:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_point_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourPointIndexOutOfRange
                    let absolute_ok %bool eq -1 gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point_error_absolute_point_index &error
                    let span_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point_error_span &error:
                        Option::None:
                            false
                        Option::Some span:
                            eq 2 gui_sfnt_simple_glyph_contour_span_point_count &span
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok and absolute_ok span_ok

fn point_topology_failure_wraps_final_endpoint_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 424
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match build_collection4 &capacity false true true false:
        Result::Err _message:
            false
        Result::Ok collection:
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point &collection 0 0:
                Result::Ok _point:
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    false
                Result::Err error:
                    let kind_ok %bool contour_point_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourPointErrorKind::ContourSpanFailed
                    let span_error_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_contour_point_error_span_error &error:
                        Option::None:
                            false
                        Option::Some span_error:
                            contour_span_error_kind_is &span_error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionContourSpanErrorKind::FinalContourEndMismatch
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
                    and kind_ok span_error_ok

fn main %impure fn void i32 \void:
    let ok0 %bool point_success_two_contours_ok
    let ok1 %bool point_span_failure_wraps_range_ok
    let ok2 %bool point_local_index_out_of_range_ok
    let ok3 %bool point_topology_failure_wraps_final_endpoint_ok
    test_assertion_exit_code assert "point stream item collection contour point contract" and and ok0 ok1 and ok2 ok3
```
