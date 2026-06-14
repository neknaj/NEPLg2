# GUI font SFNT glyf outline point stream item drain doctests

このファイルは、F5s の classified item drain が terminal-before-budget と budget-before-F5o/F5r の順序を守り、F5o と F5r の failure を別々の typed sub-error として保持することを検査する。

## point stream item drain respects budget and terminal state

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

fn summary_cursor_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary fn i32 bool \summary\expected:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary_cursor summary
    eq expected gui_sfnt_simple_glyph_outline_point_read_cursor_next_point_index &cursor

fn summary_count_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary fn i32 bool \summary\expected:
    eq expected gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary_items_read summary

fn summary_last_item_index_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary fn i32 bool \summary\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary_last_item summary:
        Option::None:
            false
        Option::Some item:
            let point %GuiSfntSimpleGlyphPoint gui_sfnt_simple_glyph_outline_point_stream_item_point &item
            eq expected gui_sfnt_simple_glyph_point_index &point

fn summary_last_item_kind_is %fn &GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary fn GuiSfntSimpleGlyphOutlinePointStreamItemKind bool \summary\expected:
    match gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary_last_item summary:
        Option::None:
            false
        Option::Some item:
            let kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
            kind_is kind expected

fn summary_last_item_is_none %fn &GuiSfntSimpleGlyphOutlinePointStreamItemDrainSummary bool \summary:
    match gui_sfnt_simple_glyph_outline_point_stream_item_drain_summary_last_item summary:
        Option::None:
            true
        Option::Some item:
            let _kind %GuiSfntSimpleGlyphOutlinePointStreamItemKind gui_sfnt_simple_glyph_outline_point_stream_item_kind &item
            false

fn item_drain_full_end_ok %fn &ByteBuf fn GuiSfntTableRecord fn GuiSfntSimpleGlyphPointStream fn &GuiSfntSimpleGlyphOutlineStorage bool \bytes\glyf\stream\storage:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
    match gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget bytes glyf stream storage cursor 4:
        Result::Err _error:
            false
        Result::Ok drain:
            match drain:
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::End summary:
                    and summary_cursor_is &summary 4 and summary_count_is &summary 4 and summary_last_item_index_is &summary 3 summary_last_item_kind_is &summary GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::StepBudgetExhausted _summary:
                    false

fn item_drain_partial_budget_exhausted_ok %fn &ByteBuf fn GuiSfntTableRecord fn GuiSfntSimpleGlyphPointStream fn &GuiSfntSimpleGlyphOutlineStorage bool \bytes\glyf\stream\storage:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
    match gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget bytes glyf stream storage cursor 2:
        Result::Err _error:
            false
        Result::Ok drain:
            match drain:
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::End _summary:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::StepBudgetExhausted summary:
                    and summary_cursor_is &summary 2 and summary_count_is &summary 2 and summary_last_item_index_is &summary 1 summary_last_item_kind_is &summary GuiSfntSimpleGlyphOutlinePointStreamItemKind::EndOffCurve

fn item_drain_zero_budget_nonterminal_ok %fn &ByteBuf fn GuiSfntTableRecord fn GuiSfntSimpleGlyphPointStream fn &GuiSfntSimpleGlyphOutlineStorage bool \bytes\glyf\stream\storage:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
    match gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget bytes glyf stream storage cursor 0:
        Result::Err _error:
            false
        Result::Ok drain:
            match drain:
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::End _summary:
                    false
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::StepBudgetExhausted summary:
                    and summary_cursor_is &summary 0 and summary_count_is &summary 0 summary_last_item_is_none &summary

fn item_drain_zero_budget_terminal_ok %fn &ByteBuf fn GuiSfntTableRecord fn GuiSfntSimpleGlyphPointStream fn &GuiSfntSimpleGlyphOutlineStorage bool \bytes\glyf\stream\storage:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 4
    match gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget bytes glyf stream storage cursor 0:
        Result::Err _error:
            false
        Result::Ok drain:
            match drain:
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::End summary:
                    and summary_cursor_is &summary 4 and summary_count_is &summary 0 summary_last_item_is_none &summary
                GuiSfntSimpleGlyphOutlinePointStreamItemDrain::StepBudgetExhausted _summary:
                    false

fn item_drain_cursor_too_far_ok %fn &ByteBuf fn GuiSfntTableRecord fn GuiSfntSimpleGlyphPointStream fn &GuiSfntSimpleGlyphOutlineStorage bool \bytes\glyf\stream\storage:
    let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 5
    match gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget bytes glyf stream storage cursor 4:
        Result::Ok _drain:
            false
        Result::Err error:
            match gui_sfnt_simple_glyph_outline_point_stream_item_drain_error_kind &error:
                GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind::CursorOutOfRange:
                    true
                _:
                    false

fn item_drain_wraps_point_step_read_failure_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 116
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
                            let cursor %GuiSfntSimpleGlyphOutlinePointReadCursor gui_sfnt_simple_glyph_outline_point_read_cursor 0
                            let result %bool match gui_sfnt_simple_glyph_outline_storage_read_point_stream_item_drain_budget &bytes glyf stream &storage cursor 1:
                                Result::Ok _drain:
                                    false
                                Result::Err error:
                                    let kind_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_drain_error_kind &error:
                                        GuiSfntSimpleGlyphOutlinePointStreamItemDrainErrorKind::PointStepReadFailed:
                                            true
                                        _:
                                            false
                                    let step_ok %bool match gui_sfnt_simple_glyph_outline_point_stream_item_drain_error_point_step_error &error:
                                        Option::None:
                                            false
                                        Option::Some step_error:
                                            let step_kind_ok %bool match gui_sfnt_simple_glyph_outline_point_read_step_error_kind &step_error:
                                                GuiSfntSimpleGlyphOutlinePointReadStepErrorKind::PointReadFailed:
                                                    true
                                                _:
                                                    false
                                            let point_error_ok %bool match gui_sfnt_simple_glyph_outline_point_read_step_error_point_error &step_error:
                                                Option::None:
                                                    false
                                                Option::Some point_error:
                                                    match gui_sfnt_simple_glyph_outline_point_read_error_kind &point_error:
                                                        GuiSfntSimpleGlyphOutlinePointReadErrorKind::FlagReadFailed:
                                                            true
                                                        _:
                                                            false
                                            and step_kind_ok point_error_ok
                                    and kind_ok step_ok
                            io_bytebuf_free bytes
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            result
        _:
            false

fn item_drain_budget_states_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 111
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
                            let full_ok %bool item_drain_full_end_ok &bytes glyf stream &storage
                            let partial_ok %bool item_drain_partial_budget_exhausted_ok &bytes glyf stream &storage
                            let zero_nonterminal_ok %bool item_drain_zero_budget_nonterminal_ok &bytes glyf stream &storage
                            let zero_terminal_ok %bool item_drain_zero_budget_terminal_ok &bytes glyf stream &storage
                            let cursor_ok %bool item_drain_cursor_too_far_ok &bytes glyf stream &storage
                            io_bytebuf_free bytes
                            gui_sfnt_simple_glyph_outline_storage_free storage
                            and full_ok and partial_ok and zero_nonterminal_ok and zero_terminal_ok cursor_ok
        _:
            false

fn main %impure fn void i32 \void:
    let budget_ok %bool item_drain_budget_states_ok
    let wrap_ok %bool item_drain_wraps_point_step_read_failure_ok
    test_assertion_exit_code assert "point stream item drain contract" and budget_ok wrap_ok
```
