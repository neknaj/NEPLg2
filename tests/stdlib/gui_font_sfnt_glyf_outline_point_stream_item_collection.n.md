# GUI font SFNT glyf outline point stream item collection doctests

このファイルは、F5t の classified point stream item collection owner が専用 limit、owner-preserving push、typed read error を持ち、F5s drain や path/raster/render へ進まないことを検査する。

## point stream item collection validates owner allocation push and read

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

fn make_capacity %fn GuiGlyphId fn i32 GuiSfntSimpleGlyphOutlineStorageCapacity \glyph\points:
    gui_sfnt_simple_glyph_outline_storage_capacity glyph 1 points points points mul points 2

fn make_item %fn GuiGlyphId fn i32 fn bool fn bool GuiSfntSimpleGlyphOutlinePointStreamItem \glyph\point_index\on_curve\end_of_contour:
    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph point_index add 10 point_index add 20 point_index on_curve end_of_contour
    gui_sfnt_simple_glyph_outline_point_stream_item point

fn collection_alloc_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::InvalidCapacity:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::InvalidLimit:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::InvalidLimit:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::CapacityRejected:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::CapacityRejected:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::ItemStorageAllocFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::ItemStorageAllocFailed:
                    true
                _:
                    false

fn collection_push_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::InvalidCapacity:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionLengthMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionLengthMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionCapacityMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionCapacityMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionFull:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionFull:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemGlyphMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemGlyphMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemIndexMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemIndexMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemKindMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemKindMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemStoragePushFailed:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemStoragePushFailed:
                    true
                _:
                    false

fn collection_read_error_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadError fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::InvalidCapacity:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::CollectionLengthMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::CollectionLengthMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::CollectionCapacityMismatch:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::CollectionCapacityMismatch:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::ItemIndexOutOfRange:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::ItemIndexOutOfRange:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::ItemStorageMissing:
            match expected:
                GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::ItemStorageMissing:
                    true
                _:
                    false

fn item_collection_alloc_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 301
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 2
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Ok collection:
            let count_ok %bool eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &collection
            let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_point_stream_item_collection_items_len &collection
            let cap_ok %bool eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_items_cap &collection
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
            and count_ok and len_ok cap_ok
        Result::Err _error:
            false

fn item_collection_invalid_capacity_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 302
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_storage_capacity glyph 1 2 3 2 4
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 2
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Ok collection:
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
            false
        Result::Err error:
            collection_alloc_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::InvalidCapacity

fn item_collection_invalid_limit_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 303
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 0
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Ok collection:
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
            false
        Result::Err error:
            collection_alloc_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::InvalidLimit

fn item_collection_limit_reject_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 304
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 1
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Ok collection:
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
            false
        Result::Err error:
            collection_alloc_error_kind_is &error GuiSfntSimpleGlyphOutlinePointStreamItemCollectionAllocErrorKind::CapacityRejected

fn item_collection_push_read_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 305
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 2
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Err _error:
            false
        Result::Ok collection0:
            let item0 %GuiSfntSimpleGlyphOutlinePointStreamItem make_item glyph 0 true false
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection0 item0:
                Result::Err error0:
                    let recovered0 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error0
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered0
                    false
                Result::Ok collection1:
                    let item1 %GuiSfntSimpleGlyphOutlinePointStreamItem make_item glyph 1 false true
                    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection1 item1:
                        Result::Err error1:
                            let recovered1 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error1
                            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered1
                            false
                        Result::Ok collection2:
                            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item &collection2 1:
                                Result::Err _read_error:
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection2
                                    false
                                Result::Ok item:
                                    let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
                                    let kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
                                    let kind_ok %bool gui_sfnt_simple_glyph_outline_point_stream_item_kind_is kind GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
                                    let count_ok %bool eq 2 gui_sfnt_simple_glyph_outline_point_stream_item_collection_item_count &collection2
                                    let index_ok %bool eq 1 gui_sfnt_simple_glyph_point_index &point
                                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection2
                                    and kind_ok and count_ok index_ok

