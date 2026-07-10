# GUI font SFNT simple glyph contour span index doctests

このファイルは、F5nv の contour span index が public classified collection owner を一度だけ順に読み、collection と O(1) index を同じ owner に保持することを検査する。

## contour span index validates public owner lifecycle and topology

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/sfnt" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn make_capacity %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphOutlineStorageCapacity \glyph\contours\points:
    gui_sfnt_simple_glyph_outline_storage_capacity glyph contours points points points mul points 2

fn make_item %fn GuiGlyphId fn i32 fn bool fn bool GuiSfntSimpleGlyphOutlinePointStreamItem \glyph\point_index\on_curve\end_of_contour:
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph point_index add 10 point_index add 20 point_index on_curve end_of_contour
    gui_sfnt_simple_glyph_outline_point_stream_item point

fn collection_alloc %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity:
    let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count capacity
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit point_count
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc capacity &limit:
        Result::Ok collection:
            Result::Ok collection
        Result::Err _error:
            Result::Err "alloc"

fn collection_push %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn GuiSfntSimpleGlyphOutlinePointStreamItem Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \collection\item:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection item:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
            Result::Err "push"

fn collection1 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match collection_alloc capacity:
        Result::Err message:
            Result::Err message
        Result::Ok c0:
            collection_push c0 make_item glyph 0 true e0

fn collection2 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0\e1:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match collection_alloc capacity:
        Result::Err message:
            Result::Err message
        Result::Ok c0:
            match collection_push c0 make_item glyph 0 true e0:
                Result::Err message:
                    Result::Err message
                Result::Ok c1:
                    collection_push c1 make_item glyph 1 false e1

fn collection3 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool impure fn bool impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0\e1\e2:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match collection_alloc capacity:
        Result::Err message:
            Result::Err message
        Result::Ok c0:
            match collection_push c0 make_item glyph 0 true e0:
                Result::Err message:
                    Result::Err message
                Result::Ok c1:
                    match collection_push c1 make_item glyph 1 false e1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok c2:
                            collection_push c2 make_item glyph 2 true e2

fn collection4 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool impure fn bool impure fn bool impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0\e1\e2\e3:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match collection_alloc capacity:
        Result::Err message:
            Result::Err message
        Result::Ok c0:
            match collection_push c0 make_item glyph 0 true e0:
                Result::Err message:
                    Result::Err message
                Result::Ok c1:
                    match collection_push c1 make_item glyph 1 false e1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok c2:
                            match collection_push c2 make_item glyph 2 true e2:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok c3:
                                    collection_push c3 make_item glyph 3 false e3

fn collection5 %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn bool impure fn bool impure fn bool impure fn bool impure fn bool Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\e0\e1\e2\e3\e4:
    let glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph capacity
    match collection_alloc capacity:
        Result::Err message:
            Result::Err message
        Result::Ok c0:
            match collection_push c0 make_item glyph 0 true e0:
                Result::Err message:
                    Result::Err message
                Result::Ok c1:
                    match collection_push c1 make_item glyph 1 false e1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok c2:
                            match collection_push c2 make_item glyph 2 true e2:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok c3:
                                    match collection_push c3 make_item glyph 3 false e3:
                                        Result::Err message:
                                            Result::Err message
                                        Result::Ok c4:
                                            collection_push c4 make_item glyph 4 true e4

fn start_kind_is %fn &GuiSfntSimpleGlyphContourSpanIndexStartError fn GuiSfntSimpleGlyphContourSpanIndexStartErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphContourSpanIndexStartErrorKind gui_sfnt_simple_glyph_contour_span_index_start_error_kind error
    match observed:
        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::InvalidCapacity: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CollectionCountMismatch:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CollectionCountMismatch: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CollectionStorageMismatch:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CollectionStorageMismatch: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::InvalidLimit:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::InvalidLimit: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CapacityRejected:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CapacityRejected: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::SpanStorageAllocFailed:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::SpanStorageAllocFailed: true
                _: false

