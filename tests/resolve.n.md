# resolve.rs 由来の doctest

このファイルは Rust テスト `resolve.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## parse_prelude_directives

以前はコンパイル確認のみでした。
`#prelude` と `#no_prelude` のディレクティブを含むコードが「実行まで通る」ことを明確にするため、`ret: 0` を追加します。

neplg2:test
ret: 0
```neplg2
#prelude std/prelude_base
#no_prelude
#entry main
fn main <() -> i32> ():
    0
```

## import_clause_merge_is_preserved

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## resolve_import_alias_open_selective

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## build_visible_map_reports_ambiguous_open

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## selective_glob_opens_module

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## package_import_resolves_std

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## resolve_import_default_alias_from_nested_relative

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## resolve_import_default_alias_from_package

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## selective_import_skips_missing_exports

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## merge_import_is_treated_as_open

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## build_visible_map_prefers_local_over_imports

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```

## build_visible_map_prefers_selective_over_open

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main <()->i32>():
    0
```