fn push_failure_is %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn GuiSfntSimpleGlyphOutlinePointStreamItem impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind bool \collection\item\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection item:
        Result::Ok next_collection:
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free next_collection
            false
        Result::Err error:
            let kind_ok %bool collection_push_error_kind_is &error expected
            let recovered %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered
            kind_ok

fn item_collection_glyph_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 306
    let other %GuiGlyphId unwrap_ok gui_glyph_id_result 307
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 2
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Err _error:
            false
        Result::Ok collection:
            let item %GuiSfntSimpleGlyphOutlinePointStreamItem make_item other 0 true false
            push_failure_is collection item GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemGlyphMismatch

fn item_collection_index_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 308
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 2
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Err _error:
            false
        Result::Ok collection:
            let item %GuiSfntSimpleGlyphOutlinePointStreamItem make_item glyph 1 true false
            push_failure_is collection item GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemIndexMismatch

fn item_collection_kind_mismatch_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 309
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 2
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 2
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Err _error:
            false
        Result::Ok collection:
            let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_point glyph 0 10 20 true false
            let item %GuiSfntSimpleGlyphOutlinePointStreamItem GuiSfntSimpleGlyphOutlinePointStreamItem point GuiSfntSimpleGlyphOutlinePointStreamItemKind::OffCurve
            push_failure_is collection item GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::ItemKindMismatch

fn item_collection_full_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 310
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 1
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Err _error:
            false
        Result::Ok collection0:
            let item0 %GuiSfntSimpleGlyphOutlinePointStreamItem make_item glyph 0 true true
            match gui_sfnt_simple_glyph_outline_point_stream_item_collection_push collection0 item0:
                Result::Err error0:
                    let recovered0 %GuiSfntSimpleGlyphOutlinePointStreamItemCollection gui_sfnt_simple_glyph_outline_point_stream_item_collection_push_error_collection error0
                    gui_sfnt_simple_glyph_outline_point_stream_item_collection_free recovered0
                    false
                Result::Ok collection1:
                    let item1 %GuiSfntSimpleGlyphOutlinePointStreamItem make_item glyph 1 true false
                    push_failure_is collection1 item1 GuiSfntSimpleGlyphOutlinePointStreamItemCollectionPushErrorKind::CollectionFull

fn read_failure_is %impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollection impure fn i32 impure fn GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind bool \collection\index\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_read_item &collection index:
        Result::Ok _item:
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
            false
        Result::Err error:
            let kind_ok %bool collection_read_error_kind_is &error expected
            gui_sfnt_simple_glyph_outline_point_stream_item_collection_free collection
            kind_ok

fn item_collection_read_out_of_range_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 311
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity make_capacity glyph 1
    let limit %GuiSfntSimpleGlyphOutlinePointStreamItemCollectionLimit gui_sfnt_simple_glyph_outline_point_stream_item_collection_limit 1
    match gui_sfnt_simple_glyph_outline_point_stream_item_collection_alloc &capacity &limit:
        Result::Err _error:
            false
        Result::Ok collection:
            read_failure_is collection 0 GuiSfntSimpleGlyphOutlinePointStreamItemCollectionReadErrorKind::ItemIndexOutOfRange

fn main %impure fn void i32 \void:
    let ok0 %bool item_collection_alloc_success_ok
    let ok1 %bool item_collection_invalid_capacity_ok
    let ok2 %bool item_collection_invalid_limit_ok
    let ok3 %bool item_collection_limit_reject_ok
    let ok4 %bool item_collection_push_read_success_ok
    let ok5 %bool item_collection_glyph_mismatch_ok
    let ok6 %bool item_collection_index_mismatch_ok
    let ok7 %bool item_collection_kind_mismatch_ok
    let ok8 %bool item_collection_full_ok
    let ok9 %bool item_collection_read_out_of_range_ok
    let all0 %bool and ok0 and ok1 ok2
    let all1 %bool and ok3 and ok4 ok5
    let all2 %bool and ok6 and ok7 ok8
    let all3 %bool and all0 all1
    let all4 %bool and all2 ok9
    let all_ok %bool and all3 all4
    test_assertion_exit_code assert "point stream item collection contract" all_ok
```