fn step_kind_is %fn &GuiSfntSimpleGlyphContourSpanIndexStepError fn GuiSfntSimpleGlyphContourSpanIndexStepErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphContourSpanIndexStepErrorKind gui_sfnt_simple_glyph_contour_span_index_step_error_kind error
    match observed:
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::CollectionReadFailed:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::CollectionReadFailed: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::PointIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::PointIndexMismatch: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ItemGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ItemGlyphMismatch: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ItemKindMismatch:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ItemKindMismatch: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ContourCountExceeded:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ContourCountExceeded: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::InvalidSpanRange:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::InvalidSpanRange: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::SpanStoragePushFailed:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::SpanStoragePushFailed: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::MissingFinalEndpoint:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::MissingFinalEndpoint: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ContourCountMismatch:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ContourCountMismatch: true
                _: false
        GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::CompletionInvariantInvalid:
            match expected:
                GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::CompletionInvariantInvalid: true
                _: false

fn span_matches %fn &GuiSfntSimpleGlyphContourSpan fn i32 fn i32 fn i32 fn i32 bool \span\contour\start\end\count:
    and eq contour gui_sfnt_simple_glyph_contour_span_index span and eq start gui_sfnt_simple_glyph_contour_span_start_point_index span and eq end gui_sfnt_simple_glyph_contour_span_end_point_index span eq count gui_sfnt_simple_glyph_contour_span_point_count span

fn completed_two_spans_match %impure fn GuiSfntSimpleGlyphContourSpanIndexedCollectionOwner impure fn i32 impure fn i32 impure fn i32 impure fn i32 bool \owner\end0\count0\end1\count1:
    match gui_sfnt_simple_glyph_contour_span_index_lookup &owner 0:
        Result::Err _error:
            gui_sfnt_simple_glyph_contour_span_indexed_collection_free owner
            false
        Result::Ok span0:
            match gui_sfnt_simple_glyph_contour_span_index_lookup &owner 1:
                Result::Err _error:
                    gui_sfnt_simple_glyph_contour_span_indexed_collection_free owner
                    false
                Result::Ok span1:
                    let span0_ok %bool span_matches &span0 0 0 end0 count0
                    let span1_ok %bool span_matches &span1 1 add end0 1 end1 count1
                    let count_ok %bool eq 2 gui_sfnt_simple_glyph_contour_span_indexed_collection_span_count &owner
                    gui_sfnt_simple_glyph_contour_span_indexed_collection_free owner
                    and count_ok and span0_ok span1_ok

fn finish_two_spans %impure fn GuiSfntSimpleGlyphContourSpanIndexBuilderOwner impure fn i32 impure fn i32 impure fn i32 impure fn i32 bool \owner\end0\count0\end1\count1:
    match gui_sfnt_simple_glyph_contour_span_index_step owner:
        Result::Err error:
            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
            false
        Result::Ok next:
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &next
            let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
            if eq point_count gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &next:
                then:
                    match gui_sfnt_simple_glyph_contour_span_index_complete next:
                        Result::Err error:
                            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
                            false
                        Result::Ok completed:
                            completed_two_spans_match completed end0 count0 end1 count1
                else:
                    finish_two_spans next end0 count0 end1 count1

fn finish_single_span %impure fn GuiSfntSimpleGlyphContourSpanIndexBuilderOwner impure fn i32 impure fn i32 bool \owner\end0\count0:
    match gui_sfnt_simple_glyph_contour_span_index_step owner:
        Result::Err error:
            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
            false
        Result::Ok next:
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &next
            let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
            if eq point_count gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &next:
                then:
                    match gui_sfnt_simple_glyph_contour_span_index_complete next:
                        Result::Err error:
                            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
                            false
                        Result::Ok completed:
                            match gui_sfnt_simple_glyph_contour_span_index_lookup &completed 0:
                                Result::Err _error:
                                    gui_sfnt_simple_glyph_contour_span_indexed_collection_free completed
                                    false
                                Result::Ok span:
                                    let ok %bool and eq 1 gui_sfnt_simple_glyph_contour_span_indexed_collection_span_count &completed span_matches &span 0 0 end0 count0
                                    gui_sfnt_simple_glyph_contour_span_indexed_collection_free completed
                                    ok
                else:
                    finish_single_span next end0 count0

