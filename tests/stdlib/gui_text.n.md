# alloc/gui text

このファイルは `alloc/gui/text` が platform API や `TextMeasurer` に依存せず、`Result` による境界検査で text storage を編集できることを固定します。

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
