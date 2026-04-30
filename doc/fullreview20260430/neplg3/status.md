# NEPLg3 Review

対象 commit: `f108cebd`

## 対象

- `doc/neplg3/**`
- `stdlib/neplg3/**`

## 概要

NEPLg3 は現行 NEPLg2.0 selfhost の実装場所ではない。`doc/neplg2/self_host_plan.md` でも、NEPLg2.0 selfhost は `stdlib/neplg2/` を正とし、`doc/neplg3/impl/compiler_structure.md` は分割・依存方向・巨大ファイル分割の参考に限定すると明記されている。

`doc/neplg3/spec` と `doc/neplg3/impl` は次世代仕様の設計文書として厚い。一方で `stdlib/neplg3/` の実装は小さな placeholder module 群で、`neplg2:test[skip]` の skeleton が中心である。

## Actions 根拠

Actions run `25157230630` の `stdlib-test` は `stdlib` 全体を対象にしているが、NEPLg3 は実質 skip doctest の skeleton であり、主要 failure は `stdlib/neplg2`, `stdlib/alloc`, `stdlib/std`, `stdlib/tests` 側である。build job の doc HTML generation は success しているため、`doc/neplg3` は少なくとも docs generation の対象としては破綻していない。

## 良い点

- `doc/neplg3/spec` は syntax/types/effects/modules/errors/memory/stdlib/platform など章立てがある。
- `doc/neplg3/impl/compiler_structure.md` は NEPLg2 selfhost の分割参考として有用。
- `doc/neplg3/README.md` は NEPLg2.1 検討を NEPLg3 として扱うと明記している。

## 問題

- `stdlib/neplg3/` は実装ではなく placeholder に近い。
- README や一部説明から、NEPLg2 selfhost と NEPLg3 bootstrap compiler の関係が読み手に混ざりやすい。
- NEPLg3 仕様は現行 NEPLg2 の後方互換を保証するものではなく、今の selfhost 実装の正にはできない。

## 必要な設計

- NEPLg2 selfhost の正規実装は `stdlib/neplg2/` と明記し続ける。
- NEPLg3 docs は「参考設計」と「将来仕様」を分けて管理する。
- `stdlib/neplg3/` placeholder は、実装開始時に skip doctest のまま増やさず、stage/acceptance criteria を明記する。

## 進捗状況

- `doc/neplg3/spec`: 設計文書あり。
- `doc/neplg3/impl`: compiler structure あり。
- `stdlib/neplg3`: placeholder。
- NEPLg2 selfhost への直接実装: 対象外。
