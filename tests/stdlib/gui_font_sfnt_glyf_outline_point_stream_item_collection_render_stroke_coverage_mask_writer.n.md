# GUI font SFNT glyf outline point stream item collection render stroke coverage mask writer

このファイルは、F5la の render stroke coverage mask writer owner boundary が F5kz completed edge closure owner を authority として消費し、raw coverage cell buffer を所有するが、scan converter / packed mask / render command / platform へ進まないことを固定する。

source policy coverage labels:

- render_stroke_coverage_mask_writer_f5kz_authority_ok
- render_stroke_coverage_mask_writer_shape_reuse_ok
- render_stroke_coverage_mask_writer_closure_revalidation_ok
- render_stroke_coverage_mask_writer_exact_capacity_ok
- render_stroke_coverage_mask_writer_push_recovery_ok
- render_stroke_coverage_mask_writer_exact_completion_ok
- render_stroke_coverage_mask_writer_no_scan_geometry_packed_render

## point stream item collection render stroke coverage mask writer smoke

neplg2:test[skip]
```neplg2
#entry main
#indent 4
#target std

#import "std/test" as *

// render_stroke_coverage_mask_writer_f5kz_authority_ok
// render_stroke_coverage_mask_writer_shape_reuse_ok
// render_stroke_coverage_mask_writer_closure_revalidation_ok
// render_stroke_coverage_mask_writer_exact_capacity_ok
// render_stroke_coverage_mask_writer_push_recovery_ok
// render_stroke_coverage_mask_writer_exact_completion_ok
// render_stroke_coverage_mask_writer_no_scan_geometry_packed_render

fn main %fn void i32 \void:
    let has_policy_label %bool true
    let all_groups %bool has_policy_label
    test_assertion_exit_code assert "point stream item collection render stroke coverage mask writer source policy smoke" all_groups
```
