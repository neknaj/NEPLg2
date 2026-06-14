# GUI font SFNT glyf outline scalar push doctests

このファイルは、F5c の scalar slot push が storage owner を失わず、push 失敗時にも owner と value と error kind を同時に返すことを検査する。

## outline scalar push preserves owner recovery

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

fn outline_storage_push_success_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 14
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage0:
                    match gui_sfnt_simple_glyph_outline_storage_push_scalar_slot storage0 17:
                        Result::Ok storage1:
                            match gui_sfnt_simple_glyph_outline_storage_push_scalar_slot storage1 23:
                                Result::Ok storage2:
                                    let len_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_scalar_slots_len &storage2
                                    let cap_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slots_cap &storage2
                                    let count_ok %bool eq 22 gui_sfnt_simple_glyph_outline_storage_scalar_slot_count &storage2
                                    gui_sfnt_simple_glyph_outline_storage_free storage2
                                    and len_ok and cap_ok count_ok
                                Result::Err error2:
                                    let recovered2 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_storage_push_error_storage error2
                                    gui_sfnt_simple_glyph_outline_storage_free recovered2
                                    false
                        Result::Err error1:
                            let recovered1 %GuiSfntSimpleGlyphOutlineStorage gui_sfnt_simple_glyph_outline_storage_push_error_storage error1
                            gui_sfnt_simple_glyph_outline_storage_free recovered1
                            false
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_storage_push_error_recovery_callback %impure fn GuiSfntSimpleGlyphOutlineStorage impure fn i32 impure fn StdErrorKind bool \storage\value\kind:
    let value_ok %bool eq value 77
    let kind_ok %bool match kind:
        StdErrorKind::CapacityExceeded:
            true
        _:
            false
    gui_sfnt_simple_glyph_outline_storage_free storage
    and value_ok kind_ok

fn outline_storage_push_error_recovery_ok %impure fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 15
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_alloc &capacity &limit:
                Result::Ok storage:
                    let error %GuiSfntSimpleGlyphOutlineStoragePushError gui_sfnt_simple_glyph_outline_storage_push_error storage 77 StdErrorKind::CapacityExceeded
                    let value_ok %bool eq 77 gui_sfnt_simple_glyph_outline_storage_push_error_scalar_value &error
                    let kind_ok %bool match gui_sfnt_simple_glyph_outline_storage_push_error_kind &error:
                        StdErrorKind::CapacityExceeded:
                            true
                        _:
                            false
                    let recovered_ok %bool gui_sfnt_simple_glyph_outline_storage_push_error_with error @outline_storage_push_error_recovery_callback
                    and value_ok and kind_ok recovered_ok
                Result::Err _error:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn main %impure fn void i32 \void:
    let push_ok %bool outline_storage_push_success_ok
    let push_recovery_ok %bool outline_storage_push_error_recovery_ok
    test_assertion_exit_code assert "outline scalar push contract" and push_ok push_recovery_ok
```
