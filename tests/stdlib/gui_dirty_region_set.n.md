# core/gui dirty region set

このファイルは、embedded/no_alloc backend 向けに固定容量 2 rect の dirty region set contract を確認します。

source policy coverage labels:

- dirty_region_set_push_region_empty_explicit_no_dirty_ok
- dirty_region_set_push_region_rect_checked_ok
- dirty_region_set_push_region_full_escalates_ok
- dirty_region_set_push_region_invalid_unchecked_rect_rejected_ok
- dirty_region_set_push_region_no_alloc_no_platform_no_fallback_ok

## dirty_region_set_keeps_two_rects

[目的/もくてき]:
- 2 つまでの rect を allocator なしで[保持/ほじ]することを確認します。
- x/y の[負/ふ]は[相対/そうたい][座標/ざひょう]として[許容/きょよう]し、slot query で[読/よ]めることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_set_dirty_region_set_keeps_two_rects\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region_set" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/test" as *
#import "std/test" as test

fn run_case %fn void i32 \void:
    let regions0 %DirtyRegionSet dirty_regions_empty
    let first_x %i32 sub 0 3
    let second_y %i32 sub 0 8
    let first_rect %GuiRect gui_rect_new first_x 4 5 6
    let second_rect %GuiRect gui_rect_new 7 second_y 9 10
    match dirty_regions_push_checked regions0 first_rect:
        Result::Ok regions1:
            match dirty_regions_push_checked regions1 second_rect:
                Result::Ok regions2:
                    assert dirty_regions_is_two regions2
                    match dirty_regions_rect_at regions2 0:
                        Option::Some first:
                            match dirty_regions_rect_at regions2 1:
                                Option::Some second:
                                    assert_eq_i32 first_x gui_rect_x &first
                                    assert_eq_i32 4 gui_rect_y &first
                                    assert_eq_i32 5 gui_rect_width &first
                                    assert_eq_i32 6 gui_rect_height &first
                                    assert_eq_i32 7 gui_rect_x &second
                                    assert_eq_i32 second_y gui_rect_y &second
                                    assert_eq_i32 9 gui_rect_width &second
                                    assert_eq_i32 10 gui_rect_height &second
                                    0
                                Option::None:
                                    3
                        Option::None:
                            2
                Result::Err _error2:
                    4
        Result::Err _error1:
            5

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_set_dirty_region_set_keeps_two_rects"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## dirty_region_set_overflow_becomes_full

[目的/もくてき]:
- 3 つ[目/め]の rect を silent no-op にせず、Full [状態/じょうたい]へ[昇格/しょうかく]することを確認します。
- Full [状態/じょうたい]では個別 rect query が `None` になることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_set_dirty_region_set_overflow_becomes_full\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region_set" as *
#import "core/gui/geometry" as *
#import "core/option" as *
#import "core/result" as *
#import "core/test" as *
#import "std/test" as test

fn run_case %fn void i32 \void:
    let regions0 %DirtyRegionSet dirty_regions_empty
    let rect0 %GuiRect gui_rect_new 0 0 1 1
    let rect1 %GuiRect gui_rect_new 1 1 2 2
    let rect2 %GuiRect gui_rect_new 2 2 3 3
    match dirty_regions_push_checked regions0 rect0:
        Result::Ok regions1:
            match dirty_regions_push_checked regions1 rect1:
                Result::Ok regions2:
                    match dirty_regions_push_checked regions2 rect2:
                        Result::Ok regions3:
                            assert dirty_regions_is_full regions3
                            match dirty_regions_rect_at regions3 0:
                                Option::None:
                                    0
                                Option::Some _rect:
                                    1
                        Result::Err _error3:
                            2
                Result::Err _error2:
                    3
        Result::Err _error1:
            4

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_set_dirty_region_set_overflow_becomes_full"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## dirty_region_set_rejects_negative_size

