# GUI font registered BeginFrame recovered-state scheduler decision

このfixtureはactual DriverCompletionFailed recovered stateをcaller-supplied decisionでresumeまたはabortへ分類し、resumeだけがslice counterをresetするF5nzx契約を検証する。

neplg2:test[stdio, normalize_newlines]

```neplg2
---
stdout: "test_report name=\"gui_font_registered_begin_frame_recovered_state_scheduler_decision\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"resume recovered state\" expected=\"124\" actual=\"124\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"abort recovered state\" expected=\"60\" actual=\"60\" message=\"\"\n"
---
#entry main
#indent 4
#target std
#import "std/test" as * with tests
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision_test" as * with tests
fn main %impure fn void i32 \void:
    let resume %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision_test_resume_contract unit
    let abort %i32 gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision_test_abort_contract unit
    let report %TestReport test_report_new "gui_font_registered_begin_frame_recovered_state_scheduler_decision"
    let report1 %TestReport test_report_push report (assert_eq_i32 "resume recovered state" 124 resume)
    let report2 %TestReport test_report_push report1 (assert_eq_i32 "abort recovered state" 60 abort)
    test_report_exit_code test_report_print_stdout report2
```
