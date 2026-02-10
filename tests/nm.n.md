# nm parser/html 連携テスト

## nm_parse_markdown_json_basic

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target wasm
#import "alloc/string" as *
#import "core/math" as *
#import "nm/parser" as *

fn main <()->i32> ():
    let doc <Document> parse_markdown "# A\n\nhello\n";
    let j <str> document_to_json doc;
    if:
        and str_starts_with j "{\"t\":\"doc\"" gt len j 0
        then 0
        else 1
```

## nm_render_document_basic

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target wasm
#import "alloc/string" as *
#import "core/math" as *
#import "nm/parser" as *
#import "nm/html_gen" as *

fn main <()->i32> ():
    let doc <Document> parse_markdown "# A\n\nhello\n";
    let h <str> render_document doc;
    if:
        and str_starts_with h "<section" str_ends_with h "</section>"
        then 0
        else 1
```