[目的/もくてき]:
- width/height が[負/ふ]の rect を `GuiError::InvalidGeometry` として[拒否/きょひ]することを確認します。
- invalid rect を push しても panic や silent no-op ではなく `Result::Err` で[返/かえ]すことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_set_dirty_region_set_rejects_negative_size\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region_set" as *
#import "core/gui/error" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

fn run_case %fn void i32 \void:
    let regions %DirtyRegionSet dirty_regions_empty
    let negative_width %i32 sub 0 1
    let negative_height %i32 sub 0 1
    let invalid_width_rect %GuiRect gui_rect_new 0 0 negative_width 4
    let invalid_height_rect %GuiRect gui_rect_new 0 0 4 negative_height
    match dirty_regions_push_checked regions invalid_width_rect:
        Result::Ok _regions1:
            1
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    match dirty_regions_push_checked regions invalid_height_rect:
                        Result::Ok _regions2:
                            2
                        Result::Err error2:
                            match error2:
                                GuiError::InvalidGeometry:
                                    0
                                _:
                                    3
                _:
                    4

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_set_dirty_region_set_rejects_negative_size"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## dirty_region_set_push_region_checked

[目的/もくてき]:
- 1 個の `DirtyRegion` を fixed-capacity set に[取/と]り[込/こ]む public helper の contract を確認します。
- Empty は dirty なしとして既存 set を[保/たも]ち、Full は Full set へ[昇格/しょうかく]し、Rect は checked push を[経由/けいゆ]することを確認します。
- unchecked constructor 由来の invalid Rect も `GuiError::InvalidGeometry` として[拒否/きょひ]されることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_set_push_region_checked\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region" as *
#import "core/gui/dirty_region_set" as *
#import "core/gui/error" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/result" as *
#import "core/test" as *
#import "std/test" as test

// dirty_region_set_push_region_empty_explicit_no_dirty_ok
// dirty_region_set_push_region_rect_checked_ok
// dirty_region_set_push_region_full_escalates_ok
// dirty_region_set_push_region_invalid_unchecked_rect_rejected_ok
// dirty_region_set_push_region_no_alloc_no_platform_no_fallback_ok

fn empty_region_case %fn void bool \void:
    let regions0 %DirtyRegionSet dirty_regions_empty
    match dirty_regions_push_region_checked regions0 dirty_region_empty:
        Result::Err _:
            false
        Result::Ok regions1:
            dirty_regions_is_empty regions1

fn rect_region_case %fn void bool \void:
    let regions0 %DirtyRegionSet dirty_regions_empty
    let rect %GuiRect gui_rect_new 1 2 3 4
    match dirty_region_rect_checked rect:
        Result::Err _:
            false
        Result::Ok region:
            match dirty_regions_push_region_checked regions0 region:
                Result::Err _:
                    false
                Result::Ok regions1:
                    dirty_regions_is_one regions1

fn full_region_case %fn void bool \void:
    let regions0 %DirtyRegionSet dirty_regions_empty
    let rect %GuiRect gui_rect_new 1 2 3 4
    match dirty_regions_push_checked regions0 rect:
        Result::Err _:
            false
        Result::Ok regions1:
            match dirty_regions_push_region_checked regions1 dirty_region_full:
                Result::Err _:
                    false
                Result::Ok regions2:
                    dirty_regions_is_full regions2

fn invalid_unchecked_region_case %fn void bool \void:
    let regions0 %DirtyRegionSet dirty_regions_empty
    let negative_width %i32 sub 0 1
    let invalid_rect %GuiRect gui_rect_new 0 0 negative_width 4
    let invalid_region %DirtyRegion dirty_region_rect_unchecked invalid_rect
    match dirty_regions_push_region_checked regions0 invalid_region:
        Result::Ok _:
            false
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    true
                _:
                    false

fn run_case %fn void i32 \void:
    let empty_ok %bool empty_region_case
    let rect_ok %bool rect_region_case
    let full_ok %bool full_region_case
    let invalid_ok %bool invalid_unchecked_region_case
    let first %bool and empty_ok rect_ok
    if and first and full_ok invalid_ok 0 1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_set_push_region_checked"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
