---
id: ISS-20260512T070310096Z-VEC-SOURCE-POLICY-REJECTS-RUSTFMT-RA-74A5475B
title: "Vec source policy rejects rustfmt raw boundary path arrays"
area: test
status: verified
resolved: true
priority: P2
type: test
created: 2026-05-12
updated: 2026-05-12
target: "nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nepl-core/src/loader.rs"
---

# ISS-20260512T070310096Z-VEC-SOURCE-POLICY-REJECTS-RUSTFMT-RA-74A5475B: Vec source policy rejects rustfmt raw boundary path arrays

## 概要

Vec の raw-memory boundary source policy が、`nepl-core/src/loader.rs` の Rust 配列を 1 行表記の正規表現だけで検査している。
rustfmt により `RAW_MEMORY_BOUNDARY_STDLIB_PATHS` のパス配列が複数行へ整形されると、実際には whitelist が存在していても policy が欠落として警告する。

## 対象

- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js, nepl-core/src/loader.rs`

## 根拠

- `node nodesrc/run_source_policy_regressions.js --warn-only` で `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` が `loader raw-memory boundary must include exact Vec implementation submodule stdlib paths` を警告した。
- `nepl-core/src/loader.rs` には `["alloc", "collections", "vec", "sort", "merge", "buffer.nepl"]` 相当の whitelist は存在するが、rustfmt により `&[` から `]` までが複数行に分割されていた。
- 既存検査は `/&\["alloc",\s*"collections", ...\]/` 形式の正規表現で、`&[` の直後に最初の文字列があることと、最後の要素の直後に trailing comma が無いことを暗黙に要求していた。

## 問題

source policy が semantic な stdlib path whitelist ではなく、rustfmt 後のソースコード表記に依存している。
このため raw-memory boundary が正しく制限されていても警告が出る一方、将来の整形差分で policy の信頼性が落ちる。

## 影響

- source policy regression の警告に誤検知が混ざり、実際の raw-memory boundary 漏れとの区別が難しくなる。
- Vec/ResourceIR 周辺の安全境界を監視するテストが、コード整形により不安定になる。
- warn-only CI で後続処理は継続するが、静的検査大規模修正の進捗判断を誤らせる。

## 修正方針

- `nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` に raw boundary path を path segment 配列から照合する補助関数を置く。
- whitelist に含めるべき path と含めてはいけない path の両方で同じ補助関数を使い、Rust 配列の改行・空白・末尾カンマ・rustfmt の整形に依存しない検査へ統一する。
- `nepl-core/src/loader.rs` の whitelist 自体は正しいため、機能側の変更は行わない。

## 検証

- `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js`
- `node nodesrc/run_source_policy_regressions.js --warn-only`
- `node nodesrc/issues.js check --dir issues`

### 検証結果

- 2026-05-12: `node nodesrc/test_stdlib_vec_no_unsafe_unwraps.js` passed.
- 2026-05-12: `node nodesrc/run_source_policy_regressions.js --warn-only` passed without source-policy warnings.
- 2026-05-12: `node nodesrc/issues.js check --dir issues` passed.