fn span_index_equal_contours_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 700
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match collection4 &capacity false true false true:
        Result::Err _message: false
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 2
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    false
                Result::Ok owner:
                    finish_two_spans owner 1 2 3 2

fn span_index_unequal_contours_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 701
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 5
    match collection5 &capacity false true false false true:
        Result::Err _message: false
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 2
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    false
                Result::Ok owner:
                    finish_two_spans owner 1 2 4 3

fn span_index_single_contour_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 702
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 3
    match collection3 &capacity false false true:
        Result::Err _message: false
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 1
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    false
                Result::Ok owner:
                    finish_single_span owner 2 3

fn step_error_progress_is %impure fn GuiSfntSimpleGlyphContourSpanIndexStepError impure fn GuiSfntSimpleGlyphContourSpanIndexStepErrorKind impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn bool bool \error\expected\point\contour\contour_start\spans\expected_value\actual_value\needs_item:
    let kind_ok %bool step_kind_is &error expected
    let rejected_item %Option GuiSfntSimpleGlyphOutlinePointStreamItem gui_sfnt_simple_glyph_contour_span_index_step_error_rejected_item &error
    let expected_ok %bool eq expected_value gui_sfnt_simple_glyph_contour_span_index_step_error_expected_value &error
    let actual_ok %bool eq actual_value gui_sfnt_simple_glyph_contour_span_index_step_error_actual_value &error
    let owner %GuiSfntSimpleGlyphContourSpanIndexBuilderOwner gui_sfnt_simple_glyph_contour_span_index_step_error_take_owner error
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &owner
    let capacity_glyph %GuiGlyphId gui_sfnt_simple_glyph_outline_storage_capacity_glyph &capacity
    let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
    let item_ok %bool match rejected_item:
        Option::Some item:
            let rejected_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
            let rejected_glyph %GuiGlyphId gui_sfnt_simple_glyph_point_glyph &rejected_point
            and needs_item and eq point gui_sfnt_simple_glyph_point_index &rejected_point and eq gui_glyph_id_raw &capacity_glyph gui_glyph_id_raw &rejected_glyph gui_sfnt_simple_glyph_outline_point_stream_item_kind_matches_point &item
        Option::None: not needs_item
    let point_ok %bool eq point gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &owner
    let contour_ok %bool eq contour gui_sfnt_simple_glyph_contour_span_index_builder_next_contour_index &owner
    let contour_start_ok %bool eq contour_start gui_sfnt_simple_glyph_contour_span_index_builder_contour_start_index &owner
    let spans_ok %bool eq spans gui_sfnt_simple_glyph_contour_span_index_builder_span_count &owner
    let read_index %i32 if lt point point_count:
        then: point
        else: sub point_count 1
    let collection_readable %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &owner read_index:
        Result::Ok item:
            let read_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
            eq read_index gui_sfnt_simple_glyph_point_index &read_point
        Result::Err _error: false
    gui_sfnt_simple_glyph_contour_span_index_builder_free owner
    and kind_ok and item_ok and expected_ok and actual_ok and point_ok and contour_ok and contour_start_ok and spans_ok collection_readable

fn finish_expect_error %impure fn GuiSfntSimpleGlyphContourSpanIndexBuilderOwner impure fn GuiSfntSimpleGlyphContourSpanIndexStepErrorKind impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn bool bool \owner\kind\point\contour\contour_start\spans\expected_value\actual_value\needs_item:
    match gui_sfnt_simple_glyph_contour_span_index_step owner:
        Result::Err error:
            step_error_progress_is error kind point contour contour_start spans expected_value actual_value needs_item
        Result::Ok next:
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &next
            let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
            if eq point_count gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &next:
                then:
                    match gui_sfnt_simple_glyph_contour_span_index_complete next:
                        Result::Err error:
                            step_error_progress_is error kind point contour contour_start spans expected_value actual_value needs_item
                        Result::Ok completed:
                            gui_sfnt_simple_glyph_contour_span_indexed_collection_free completed
                            false
                else:
                    finish_expect_error next kind point contour contour_start spans expected_value actual_value needs_item

