# nm parser/html 連携テスト

## nm_parse_markdown_json_escapes_tab

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "nm/parser" as *

fn main <()->i32> ():
    let doc <Document> parse_markdown "a\tb\n";
    let j <str> document_to_json doc;
    let mut sb <StringBuilder> string_builder_new;
    set sb sb_append sb "{\"t\":\"doc\",\"nodes\":[";
    set sb sb_append sb "{\"t\":\"p\",\"inl\":[";
    set sb sb_append sb "{\"t\":\"text\",\"s\":\"a\\tb\"}";
    set sb sb_append sb "]}]}";
    let expected <str> sb_build sb;
    if:
        str_eq expected j
        then 0
        else 1
```

## nm_render_document_section_exact

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "nm/parser" as *
#import "nm/html_gen" as *

fn main <()->i32> ():
    let doc <Document> parse_markdown "# A\n\nhello\n";
    let h <str> render_document doc;
    let expected <str> "<section class=\"nm-sec level-1\"><h1>A</h1><p>hello</p></section>";
    if:
        str_eq expected h
        then 0
        else 1
```

## nm_render_document_gloss_inline_markup

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "nm/parser" as *
#import "nm/html_gen" as *

fn main <()->i32> ():
    let doc <Document> parse_markdown "[Word/ruby] {Term/gloss/extra}\n";
    let h <str> render_document doc;
    let expected <str> "<p><ruby class=\"nm-ruby\"><rb>Word</rb><rt>ruby</rt></ruby> <ruby class=\"nm-anno\"><rb>Term</rb><rt><span class=\"nm-anno-note\">gloss</span><span class=\"nm-anno-note\">extra</span></rt></ruby></p>";
    if:
        str_eq expected h
        then 0
        else 1
```

## nm_render_document_html_escaping

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "nm/parser" as *
#import "nm/html_gen" as *

fn main <()->i32> ():
    let doc <Document> parse_markdown "&<>'\"\n";
    let h <str> render_document doc;
    let expected <str> "<p>&amp;&lt;&gt;&#39;&quot;</p>";
    if:
        str_eq expected h
        then 0
        else 1
```
