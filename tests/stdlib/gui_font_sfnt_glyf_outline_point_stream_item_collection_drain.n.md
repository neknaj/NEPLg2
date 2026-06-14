# GUI font SFNT glyf outline point stream item collection drain doctests

このファイルは、F5u の classified point stream item collection drain が F5s を 0 / 1 budget だけで呼び、collection owner と cursor commit 位置を失わずに返すことを検査する。

## point stream item collection drain preserves owner and commit cursor

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: ""
```neplg2
#entry main
#indent 4
#target std

#import "alloc/io" as *
#import "alloc/gui/font/sfnt/glyf" as *
#import "alloc/gui/font/sfnt/metadata" as *
#import "core/gui/font" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn make_topology %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphTopology \glyph\contours\points:
    let bounds %GuiSfntGlyphBounds gui_sfnt_glyph_bounds glyph 0 0 10 12
    gui_sfnt_simple_glyph_topology glyph bounds contours points 0 0 0

fn make_stream %fn GuiSfntSimpleGlyphTopology fn i32 GuiSfntSimpleGlyphPointStream \topology\flag_length:
    gui_sfnt_simple_glyph_point_stream topology 0 flag_length 1000 0 1000 0 1000 0

fn make_capacity %fn GuiGlyphId fn i32 GuiSfntSimpleGlyphOutlineStorageCapacity \glyph\points:
    gui_sfnt_simple_glyph_outline_storage_capacity glyph 1 points points points mul points 2

fn push_region_scalar_or_free %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn GuiSfntSimpleGlyphOutlineScalarRegionCursor impure fn i32 Result GuiSfntSimpleGlyphOutlineRegionPush str \storage\cursor\value:
    match gui_sfnt_simple_glyph_outline_storage_push_region_scalar storage cursor value:
        Result::Ok pushed:
            Result::Ok pushed
        Result::Err error:
            let recovered %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_error_storage error
            gui_sfnt_simple_glyph_outline_storage_free recovered
            Result::Err "push_region_scalar"

fn push4_region_scalars %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn GuiSfntSimpleGlyphOutlineScalarRegionCursor impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result GuiSfntSimpleGlyphOutlineStorage str \storage\cursor\a\b\c\d:
    match push_region_scalar_or_free storage cursor a:
        Result::Err message:
            Result::Err message
        Result::Ok push_a:
            let cursor_b %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_a
            let storage_b %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_a
            match push_region_scalar_or_free storage_b cursor_b b:
                Result::Err message:
                    Result::Err message
                Result::Ok push_b:
                    let cursor_c %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_b
                    let storage_c %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_b
                    match push_region_scalar_or_free storage_c cursor_c c:
                        Result::Err message:
                            Result::Err message
                        Result::Ok push_c:
                            let cursor_d %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &push_c
                            let storage_d %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage push_c
                            match push_region_scalar_or_free storage_d cursor_d d:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok push_d:
                                    Result::Ok gui_sfnt_simple_glyph_outline_region_push_storage push_d

fn prepare_full_storage %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn &GuiSfntSimpleGlyphOutlineStorageLimit Result GuiSfntSimpleGlyphOutlineStorage str \capacity\limit:
    match gui_sfnt_simple_glyph_outline_storage_alloc capacity limit:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok storage0:
            match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::ContourEndpoint:
                Result::Err _cursor_error:
                    gui_sfnt_simple_glyph_outline_storage_free storage0
                    Result::Err "endpoint_cursor"
                Result::Ok endpoint_cursor:
                    match push_region_scalar_or_free storage0 endpoint_cursor 1:
                        Result::Err message:
                            Result::Err message
                        Result::Ok endpoint_push0:
                            let endpoint_cursor1 %GuiSfntSimpleGlyphOutlineScalarRegionCursor gui_sfnt_simple_glyph_outline_region_push_cursor &endpoint_push0
                            let storage1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage endpoint_push0
                            match push_region_scalar_or_free storage1 endpoint_cursor1 3:
                                Result::Err message:
                                    Result::Err message
                                Result::Ok endpoint_push1:
                                    let storage2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_region_push_storage endpoint_push1
                                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointX:
                                        Result::Err _cursor_error:
                                            gui_sfnt_simple_glyph_outline_storage_free storage2
                                            Result::Err "point_x_cursor"
                                        Result::Ok x_cursor:
                                            match push4_region_scalars storage2 x_cursor 10 15 15 15:
                                                Result::Err message:
                                                    Result::Err message
                                                Result::Ok storage3:
                                                    match gui_sfnt_simple_glyph_outline_scalar_region_cursor_try_from_capacity capacity GuiSfntSimpleGlyphOutlineScalarRegion::PointY:
                                                        Result::Err _cursor_error:
                                                            gui_sfnt_simple_glyph_outline_storage_free storage3
                                                            Result::Err "point_y_cursor"
                                                        Result::Ok y_cursor:
                                                            push4_region_scalars storage3 y_cursor 20 25 30 35

fn push_u8_or_free %impure fn ByteBuilder impure fn i32 Result ByteBuilder str \builder\byte:
    match byte_builder_push_u8 builder byte:
        Result::Ok next:
            Result::Ok next
        Result::Err error:
            byte_builder_error_free error
            Result::Err "push_u8"

fn finish_bytes %impure fn Result ByteBuilder str Result ByteBuf str \builder_result:
    match builder_result:
        Result::Err message:
            Result::Err message
        Result::Ok builder:
            match byte_builder_finish builder:
                Result::Err error:
                    byte_builder_error_free error
                    Result::Err "finish"
                Result::Ok bytes:
                    Result::Ok bytes

fn bytes4_result %impure fn i32 impure fn i32 impure fn i32 impure fn i32 Result ByteBuf str \a\b\c\d:
    match byte_builder_with_capacity 4:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            finish_bytes:
                match push_u8_or_free b0 a:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        match push_u8_or_free b1 b:
                            Result::Err message:
                                Result::Err message
                            Result::Ok b2:
                                match push_u8_or_free b2 c:
                                    Result::Err message:
                                        Result::Err message
                                    Result::Ok b3:
                                        push_u8_or_free b3 d

fn bytes2_result %impure fn i32 impure fn i32 Result ByteBuf str \a\b:
    match byte_builder_with_capacity 2:
        Result::Err _error:
            Result::Err "alloc"
        Result::Ok b0:
            finish_bytes:
                match push_u8_or_free b0 a:
                    Result::Err message:
                        Result::Err message
                    Result::Ok b1:
                        push_u8_or_free b1 b

fn kind_is %fn GuiSfntSimpleGlyphOutlinePointStreamItemKind fn GuiSfntSimpleGlyphOutlinePointStreamItemKind bool \observed\expected:
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OnCurve:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOnCurve:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve:
                    true
                _:
                    false

fn collection_drain_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::CollectionCursorMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::CollectionCursorMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::ItemDrainFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::ItemDrainFailed:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::ItemDrainInvariantInvalid:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::ItemDrainInvariantInvalid:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::CollectionPushFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::CollectionPushFailed:
                    true
                _:
                    false

fn push_error_kind_option_is_full %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainError bool \error:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_push_error_kind error:
        Option::None:
            false
        Option::Some kind:
            match kind:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionFull:
                    true
                _:
                    false

fn summary_cursor_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary fn i32 bool \summary\expected:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_cursor summary
    eq expected gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &cursor

fn summary_count_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary fn i32 bool \summary\expected:
    eq expected gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_items_read summary

fn summary_last_item_index_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary fn i32 bool \summary\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_last_item summary:
        Option::None:
            false
        Option::Some item:
            let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
            eq expected gui_sfnt_simple_glyph_point_index &point

fn summary_last_item_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary fn GuiSfntSimpleGlyphOutlinePointStreamItemKind bool \summary\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_last_item summary:
        Option::None:
            false
        Option::Some item:
            let kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
            kind_is kind expected

fn summary_last_item_is_none %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainSummary bool \summary:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_last_item summary:
        Option::None:
            true
        Option::Some item:
            let _kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
            false

fn collection_item_index_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollection fn i32 bool \collection\index:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item collection index:
        Result::Err _error:
            false
        Result::Ok item:
            let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
            eq index gui_sfnt_simple_glyph_point_index &point

fn alloc_collection_or_false %impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity impure fn i32 Result GuiSfntSimpleGlyphOutlinePointStreamItemCollection str \capacity\limit_items:
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit limit_items
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc capacity &limit:
        Result::Err _error:
            Result::Err "collection_alloc"
        Result::Ok collection:
            Result::Ok collection

fn item_collection_drain_full_end_ok %impure fn &ByteBuf impure fn GuiSfntTableRecord impure fn GuiSfntSimpleGlyphPointStream impure fn &GuiSfntSimpleGlyphOutlineStorage impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \bytes\glyf\stream\storage\capacity:
    match alloc_collection_or_false capacity 4:
        Result::Err _message:
            false
        Result::Ok collection:
            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection cursor 4:
                Result::Err error:
                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                    false
                Result::Ok drain:
                    match drain:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End summary:
                            let cursor_ok %bool summary_cursor_is &summary 4
                            let count_ok %bool summary_count_is &summary 4
                            let last_index_ok %bool summary_last_item_index_is &summary 3
                            let last_kind_ok %bool summary_last_item_kind_is &summary GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            let collection_count_ok %bool eq 4 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &collection1
                            let read_ok %bool collection_item_index_is &collection1 3
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection1
                            and cursor_ok and count_ok and last_index_ok and last_kind_ok and collection_count_ok read_ok
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted summary:
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection1
                            false

fn item_collection_drain_partial_budget_ok %impure fn &ByteBuf impure fn GuiSfntTableRecord impure fn GuiSfntSimpleGlyphPointStream impure fn &GuiSfntSimpleGlyphOutlineStorage impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \bytes\glyf\stream\storage\capacity:
    match alloc_collection_or_false capacity 4:
        Result::Err _message:
            false
        Result::Ok collection:
            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection cursor 2:
                Result::Err error:
                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                    false
                Result::Ok drain:
                    match drain:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End summary:
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection1
                            false
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted summary:
                            let cursor_ok %bool summary_cursor_is &summary 2
                            let count_ok %bool summary_count_is &summary 2
                            let last_index_ok %bool summary_last_item_index_is &summary 1
                            let last_kind_ok %bool summary_last_item_kind_is &summary GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            let collection_count_ok %bool eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &collection1
                            let read_ok %bool collection_item_index_is &collection1 1
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection1
                            and cursor_ok and count_ok and last_index_ok and last_kind_ok and collection_count_ok read_ok

fn item_collection_drain_zero_budget_nonterminal_ok %impure fn &ByteBuf impure fn GuiSfntTableRecord impure fn GuiSfntSimpleGlyphPointStream impure fn &GuiSfntSimpleGlyphOutlineStorage impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \bytes\glyf\stream\storage\capacity:
    match alloc_collection_or_false capacity 4:
        Result::Err _message:
            false
        Result::Ok collection:
            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection cursor 0:
                Result::Err error:
                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                    false
                Result::Ok drain:
                    match drain:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End summary:
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection1
                            false
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted summary:
                            let cursor_ok %bool summary_cursor_is &summary 0
                            let count_ok %bool summary_count_is &summary 0
                            let last_ok %bool summary_last_item_is_none &summary
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            let collection_count_ok %bool eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &collection1
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection1
                            and cursor_ok and count_ok and last_ok collection_count_ok

fn item_collection_drain_zero_budget_terminal_ok %impure fn &ByteBuf impure fn GuiSfntTableRecord impure fn GuiSfntSimpleGlyphPointStream impure fn &GuiSfntSimpleGlyphOutlineStorage impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \bytes\glyf\stream\storage\capacity:
    match alloc_collection_or_false capacity 4:
        Result::Err _message:
            false
        Result::Ok collection0:
            let start_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection0 start_cursor 4:
                Result::Err error:
                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                    false
                Result::Ok first_drain:
                    match first_drain:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted first_summary:
                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection first_summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                            false
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End first_summary:
                            let collection1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection first_summary
                            let terminal_cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 4
                            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection1 terminal_cursor 0:
                                Result::Err error:
                                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                                    false
                                Result::Ok second_drain:
                                    match second_drain:
                                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted second_summary:
                                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection second_summary
                                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                                            false
                                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End second_summary:
                                            let cursor_ok %bool summary_cursor_is &second_summary 4
                                            let count_ok %bool summary_count_is &second_summary 0
                                            let last_ok %bool summary_last_item_is_none &second_summary
                                            let collection2 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection second_summary
                                            let collection_count_ok %bool eq 4 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &collection2
                                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection2
                                            and cursor_ok and count_ok and last_ok collection_count_ok

fn item_collection_drain_cursor_mismatch_ok %impure fn &ByteBuf impure fn GuiSfntTableRecord impure fn GuiSfntSimpleGlyphPointStream impure fn &GuiSfntSimpleGlyphOutlineStorage impure fn &GuiSfntSimpleGlyphOutlineStorageCapacity bool \bytes\glyf\stream\storage\capacity:
    match alloc_collection_or_false capacity 4:
        Result::Err _message:
            false
        Result::Ok collection:
            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 4
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection cursor 0:
                Result::Ok drain:
                    match drain:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End summary:
                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                            false
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted summary:
                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                            false
                Result::Err error:
                    let kind_ok %bool collection_drain_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::CollectionCursorMismatch
                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                    let count_ok %bool eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &recovered
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                    and kind_ok count_ok

fn item_collection_drain_wraps_item_drain_failure_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 416
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 2
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 2
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_full_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes2_result 9 4:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            match alloc_collection_or_false &capacity 4:
                                Result::Err _message:
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    false
                                Result::Ok collection:
                                    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
                                    let result %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget &bytes glyf stream &storage collection cursor 1:
                                        Result::Ok drain:
                                            match drain:
                                                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End summary:
                                                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                                                    false
                                                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted summary:
                                                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                                                    false
                                        Result::Err error:
                                            let kind_ok %bool collection_drain_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::ItemDrainFailed
                                            let lower_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_item_drain_error &error:
                                                Option::None:
                                                    false
                                                Option::Some lower:
                                                    match gui_sfnt_simple_glyph_outline_point_stream_item_drain_error_kind &lower:
                                                        GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind::PointStepReadFailed:
                                                            true
                                                        _:
                                                            false
                                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                                            let count_ok %bool eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &recovered
                                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                                            and kind_ok and lower_ok count_ok
                                    io_bytebuf_free bytes
                                    gui_sfnt_simple_glyph_outline_storage_free storage
                                    result
        _:
            false

fn item_collection_drain_push_failure_ok %impure fn &ByteBuf impure fn GuiSfntTableRecord impure fn GuiSfntSimpleGlyphPointStream impure fn &GuiSfntSimpleGlyphOutlineStorage impure fn GuiGlyphId bool \bytes\glyf\stream\storage\glyph:
    let small_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1
    match alloc_collection_or_false &small_capacity 1:
        Result::Err _message:
            false
        Result::Ok collection:
            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_budget bytes glyf stream storage collection cursor 2:
                Result::Ok drain:
                    match drain:
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::End summary:
                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                            false
                        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrain::StepBudgetExhausted summary:
                            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_summary_collection summary
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                            false
                Result::Err error:
                    let kind_ok %bool collection_drain_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionDrainErrorKind::CollectionPushFailed
                    let push_kind_ok %bool push_error_kind_option_is_full &error
                    let rejected_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_rejected_item &error:
                        Option::None:
                            false
                        Option::Some rejected_item:
                            let rejected_point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &rejected_item
                            eq 1 gui_sfnt_simple_glyph_point_index &rejected_point
                    let item_drain_result_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_item_drain_result &error:
                        Option::None:
                            false
                        Option::Some item_drain_result:
                            match item_drain_result:
                                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::End _summary:
                                    false
                                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::StepBudgetExhausted _summary:
                                    true
                    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_cursor &error
                    let cursor_ok %bool eq 1 gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &cursor
                    let count_ok %bool eq 1 gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_items_read &error
                    let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_drain_error_collection error
                    let collection_count_ok %bool eq 1 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &recovered
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
                    and kind_ok and push_kind_ok and rejected_ok and item_drain_result_ok and cursor_ok and count_ok collection_count_ok

fn item_collection_drain_budget_states_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 411
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let stream %GuiSfntSimpleGlyphPointStream make_stream topology 4
    let glyf %GuiSfntTableRecord gui_sfnt_table_record 0 0 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match prepare_full_storage &capacity &limit:
                Result::Err _message:
                    false
                Result::Ok storage:
                    match bytes4_result 1 0 1 0:
                        Result::Err _message:
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            false
                        Result::Ok bytes:
                            let full_ok %bool item_collection_drain_full_end_ok &bytes glyf stream &storage &capacity
                            let partial_ok %bool item_collection_drain_partial_budget_ok &bytes glyf stream &storage &capacity
                            let zero_nonterminal_ok %bool item_collection_drain_zero_budget_nonterminal_ok &bytes glyf stream &storage &capacity
                            let zero_terminal_ok %bool item_collection_drain_zero_budget_terminal_ok &bytes glyf stream &storage &capacity
                            let cursor_mismatch_ok %bool item_collection_drain_cursor_mismatch_ok &bytes glyf stream &storage &capacity
                            let push_failure_ok %bool item_collection_drain_push_failure_ok &bytes glyf stream &storage glyph
                            io_bytebuf_free bytes
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and full_ok and partial_ok and zero_nonterminal_ok and zero_terminal_ok and cursor_mismatch_ok push_failure_ok
        _:
            false

fn main %impure fn void i32 \void:
    let budget_ok %bool item_collection_drain_budget_states_ok
    let wrap_ok %bool item_collection_drain_wraps_item_drain_failure_ok
    test_assertion_exit_code assert "point stream item collection drain contract" and budget_ok wrap_ok
```