fn finish_and_free_error %impure fn GuiSfntSimpleGlyphContourSpanIndexBuilderOwner impure fn GuiSfntSimpleGlyphContourSpanIndexStepErrorKind bool \owner\kind:
    match gui_sfnt_simple_glyph_contour_span_index_step owner:
        Result::Err error:
            let ok %bool step_kind_is &error kind
            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
            ok
        Result::Ok next:
            let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_point_stream_item_collection_capacity gui_sfnt_simple_glyph_contour_span_index_builder_collection_ref &next
            let point_count %i32 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
            if eq point_count gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &next:
                then:
                    match gui_sfnt_simple_glyph_contour_span_index_complete next:
                        Result::Err error:
                            let ok %bool step_kind_is &error kind
                            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
                            ok
                        Result::Ok completed:
                            gui_sfnt_simple_glyph_contour_span_indexed_collection_free completed
                            false
                else:
                    finish_and_free_error next kind

fn finish_one_point_completion %impure fn GuiSfntSimpleGlyphContourSpanIndexBuilderOwner bool \owner:
    match gui_sfnt_simple_glyph_contour_span_index_step owner:
        Result::Err error:
            gui_sfnt_simple_glyph_contour_span_index_step_error_free error
            false
        Result::Ok ready:
            let progress_ok %bool and eq 1 gui_sfnt_simple_glyph_contour_span_index_builder_next_point_index &ready and eq 1 gui_sfnt_simple_glyph_contour_span_index_builder_next_contour_index &ready eq 1 gui_sfnt_simple_glyph_contour_span_index_builder_span_count &ready
            match gui_sfnt_simple_glyph_contour_span_index_complete ready:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_step_error_free error
                    false
                Result::Ok completed:
                    gui_sfnt_simple_glyph_contour_span_indexed_collection_free completed
                    progress_ok

fn expect_topology_error %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn i32 impure fn GuiSfntSimpleGlyphContourSpanIndexStepErrorKind impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn i32 impure fn bool bool \collection\max_contours\kind\point\contour\contour_start\spans\expected_value\actual_value\needs_item:
    let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit max_contours
    match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
        Result::Err error:
            gui_sfnt_simple_glyph_contour_span_index_start_error_free error
            false
        Result::Ok owner:
            finish_expect_error owner kind point contour contour_start spans expected_value actual_value needs_item

fn span_index_extra_endpoint_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 703
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match collection2 &capacity true true:
        Result::Err _message: false
        Result::Ok collection:
            expect_topology_error collection 1 GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ContourCountExceeded 1 1 1 1 1 2 true

fn span_index_contour_count_mismatch_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 704
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 3
    match collection3 &capacity false false true:
        Result::Err _message: false
        Result::Ok collection:
            expect_topology_error collection 2 GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::ContourCountMismatch 3 1 3 1 2 1 false

fn span_index_missing_final_endpoint_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 705
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match collection2 &capacity false false:
        Result::Err _message: false
        Result::Ok collection:
            expect_topology_error collection 1 GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::MissingFinalEndpoint 2 0 0 0 2 0 false

fn span_index_collection_count_mismatch_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 706
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 2
    match collection_alloc &capacity:
        Result::Err _message: false
        Result::Ok empty:
            match collection_push empty make_item glyph 0 true false:
                Result::Err _message: false
                Result::Ok partial:
                    let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 1
                    match gui_sfnt_simple_glyph_contour_span_index_start partial &limit:
                        Result::Ok owner:
                            gui_sfnt_simple_glyph_contour_span_index_builder_free owner
                            false
                        Result::Err error:
                            let kind_ok %bool start_kind_is &error GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CollectionCountMismatch
                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_contour_span_index_start_error_take_collection error
                            let read_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item &recovered 0:
                                Result::Ok _item: true
                                Result::Err _read_error: false
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                            and kind_ok read_ok

