# GUI font SFNT glyf outline capacity doctests

このファイルは、F5a の simple glyph outline storage capacity が byte fixture、renderer、rasterizer、platform API を使わず、typed value だけで容量計画を返すことを検査する。

## outline capacity validates topology and limits

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

fn make_bounds %fn GuiGlyphId GuiSfntGlyphBounds \glyph:
    gui_sfnt_glyph_bounds glyph 0 0 10 12

fn make_topology %fn GuiGlyphId fn i32 fn i32 GuiSfntSimpleGlyphTopology \glyph\contours\points:
    let bounds %GuiSfntGlyphBounds make_bounds glyph
    gui_sfnt_simple_glyph_topology glyph bounds contours points 0 0 0

fn outline_capacity_valid_topology_ok %fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 1
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            let contours_ok %bool eq 2 gui_sfnt_simple_glyph_outline_storage_capacity_contour_count &capacity
            let points_ok %bool eq 4 gui_sfnt_simple_glyph_outline_storage_capacity_point_count &capacity
            let edges_ok %bool eq 4 gui_sfnt_simple_glyph_outline_storage_capacity_edge_count &capacity
            let pairs_ok %bool eq 4 gui_sfnt_simple_glyph_outline_storage_capacity_path_command_pair_count &capacity
            let commands_ok %bool eq 8 gui_sfnt_simple_glyph_outline_storage_capacity_path_command_count &capacity
            and contours_ok and points_ok and edges_ok and pairs_ok commands_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_capacity_invalid_topology_ok %fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 2
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 5 4
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits _capacity:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology invalid:
            eq 5 gui_sfnt_simple_glyph_topology_contour_count &invalid
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_capacity_command_overflow_ok %fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 3
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 1 1073741824
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits _capacity:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow overflow:
            eq 1073741824 gui_sfnt_simple_glyph_topology_point_count &overflow
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_capacity_reject_reason_is %fn GuiSfntSimpleGlyphOutlineCapacityCheck fn GuiSfntSimpleGlyphOutlineCapacityRejectReason bool \check\expected:
    match check:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits _capacity:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected rejected:
            let reason %GuiSfntSimpleGlyphOutlineCapacityRejectReason gui_sfnt_simple_glyph_outline_capacity_rejected_reason &rejected
            match reason:
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded:
                    match expected:
                        GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::PointCapacityExceeded:
                    match expected:
                        GuiSfntSimpleGlyphOutlineCapacityRejectReason::PointCapacityExceeded:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::EdgeCapacityExceeded:
                    match expected:
                        GuiSfntSimpleGlyphOutlineCapacityRejectReason::EdgeCapacityExceeded:
                            true
                        _:
                            false
                GuiSfntSimpleGlyphOutlineCapacityRejectReason::CommandCapacityExceeded:
                    match expected:
                        GuiSfntSimpleGlyphOutlineCapacityRejectReason::CommandCapacityExceeded:
                            true
                        _:
                            false

fn outline_capacity_limit_reject_ok %fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 4
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            let contour_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 1 4 4 8
            let point_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 3 4 8
            let edge_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 3 8
            let command_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 7
            let zero_limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 0 4 4 8
            let contour_ok %bool outline_capacity_reject_reason_is gui_sfnt_simple_glyph_outline_storage_capacity_check_limit &capacity &contour_limit GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded
            let point_ok %bool outline_capacity_reject_reason_is gui_sfnt_simple_glyph_outline_storage_capacity_check_limit &capacity &point_limit GuiSfntSimpleGlyphOutlineCapacityRejectReason::PointCapacityExceeded
            let edge_ok %bool outline_capacity_reject_reason_is gui_sfnt_simple_glyph_outline_storage_capacity_check_limit &capacity &edge_limit GuiSfntSimpleGlyphOutlineCapacityRejectReason::EdgeCapacityExceeded
            let command_ok %bool outline_capacity_reject_reason_is gui_sfnt_simple_glyph_outline_storage_capacity_check_limit &capacity &command_limit GuiSfntSimpleGlyphOutlineCapacityRejectReason::CommandCapacityExceeded
            let zero_ok %bool outline_capacity_reject_reason_is gui_sfnt_simple_glyph_outline_storage_capacity_check_limit &capacity &zero_limit GuiSfntSimpleGlyphOutlineCapacityRejectReason::ContourCapacityExceeded
            and contour_ok and point_ok and edge_ok and command_ok zero_ok
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn outline_capacity_fit_limit_ok %fn void bool \void:
    let glyph %GuiGlyphId unwrap_ok gui_glyph_id_result 5
    let topology %GuiSfntSimpleGlyphTopology make_topology glyph 2 4
    let limit %GuiSfntSimpleGlyphOutlineStorageLimit gui_sfnt_simple_glyph_outline_storage_limit 2 4 4 8
    match gui_sfnt_simple_glyph_outline_storage_capacity_from_topology &topology:
        GuiSfntSimpleGlyphOutlineCapacityCheck::Fits capacity:
            match gui_sfnt_simple_glyph_outline_storage_capacity_check_limit &capacity &limit:
                GuiSfntSimpleGlyphOutlineCapacityCheck::Fits checked:
                    eq 8 gui_sfnt_simple_glyph_outline_storage_capacity_path_command_count &checked
                GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
                    false
                GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
                    false
        GuiSfntSimpleGlyphOutlineCapacityCheck::InvalidTopology _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::CommandCountOverflow _topology:
            false
        GuiSfntSimpleGlyphOutlineCapacityCheck::Rejected _rejected:
            false

fn main %impure fn void i32 \void:
    let valid_ok %bool outline_capacity_valid_topology_ok
    let invalid_ok %bool outline_capacity_invalid_topology_ok
    let overflow_ok %bool outline_capacity_command_overflow_ok
    let reject_ok %bool outline_capacity_limit_reject_ok
    let fit_ok %bool outline_capacity_fit_limit_ok
    test_assertion_exit_code assert "outline capacity value contract" and valid_ok and invalid_ok and overflow_ok and reject_ok fit_ok
```
