# compile_fail diagnostic location

## compile_fail_matches_diag_id_and_line_col

neplg2:test[compile_fail]
diag_id: 3001
diag_span: 3:5
```neplg2
#entry main
fn main <()->i32> ():
    missing_name
```

## compile_fail_matches_multiple_diag_spans

neplg2:test[compile_fail]
diag_id: 3001
diag_spans: ["3:5", "/virtual/entry.nepl:3:5"]
```neplg2
#entry main
fn main <()->i32> ():
    missing_name
```

## compile_fail_matches_object_style_diag_spans

neplg2:test[compile_fail]
diag_id: 3001
diag_spans: [{"line": 3, "col": 5}, {"file": "/virtual/entry.nepl", "line": 3, "col": 5}]
```neplg2
#entry main
fn main <()->i32> ():
    missing_name
```

## entry_missing_uses_entry_directive_span

[目的/もくてき]:
- `#entry` が[存在/そんざい]しない[関数/かんすう]を[指/さ]すとき、dummy span ではなく `#entry` の[名前/なまえ][位置/いち]に[診断/しんだん]が[付/つ]くことを[確認/かくにん]します。

[確/たし]かめること:
- `TypeEntryFunctionMissingOrAmbiguous` が `diag_id: 3092` で[返/かえ]ること。
- [位置/いち]が `main` の[識別子/しきべつし]を[指/さ]すこと。

neplg2:test[compile_fail]
diag_id: 3092
diag_span: 2:8
```neplg2
#target llvm
#entry main
fn boot <()->i32> ():
    0
```
