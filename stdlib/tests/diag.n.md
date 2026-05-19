# stdlib/diag.n.md

`alloc/diag/diag` の[文字列化/もじれつか] helper を[確/たし]かめるためのテストです。
ここでは `Diag` / `Diags` を[人間/にんげん]が[読/よ]める[表示/ひょうじ]へ[整形/せいけい]したとき、
kind / span / note / help / source が[期待/きたい]どおりの[順番/じゅんばん]で[出力/しゅつりょく]されるかを[確認/かくにん]します。

## diag_to_string_formats_structured_fields

[目的/もくてき]:
- `Diag` 1 [件/けん]の[表示/ひょうじ]が kind / message / span / note / help / source を[崩/くず]さず[整形/せいけい]することを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- `kind_str`
- `span_to_string`
- `diag_to_string`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"diag_to_string_formats_structured_fields\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"diag string\" expected=\"error[Failure]: with source\\nat 4:5-6\\ncheck input\\nhelp: doc: std/test\\nsource: parser\\n\" actual=\"error[Failure]: with source\\nat 4:5-6\\ncheck input\\nhelp: doc: std/test\\nsource: parser\\n\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "alloc/diag/diag" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let sp <Span> Span 4 5 6;
    let d0 <Diag> diag_error StdErrorKind::Failure "with source";
    let d1 <Diag> diag_with_span d0 sp;
    let d2 <Diag> diag_add_note d1 "check input";
    let d3 <Diag> diag_add_help d2 "doc: std/test";
    let d4 <Diag> diag_with_source d3 "parser";
    let s <str> diag_to_string d4;
    let report:
        test_report_new "diag_to_string_formats_structured_fields"
        |> test_report_push assert_str_eq "diag string" "error[Failure]: with source\nat 4:5-6\ncheck input\nhelp: doc: std/test\nsource: parser\n" s
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## diags_to_string_keeps_order

[目的/もくてき]:
- `Diags` が[順序/じゅんじょ][付/づ]きの[診断群/しんだんぐん]として[扱/あつか]われることを[確/たし]かめます。
- warn と info を[積/つ]んだとき、[追加/ついか]した[順/じゅん]に[並/なら]ぶことを[確認/かくにん]します。

[何/なに]を[確/たし]かめるか:
- `diags_one`
- `diags_push`
- `diags_to_string`

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"diags_to_string_keeps_order\" count=1 failed=0\nassertion index=0 status=ok kind=str_eq label=\"diags order\" expected=\"warn[warn]: careful\\ninfo[info]: loaded\\n\" actual=\"warn[warn]: careful\\ninfo[info]: loaded\\n\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "alloc/diag/error" as *
#import "alloc/diag/diag" as *
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let ds0 <Diags> diags_one diag_warn "careful";
    let ds1 <Diags> diags_push ds0 diag_info "loaded";
    let s <str> diags_to_string ds1;
    let report:
        test_report_new "diags_to_string_keeps_order"
        |> test_report_push assert_str_eq "diags order" "warn[warn]: careful\ninfo[info]: loaded\n" s
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
