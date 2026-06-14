# GUI font SFNT glyf outline storage doctests

このファイルは、F5b の simple glyph outline scalar storage owner が typed capacity / limit だけを使い、byte fixture、renderer、rasterizer、platform API に依存しないことを検査する。

## outline storage validates owner allocation boundaries

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

fn make_bounds %fn GuiGlyphId GuiSfntGlyphBounds \glyph:
    gui_sfnt_glyph_bounds glyph 0 0 10 12

fn make_topology %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphTopology \glyph\contours\points:
    let bounds %GuiSfntGlyphBounds make_bounds glyph
    gui_sfnt_simple_glyph_topology glyph bounds contours points 0 0 0

fn outline_storage_error_kind_is %fn &GuiSfntSimpleGlyphOutlineStorageAllocError fn GuiSfntSimpleGlyphOutlineStorageAllocErrorKind bool \error\expected:
    let observed %GuiSfntSimpleGlyphOutlineStorageAllocErrorKind gui_sfnt_simple_glyph_outline_storage_alloc_error_kind error
    match observed:
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::InvalidCapacity:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::InvalidCapacity:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotCountOverflow:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotCountOverflow:
                    true
                _:
                    false
        GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotStorageAllocFailed:
            match expected:
                GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotStorageAllocFailed:
                    true
                _:
                    false

fn outline_storage_error_has_capacity_check %fn &GuiSfntSimpleGlyphOutlineStorageAllocError bool \error:
    match gui_sfnt_simple_glyph_outline_storage_alloc_error_capacity_check error:
        Option::Some _check:
            true
        Option::None:
            false

fn outline_storage_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 10
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    let count_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slot_count &storage
                    let len_ok %bool eq 0 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage
                    let cap_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slots_cap &storage
                    gui_sfnt_simple_glyph_outline_storage_free storage
                    and count_ok and len_ok cap_ok
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_storage_invalid_capacity_precedes_limit_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 11
    let bad_capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_storage_capacity glyph 2 4 5 4 8
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 4 5 8
    match gui_sfnt_simple_glyph_outline_storage_alloc &bad_capacity &limit:
        Result::Ok storage:
            gui_sfnt_simple_glyph_outline_storage_free storage
            false
        Result::Err error:
            let kind_ok %bool outline_storage_error_kind_is &error GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::InvalidCapacity
            let no_check_ok %bool not outline_storage_error_has_capacity_check &error
            and kind_ok no_check_ok

fn outline_storage_limit_reject_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 12
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    gui_sfnt_simple_glyph_outline_storage_free storage
                    false
                Result::Err error:
                    let kind_ok %bool outline_storage_error_kind_is &error GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::CapacityRejected
                    let check_ok %bool outline_storage_error_has_capacity_check &error
                    and kind_ok check_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_storage_scalar_overflow_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 13
    let capacity %GuiSfntSimpleGlyphOutlineStorageCapacity gui_sfnt_simple_glyph_outline_storage_capacity glyph 1 1073741823 1073741823 1073741823 2147483646
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 1073741823 1073741823 2147483647
    match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
        Result::Ok storage:
            gui_sfnt_simple_glyph_outline_storage_free storage
            false
        Result::Err error:
            let kind_ok %bool outline_storage_error_kind_is &error GuiSfntSimpleGlyphOutlineStorageAllocErrorKind::ScalarSlotCountOverflow
            let check_ok %bool outline_storage_error_has_capacity_check &error
            and kind_ok check_ok

fn main %impure fn void i32 \void:
    let success_ok %bool outline_storage_success_ok
    let invalid_ok %bool outline_storage_invalid_capacity_precedes_limit_ok
    let reject_ok %bool outline_storage_limit_reject_ok
    let overflow_ok %bool outline_storage_scalar_overflow_ok
    test_assertion_exit_code assert "outline storage owner contract" and success_ok and invalid_ok and reject_ok overflow_ok
```
