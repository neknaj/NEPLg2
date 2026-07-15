# GUI font registered simple glyph indexed stroke coverage cell

このファイルは、test-only helperが返すinvariant-validなactual F5nxp completed ownerをproduction F5nxq writerへ渡し、回収可能な失敗とexact completionを検査する。

## invariant-valid join geometry owner reaches exact coverage completion

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"registered stroke coverage writer preserves owner recovery\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" as *
#import "alloc/gui/font/sfnt/glyf" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn coverage_finish %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellWriterOwner impure fn bool bool \owner\prefix_ok:
    match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_push owner 1:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_free error
            false
        Result::Ok full:
            match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_complete full:
                Result::Err error:
                    gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_completion_error_free error
                    false
                Result::Ok completed:
                    let exact_ok %bool and eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_owner_cell_count &completed and eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_owner_cells_len &completed eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_owner_cells_cap &completed
                    gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_owner_free completed
                    and prefix_ok exact_ok

fn coverage_write %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellWriterOwner impure fn bool bool \owner0\start_ok:
    let negative %i32 sub 0 1
    match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_push owner0 negative:
        Result::Ok unexpected:
            gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_free unexpected
            false
        Result::Err negative_error:
            let negative_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_kind &negative_error:
                GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellPushErrorKind::CoverageNegative: eq negative gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_value &negative_error
                _: false
            let owner1 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_owner negative_error
            match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_push owner1 5:
                Result::Ok unexpected:
                    gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_free unexpected
                    false
                Result::Err over_error:
                    let over_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_kind &over_error:
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellPushErrorKind::CoverageExceedsMax: eq 5 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_value &over_error
                        _: false
                    let owner2 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_owner over_error
                    let forced gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_test_force_push_failure owner2 1
                    let forced_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_kind &forced:
                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellPushErrorKind::StorageFailed kind:
                            match kind:
                                StdErrorKind::CapacityExceeded: true
                                _: false
                        _: false
                    let owner3 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_owner forced
                    match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_push owner3 1:
                        Result::Err error:
                            gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_push_error_free error
                            false
                        Result::Ok partial:
                            match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_complete partial:
                                Result::Ok unexpected:
                                    gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_owner_free unexpected
                                    false
                                Result::Err incomplete:
                                    let incomplete_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_completion_error_kind &incomplete:
                                        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellCompletionErrorKind::MaskIncomplete: true
                                        _: false
                                    let recovered gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_completion_error_owner incomplete
                                    let prefix_ok %bool and start_ok and negative_ok and over_ok and forced_ok incomplete_ok
                                    coverage_finish recovered prefix_ok

fn coverage_start %impure fn GuiFontRegisteredFaceSimpleGlyphIndexedStrokeJoinGeometryCompletedOwner bool \completed:
    let config %GuiSfntSimpleGlyphRasterCoverageConfig gui_sfnt_simple_glyph_raster_coverage_config 0 0 2 1 2 2
    let forced gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_test_force_start_storage_failure completed config
    let forced_ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_start_error_kind &forced:
        GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCoverageCellStartErrorKind::StorageFailed kind:
            match kind:
                StdErrorKind::CapacityExceeded: true
                _: false
        _: false
    let recovered_config %GuiSfntSimpleGlyphRasterCoverageConfig gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_start_error_config &forced
    let recovered gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_start_error_owner forced
    match gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_start recovered recovered_config:
        Result::Err error:
            gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_start_error_free error
            false
        Result::Ok writer:
            let initial_ok %bool and eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_written_count &writer and eq 0 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_cells_len &writer eq 2 gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell_writer_cells_cap &writer
            let start_ok %bool and forced_ok initial_ok
            coverage_write writer start_ok

fn main %impure fn void i32 \void:
    let ok %bool match gui_font_registered_face_simple_glyph_indexed_stroke_join_geometry_test_completed_owner unit:
        Option::None: false
        Option::Some completed: coverage_start completed
    let report %TestReport test_report_new "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_cell"
    let checked %TestReport test_report_push report assert "registered stroke coverage writer preserves owner recovery" ok
    let shown test_report_print_stdout checked
    test_report_exit_code shown
```