fn valid_collection_start_error %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn GuiSfntSimpleGlyphContourSpanIndexLimit impure fn GuiSfntSimpleGlyphContourSpanIndexStartErrorKind bool \collection\limit\kind:
    match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
        Result::Ok owner:
            gui_sfnt_simple_glyph_contour_span_index_builder_free owner
            false
        Result::Err error:
            let kind_ok %bool start_kind_is &error kind
            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_contour_span_index_start_error_take_collection error
            let read_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item &recovered 0:
                Result::Ok _item: true
                Result::Err _read_error: false
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
            and kind_ok read_ok

fn span_index_invalid_limit_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 707
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 1
    match collection1 &capacity true:
        Result::Err _message: false
        Result::Ok collection:
            valid_collection_start_error collection gui_sfnt_simple_glyph_contour_span_index_limit 0 GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::InvalidLimit

fn span_index_limit_rejection_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 708
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2 4
    match collection4 &capacity false true false true:
        Result::Err _message: false
        Result::Ok collection:
            valid_collection_start_error collection gui_sfnt_simple_glyph_contour_span_index_limit 1 GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::CapacityRejected

fn span_index_start_error_free_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 709
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 1
    match collection1 &capacity true:
        Result::Err _message: false
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 0
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Ok owner:
                    gui_sfnt_simple_glyph_contour_span_index_builder_free owner
                    false
                Result::Err error:
                    let ok %bool start_kind_is &error GuiSfntSimpleGlyphContourSpanIndexStartErrorKind::InvalidLimit
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    ok

fn span_index_step_error_free_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 710
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 1
    match collection1 &capacity false:
        Result::Err _message: false
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 1
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    false
                Result::Ok owner:
                    finish_and_free_error owner GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::MissingFinalEndpoint

fn span_index_completion_without_progress_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 711
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1 1
    match collection1 &capacity true:
        Result::Err _message: false
        Result::Ok collection:
            let limit %GuiSfntSimpleGlyphContourSpanIndexLimit gui_sfnt_simple_glyph_contour_span_index_limit 1
            match gui_sfnt_simple_glyph_contour_span_index_start collection &limit:
                Result::Err error:
                    gui_sfnt_simple_glyph_contour_span_index_start_error_free error
                    false
                Result::Ok owner:
                    match gui_sfnt_simple_glyph_contour_span_index_complete owner:
                        Result::Ok completed:
                            gui_sfnt_simple_glyph_contour_span_indexed_collection_free completed
                            false
                        Result::Err error:
                            step_error_progress_is error GuiSfntSimpleGlyphContourSpanIndexStepErrorKind::CompletionInvariantInvalid 0 0 0 0 1 0 false

fn main %impure fn void i32 \void:
    let ok0 %bool span_index_equal_contours_ok
    let ok1 %bool span_index_unequal_contours_ok
    let ok2 %bool span_index_single_contour_ok
    let ok3 %bool span_index_extra_endpoint_recovery_ok
    let ok4 %bool span_index_contour_count_mismatch_recovery_ok
    let ok5 %bool span_index_missing_final_endpoint_recovery_ok
    let ok6 %bool span_index_collection_count_mismatch_recovery_ok
    let ok7 %bool span_index_invalid_limit_recovery_ok
    let ok8 %bool span_index_limit_rejection_recovery_ok
    let ok9 %bool span_index_start_error_free_ok
    let ok10 %bool span_index_step_error_free_ok
    let ok11 %bool span_index_completion_without_progress_ok
    let all0 %bool and ok0 and ok1 ok2
    let all1 %bool and ok3 and ok4 ok5
    let all2 %bool and ok6 and ok7 ok8
    let all3 %bool and ok9 and ok10 ok11
    test_assertion_exit_code assert "F5nv contour span index contract" and and all0 all1 and all2 all3
```
