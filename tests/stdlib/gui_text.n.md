# alloc/gui text

このファイルは `alloc/gui/text` が platform API に依存せず、`Result` による境界検査で text storage と text layout cache data を扱えることを固定します。

## text_buffer_insert_replace_delete_store_string

[目的/もくてき]:
- insert / replace / delete が `TextBufferId` を保ったまま新しい本文を返すことを確認します。
- 本文 storage は `str` として扱い、測定 API に依存しないことを固定します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/text" as *
#import "alloc/string/search" as *
#import "core/math" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let initial %TextBuffer text_buffer_new (text_buffer_id_new 42) "ac"
    let inserted %TextBuffer unwrap_ok text_buffer_insert initial 1 "b"
    let replaced %TextBuffer unwrap_ok text_buffer_replace inserted 1 2 "B"
    let deleted %TextBuffer unwrap_ok text_buffer_delete replaced 2 3
    let actual_text %str text_buffer_text &deleted
    let actual_buffer_id %TextBufferId text_buffer_id &deleted
    let actual_id %i32 text_buffer_id_raw &actual_buffer_id
    let actual_len %i32 text_buffer_len &deleted
    if not str_eq actual_text "aB":
        then 1
        else if not eq actual_id 42:
            then 2
            else if not eq actual_len 2:
                then 3
                else 0
```

## text_buffer_rejects_out_of_range

[目的/もくてき]:
- 範囲外 insert が panic や silent no-op ではなく `GuiError::InvalidCommand` になることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/text" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let buffer %TextBuffer text_buffer_new (text_buffer_id_new 1) "abc"
    match text_buffer_insert buffer 4 "z":
        Result::Ok _next:
            1
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    0
                _:
                    2
```

## text_buffer_rejects_utf8_middle_byte

[目的/もくてき]:
- UTF-8 文字の途中 byte を編集境界にした場合、文字列を壊さず `GuiError::InvalidCommand` を返すことを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/text" as *
#import "core/result" as *

fn main %fn unit i32 \unit:
    let buffer %TextBuffer text_buffer_new (text_buffer_id_new 1) "aあ"
    match text_buffer_delete buffer 2 3:
        Result::Ok _next:
            1
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    0
                _:
                    2
```

## text_layout_measure_uses_char_count_and_cache_key

[目的/もくてき]:
- `TextLayout` が buffer / run / font / max_width と測定結果を platform 非依存 data として保持することを確認します。
- fallback cell count hint が byte length ではなく `str_char_count` に基づくことを、UTF-8 を含む文字列で固定します。
- `CachedTextLayout` の key が buffer id / run id / font id / max_width / byte length / char count から決まることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/text" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let buffer %TextBuffer text_buffer_new (text_buffer_id_new 7) "aあ"
    let measurer %MockTextMeasurer mock_text_measurer_new 5 11 8
    let layout %TextLayout unwrap_ok text_layout_from_buffer &measurer &buffer (text_run_id_new 9) (font_id_new 3) 80
    let cached %CachedTextLayout cached_text_layout_from_layout layout
    let key %TextLayoutCacheKey cached_text_layout_key &cached
    let layout_buffer_id %TextBufferId text_layout_buffer_id &layout
    let layout_run_id %TextRunId text_layout_run_id &layout
    let layout_font_id %FontId text_layout_font_id &layout
    let key_buffer_id %TextBufferId text_layout_cache_key_buffer_id &key
    let key_run_id %TextRunId text_layout_cache_key_run_id &key
    let key_font_id %FontId text_layout_cache_key_font_id &key
    if not eq 7 text_buffer_id_raw &layout_buffer_id:
        then 1
        else if not eq 9 text_run_id_raw &layout_run_id:
            then 2
            else if not eq 3 font_id_raw &layout_font_id:
                then 3
                else if not eq 2 text_layout_char_count &layout:
                    then 4
                    else if not eq 4 text_layout_byte_len &layout:
                        then 5
                        else if not eq 2 text_layout_cell_count &layout:
                            then 6
                            else if not eq 10 text_layout_width &layout:
                                then 7
                                else if not eq 11 text_layout_height &layout:
                                    then 8
                                    else if not eq 8 text_layout_baseline &layout:
                                        then 9
                                        else if not eq 80 text_layout_max_width &layout:
                                            then 10
                                            else if not eq 7 text_buffer_id_raw &key_buffer_id:
                                                then 11
                                                else if not eq 9 text_run_id_raw &key_run_id:
                                                    then 12
                                                    else if not eq 3 font_id_raw &key_font_id:
                                                        then 13
                                                        else if not eq 80 text_layout_cache_key_max_width &key:
                                                            then 14
                                                            else if not eq 4 text_layout_cache_key_byte_len &key:
                                                                then 15
                                                                else if not eq 2 text_layout_cache_key_char_count &key:
                                                                    then 16
                                                                    else 0
```

## text_layout_rejects_invalid_max_width

[目的/もくてき]:
- 負の max_width が panic や sentinel value ではなく `GuiError::InvalidGeometry` になることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/text" as *
#import "core/gui" as *
#import "core/math" as *
#import "core/result" as *

fn main %impure fn unit i32 \unit:
    let buffer %TextBuffer text_buffer_new (text_buffer_id_new 1) "abc"
    let measurer %MockTextMeasurer mock_text_measurer_new 5 11 8
    match text_layout_from_buffer &measurer &buffer (text_run_id_new 1) (font_id_new 1) (sub 0 1):
        Result::Ok _layout:
            1
        Result::Err error:
            match error:
                GuiError::InvalidGeometry:
                    0
                _:
                    2
```

## text_layout_propagates_measurer_error

[目的/もくてき]:
- injected `TextMeasurer` が返した error を、text layout 測定 helper が別の error に置き換えないことを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui/text" as *
#import "core/gui" as *
#import "core/result" as *

struct RejectingTextMeasurer:
    marker %i32

impl TextMeasurer for RejectingTextMeasurer:
    fn measure_text %fn &RejectingTextMeasurer fn TextMeasureRequest Result TextMeasureResult GuiError \measurer\request:
        Result::Err GuiError::Unsupported

fn main %impure fn unit i32 \unit:
    let buffer %TextBuffer text_buffer_new (text_buffer_id_new 1) "abc"
    let measurer %RejectingTextMeasurer RejectingTextMeasurer 0
    match text_layout_from_buffer &measurer &buffer (text_run_id_new 1) (font_id_new 1) 40:
        Result::Ok _layout:
            1
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    0
                _:
                    2
```
