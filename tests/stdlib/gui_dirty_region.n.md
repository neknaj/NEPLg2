# core/gui dirty region

このファイルは embedded/no_alloc dirty region contract が allocator、std、platform API に依存せず、`GuiRect` と `GuiError` だけで扱えることを固定します。

## dirty_region_empty_rect_and_full_merge

[目的/もくてき]:
- Empty と Rect の merge は Rect を[返/かえ]すことを確認します。
- Full が merge に[含/ふく]まれる場合は Full を[返/かえ]すことを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/gui/dirty_region" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let rect %GuiRect gui_rect_new 4 5 6 7
    let region %DirtyRegion dirty_region_rect_unchecked rect
    let merged %DirtyRegion dirty_region_merge (dirty_region_empty unit) region
    match merged:
        DirtyRegion::Rect out:
            assert_eq_i32 4 gui_rect_x &out
            assert_eq_i32 5 gui_rect_y &out
            assert_eq_i32 6 gui_rect_width &out
            assert_eq_i32 7 gui_rect_height &out
            assert dirty_region_is_full (dirty_region_merge merged (dirty_region_full unit))
            0
        _:
            1
```

## dirty_region_rect_merge_uses_bounding_rect

[目的/もくてき]:
- 2 つの Rect を list 化せず、bounding rect へ O(1) で[畳/たた]む contract を固定します。
- 負の x/y は相対座標として[許容/きょよう]し、width/height から right/bottom を[計算/けいさん]することを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/gui/dirty_region" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let a %DirtyRegion dirty_region_rect_unchecked (gui_rect_new 10 20 5 8)
    let b %DirtyRegion dirty_region_rect_unchecked (gui_rect_new (sub 0 2) 18 4 5)
    match dirty_region_merge a b:
        DirtyRegion::Rect out:
            assert_eq_i32 (sub 0 2) gui_rect_x &out
            assert_eq_i32 18 gui_rect_y &out
            assert_eq_i32 17 gui_rect_width &out
            assert_eq_i32 10 gui_rect_height &out
            0
        _:
            1
```

## dirty_region_checked_rejects_negative_size

[目的/もくてき]:
- width/height が[負/ふ]の rect を `GuiError::InvalidGeometry` として[拒否/きょひ]することを確認します。
- x/y が[負/ふ]でも size が[非負/ひふ]なら relative coordinate として[受/う]け[入/い]れることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core

#import "core/gui/dirty_region" as *
#import "core/gui/error" as *
#import "core/gui/geometry" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    match dirty_region_rect_checked (gui_rect_new 0 0 (sub 0 1) 4):
        Result::Ok _region:
            1
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    match dirty_region_rect_checked (gui_rect_new (sub 0 10) (sub 0 20) 3 4):
                        Result::Ok _valid:
                            0
                        Result::Err _valid_error:
                            2
                _:
                    3
```
