# core/gui dirty region

このファイルは embedded/no_alloc dirty region contract が allocator、std、platform API に依存せず、`GuiRect` と `GuiError` だけで扱えることを固定します。

## dirty_region_empty_rect_and_full_merge

[目的/もくてき]:
- Empty と Rect の merge は Rect を[返/かえ]すことを確認します。
- Full が merge に[含/ふく]まれる場合は Full を[返/かえ]すことを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_dirty_region_empty_rect_and_full_merge\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/test" as *
#import "std/test" as test

fn run_case %fn void i32 \void:
    let rect %GuiRect gui_rect_new 4 5 6 7
    let region %DirtyRegion dirty_region_rect_unchecked rect
    let empty %DirtyRegion dirty_region_empty
    let merged %DirtyRegion dirty_region_merge empty region
    match merged:
        DirtyRegion::Rect out:
            assert_eq_i32 4 gui_rect_x &out
            assert_eq_i32 5 gui_rect_y &out
            assert_eq_i32 6 gui_rect_width &out
            assert_eq_i32 7 gui_rect_height &out
            let full %DirtyRegion dirty_region_full
            let full_merged %DirtyRegion dirty_region_merge merged full
            assert dirty_region_is_full full_merged
            0
        _:
            1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_dirty_region_empty_rect_and_full_merge"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## dirty_region_rect_merge_uses_bounding_rect

[目的/もくてき]:
- 2 つの Rect を list 化せず、bounding rect へ O(1) で[畳/たた]む contract を固定します。
- 負の x/y は相対座標として[許容/きょよう]し、width/height から right/bottom を[計算/けいさん]することを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_dirty_region_rect_merge_uses_bounding_rect\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/test" as *
#import "std/test" as test

fn run_case %fn void i32 \void:
    let a_rect %GuiRect gui_rect_new 10 20 5 8
    let negative_x %i32 sub 0 2
    let b_rect %GuiRect gui_rect_new negative_x 18 4 5
    let a %DirtyRegion dirty_region_rect_unchecked a_rect
    let b %DirtyRegion dirty_region_rect_unchecked b_rect
    match dirty_region_merge a b:
        DirtyRegion::Rect out:
            assert_eq_i32 negative_x gui_rect_x &out
            assert_eq_i32 18 gui_rect_y &out
            assert_eq_i32 17 gui_rect_width &out
            assert_eq_i32 10 gui_rect_height &out
            0
        _:
            1

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_dirty_region_rect_merge_uses_bounding_rect"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## dirty_region_checked_rejects_negative_size

[目的/もくてき]:
- width/height が[負/ふ]の rect を `GuiError::InvalidGeometry` として[拒否/きょひ]することを確認します。
- x/y が[負/ふ]でも size が[非負/ひふ]なら relative coordinate として[受/う]け[入/い]れることを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_dirty_region_dirty_region_checked_rejects_negative_size\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

fn run_case %fn void i32 \void:
    let negative_width %i32 sub 0 1
    let invalid_rect %GuiRect gui_rect_new 0 0 negative_width 4
    match dirty_region_rect_checked invalid_rect:
        Result::Ok _region:
            1
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    let negative_x %i32 sub 0 10
                    let negative_y %i32 sub 0 20
                    let valid_rect %GuiRect gui_rect_new negative_x negative_y 3 4
                    match dirty_region_rect_checked valid_rect:
                        Result::Ok _valid:
                            0
                        Result::Err _valid_error:
                            2
                _:
                    3

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_dirty_region_dirty_region_checked_rejects_negative_size"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
