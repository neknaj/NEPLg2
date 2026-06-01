# core/gui dirty region set

このファイルは、embedded/no_alloc backend 向けに固定容量 2 rect の dirty region set contract を確認します。

## dirty_region_set_keeps_two_rects

[目的/もくてき]:
- 2 つまでの rect を allocator なしで[保持/ほじ]することを確認します。
- x/y の[負/ふ]は[相対/そうたい][座標/ざひょう]として[許容/きょよう]し、slot query で[読/よ]めることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/gui/dirty_region_set" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let regions0 %DirtyRegionSet dirty_regions_empty
    match dirty_regions_push_checked regions0 (gui_rect_new (sub 0 3) 4 5 6):
        Result::Ok regions1:
            match dirty_regions_push_checked regions1 (gui_rect_new 7 (sub 0 8) 9 10):
                Result::Ok regions2:
                    assert dirty_regions_is_two regions2
                    match dirty_regions_rect_at regions2 0:
                        Option::Some first:
                            match dirty_regions_rect_at regions2 1:
                                Option::Some second:
                                    assert_eq_i32 (sub 0 3) gui_rect_x &first
                                    assert_eq_i32 4 gui_rect_y &first
                                    assert_eq_i32 5 gui_rect_width &first
                                    assert_eq_i32 6 gui_rect_height &first
                                    assert_eq_i32 7 gui_rect_x &second
                                    assert_eq_i32 (sub 0 8) gui_rect_y &second
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
```

## dirty_region_set_overflow_becomes_full

[目的/もくてき]:
- 3 つ[目/め]の rect を silent no-op にせず、Full [状態/じょうたい]へ[昇格/しょうかく]することを確認します。
- Full [状態/じょうたい]では個別 rect query が `None` になることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/gui/dirty_region_set" as *
#import "core/gui/geometry" as *
#import "core/option" as *
#import "core/result" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let regions0 %DirtyRegionSet dirty_regions_empty
    match dirty_regions_push_checked regions0 (gui_rect_new 0 0 1 1):
        Result::Ok regions1:
            match dirty_regions_push_checked regions1 (gui_rect_new 1 1 2 2):
                Result::Ok regions2:
                    match dirty_regions_push_checked regions2 (gui_rect_new 2 2 3 3):
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
```

## dirty_region_set_rejects_negative_size

[目的/もくてき]:
- width/height が[負/ふ]の rect を `GuiError::InvalidGeometry` として[拒否/きょひ]することを確認します。
- invalid rect を push しても panic や silent no-op ではなく `Result::Err` で[返/かえ]すことを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/gui/dirty_region_set" as *
#import "core/gui/error" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let regions %DirtyRegionSet dirty_regions_empty
    match dirty_regions_push_checked regions (gui_rect_new 0 0 (sub 0 1) 4):
        Result::Ok _regions1:
            1
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    match dirty_regions_push_checked regions (gui_rect_new 0 0 4 (sub 0 1)):
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
```
