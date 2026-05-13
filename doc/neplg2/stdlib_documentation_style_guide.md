# NEPLg2 stdlib documentation style guide

作成日: 2026-05-13

関連 issue:

- [ISS-20260513T212824962Z-STDLIB-DOCUMENTATION-STYLE-GUIDE-NEE-DE6542E2](../../issues/items/ISS-20260513T212824962Z-STDLIB-DOCUMENTATION-STYLE-GUIDE-NEE-DE6542E2.md)

## 目的

NEPLg2 の stdlib に置く `//:` documentation comment は、利用者向け説明であると同時に API 契約である。コード量を抑えるために documentation comment を削ることはしない。ファイルが大きくなった場合は責務で分割し、分割先にも必要な documentation を置く。

この文書は、既存の [stdlib ドキュメントコメント整備方針](../stdlib_doc_comment_policy.md) と [documentation contract plan](./stdlib_documentation_contract_plan.md) を踏まえ、実際にどの粒度・文体・doctest 方針で書くかを定める。

## 調査結果

2026-05-13 時点の `nodesrc/test_stdlib_documentation_contract.js` による観測では、module doc は欠落していないが、module doctest と declaration doc/doctest はまだ大きく不足している。

| 項目 | 現状 |
|---|---:|
| 対象 `.nepl` file | 399 |
| module doc missing | 0 |
| module doctest missing | 308 |
| declaration | 1750 |
| declaration doc missing | 532 |
| declaration doctest missing | 1031 |

この数値は「これ以上悪化させない baseline」であり、十分な documentation が揃ったことを意味しない。今後は baseline を下げ、最終的に missing を 0 に近づける。

### `stdlib/kp/kpgraph.nepl`

良い点:

- module doc が、密行列表現、BFS、0-index/1-index、O(n^2) memory を説明している。
- module doctest が stdin/stdout を使い、競技プログラミングでの典型的な利用形を示している。
- `dense_graph_bfs_dist_raw` は BFS の未訪問 sentinel、配列キュー、計算量を説明している。

不足:

- `kp_push_i32` のような helper は一行説明だけで、失敗時に owner をどう扱うか、なぜ結果 struct が必要かが薄い。
- `DenseGraph` は `mat <i32>` が raw pointer であること、誰が解放するか、`Copy` できない owner handle として扱うべきことが明確ではない。
- module doctest は raw `mem_ptr_addr` / `load_i32` へ踏み込んでいる。`kp` の性能層としては許容できるが、通常 stdlib の典型例に raw memory discipline を混ぜるべきではない。
- declaration doctest はほぼ無い。全関数へ重い統合 doctestを置く必要はないが、代表 API と誤用しやすい API には典型例や compile_fail を置くべきである。

結論として、`kpgraph` は「何のアルゴリズムでどの計算量か」を示す方向は良い。一方で、所有権・raw boundary・失敗時契約をもう一段具体化する必要がある。

## 基本方針

### documentation は仕様である

- documentation comment は装飾ではなく、API 仕様の一部として扱う。
- 実装変更で目的、戻り値、所有権、効果、計算量、失敗条件が変わる場合、同じ commit で documentation を更新する。
- 古いコメントは無いコメントより悪い。迷った場合は、実装の現在の contract を短く正確に書き直す。

### 手書きで固有情報を書く

- テンプレート文や機械生成のような反復説明は避ける。
- 「この関数は値を返します」のような、どの関数にも貼れる文は書かない。
- その API に固有の設計判断、前提条件、アルゴリズム、不変条件を書く。

### 日本語として自然に書く

- 見出しは短くし、本文は日本語として読める文にする。
- 不自然な直訳や過剰な専門語の羅列を避ける。
- 用語は統一する。例: 所有権、借用、初期化済み cell、未初期化、解放責務、半開区間、償却計算量。
- 注意書きは曖昧にしない。「危険です」ではなく、何が壊れるかを書く。

### ルビは理解を助けるために使う

`//:` documentation は `stdlib/nm` と同じ拡張 markdown として扱う。`[目的/もくてき]` のようなルビ記法は使ってよい。

推奨:

- 見出しの固定語: `[目的/もくてき]`、`[実装/じっそう]`、`[注意/ちゅうい]`、`[計算量/けいさんりょう]`。
- 初出の難しい語: `[償却/しょうきゃく][計算量/けいさんりょう]`、`[半開区間/はんかいくかん]`、`[隣接行列/りんせつぎょうれつ]`。
- 誤読しやすい語や、教育的に説明したい語。

避ける:

- すべての漢字へ機械的にルビを付ける。
- 変数名や型名の中へ不要なルビを入れる。
- 読みやすさより装飾が目立つ書き方。

## 構造

### module doc

各 `.nepl` file の先頭には module doc を置く。

必須内容:

- その file が集める機能。
- stdlib layer と責務。例: `core`、`alloc`、`std`、`kp`、`nm`。
- facade か実装本体か。
- raw memory、external I/O、target 依存、allocator 依存、effect がある場合の境界。
- 主要な使い方を示す module-level doctest。

`kpgraph` のような performance helper では、module doc で次を明示する。

- 想定用途。例: 競技プログラミング、小中規模入力。
- 入力制約と計算量が合わない場合の不向きさ。
- 0-index / 1-index 変換。
- raw memory を使う場合、所有権と cleanup の責務。

### type doc

`struct` / `enum` / `trait` の直前には type doc を置く。

必須内容:

- その型が表す概念。
- field / variant / trait method の意味。
- 不変条件。
- `Copy` / `Clone` / `Drop` / move の扱い。
- constructor を直接使ってよいか、helper を通すべきか。
- raw pointer や owner token を持つ場合は、誰がいつ解放するか。

例: `DenseGraph` なら `n`、`mat`、`mat` の layout、`dense_graph_free` の必要性、shallow copy 禁止を説明する。

### function doc

関数 doc は、公開 API と内部 helper のどちらにも置く。内部 helper は doctest まで必須にしないが、責務と安全条件は省略しない。

基本節:

- `[目的/もくてき]`: 呼び出し側が何のために使うか。
- `[実装/じっそう]`: アルゴリズム、delegation、data layout、state transition。
- `[注意/ちゅうい]`: 前提条件、失敗条件、所有権、borrow、effect、panic/unreachable。
- `[計算量/けいさんりょう]`: 時間計算量と必要なら空間計算量。平均/最悪/償却を分ける。

必要に応じて追加する節:

- `[入力形式/にゅうりょくけいしき]`
- `[出力形式/しゅつりょくけいしき]`
- `[安全性/あんぜんせい]`
- `[target]`
- `[移行/いこう]`

## 計算機科学的な正確性

documentation では、実装の見た目ではなく contract を正確に書く。

### アルゴリズム

- BFS / DFS / Dijkstra / Fenwick tree / binary search などは、標準名だけで終わらせず、実装上の状態を説明する。
- sentinel を使う場合、値と意味を明示する。例: `-1` は未訪問。
- `match` で扱う enum state は、variant ごとの意味を書く。

### 計算量

- `O(n)` だけでなく、`n` が何を表すかを書く。
- `HashMap` は平均と最悪を分ける。
- `Vec.push` のような API は償却計算量と再確保時計算量を分ける。
- `kpgraph` の dense graph は、辺数 `m` ではなく `n^2` 走査であることを書く。
- memory 使用量が重要な API は空間計算量も書く。

### 所有権とメモリ

- owner を消費する関数は「渡した値を以後使えない」ことを書く。
- cleanup が必要な型は、解放関数名を書く。
- raw pointer / `MemPtr` / `RegionToken` / `OwnedBuffer` などは、owner か non-owning view かを書く。
- 失敗時に入力 owner を返すか、消費するか、解放するかを書く。
- allocation failure を空値 success に潰す API は、なぜその方針なのかを明記する。

### effect / target

- `stdio`、`fs`、`env`、`streamio` など外部 I/O を行う API は effect と target 条件を書く。
- `#target std` / WASI / LLVM などに依存する doctest は、必要な tag と import を明示する。
- pure に見える API の内部で raw memory を使う場合は、現在の移行状態と boundary を説明する。

## doctest 方針

documentation comment 内の `neplg2:test` は、実装を網羅的に検証するためではなく、利用者に典型的な使い方を示すために置く。

### doctest に書くもの

- 最小だが実用的な呼び出し例。
- `Result` / `Option` / owner-bearing result の受け方。
- import、target、型注釈が重要な API の使い方。
- 代表的な stdout / assertion 出力。
- 誤用しやすい API の `compile_fail`。

### doctest に押し込まないもの

- 全境界条件の網羅。
- performance stress。
- compiler regression の細かい再現。
- source policy で見るべき構造検査。

これらは `tests/stdlib`、`stdlib/tests`、`nodesrc/test_stdlib*.js` へ置く。

### stdout と assertion

- 実行 doctest は、exit code だけでなく `std/test` の assertion report や `print` / `println` で結果が読めるようにする。
- `stdout:` を使う場合は、利用者が期待する結果を具体的に示す。
- 戻り値だけを使う古い doctest は、順次 stdout/assertion report へ移す。

### cleanup

- allocation を伴う doctest は、使い終わった owner を解放する。
- raw memory を直接扱う doctest は、通常 API の例として置かない。raw/performance/API 自体の説明で必要な場合だけにする。
- `kp` のような performance layer の例でも、cleanup を省略しない。

### compile_fail / should_panic

- `compile_fail` は、利用者が踏みやすい誤用を示す時に使う。例: non-Copy payload を Copy-only collection API に渡す。
- `should_panic` は、API が意図的に panic/unreachable することを利用者へ示す場合に限る。
- static-check の正確性を下げるために doctest を弱めてはならない。

## 層ごとの重点

| 層 | documentation の重点 |
|---|---|
| `core` | primitive、trait、type/effect/memory の意味。compiler が頼る contract。 |
| `alloc` | owner transfer、allocation failure、collection state、UTF-8、encoding、hash。 |
| `std` | external I/O、target、fd/scratch buffer、cleanup、stdout/stderr/stdin。 |
| `kp` | 入力制約、計算量、raw/performance 境界、競技用途としての割り切り。 |
| `nm` | 拡張 markdown grammar、section/inline state、HTML/JSON escaping。 |
| `neplg2` selfhost | compiler model、diagnostic enum、static-check parity、source range。 |

## `kp` の方針

`kp` は競技プログラミング向けの暫定 performance layer であり、一般 stdlib の safe API とは別の説明が必要である。

書くべきこと:

- 想定する入力規模。
- 計算量とメモリ量。
- 0-index / 1-index の規則。
- raw memory を使う場合の cleanup。
- invalid input を検査するか、競技用途として前提にするか。
- 一般 API へ昇格する場合に直すべき点。

`kpgraph` の `dense_graph_bfs_dist_raw` は、dense matrix の `O(n^2)` BFS としては説明が適切である。ただし、`raw` suffix の意味、返却 `Vec<i32>` の owner、allocation failure 時の fallback、`dense_graph_free` の必要性を型・関数 doc へ追加するべきである。

## 移行計画

### Stage 0: style guide 固定

- この文書を方針として追加する。
- root の `doc/stdlib_doc_comment_policy.md` と `doc/neplg2/stdlib_documentation_contract_plan.md` から参照する。
- baseline policy は現状の悪化防止として維持する。

### Stage 1: audit と優先順位

- `stdlib/core/mem`、`alloc/collections/vec`、`alloc/io`、`std/streamio` など safety-critical な module から更新する。
- `kp` は raw/performance 境界を明文化し、一般 stdlib へ持ち込むべきでない raw discipline を分離する。
- declaration doc missing と declaration doctest missing を module ごとに下げる。

### Stage 2: representative doctest

- 各 module に module-level doctest を置く。
- 代表 public API に典型例を置く。
- 内部 helper は説明を厚くし、必要なものだけ doctest を置く。

### Stage 3: baseline 引き下げ

- documentation contract baseline を更新し、missing 数を段階的に減らす。
- boilerplate な doctest 増殖ではなく、実際に読める doc を増やす。

## チェックリスト

documentation 追加・更新時は次を確認する。

- 日本語として自然か。
- 実装の現在の contract と一致しているか。
- API の目的が、その API 固有の言葉で書かれているか。
- 所有権、borrow、effect、target、raw memory、失敗時の扱いを必要に応じて書いたか。
- 計算量の `n` や前提条件が明確か。
- doctest は典型的な使い方を示しているか。
- doctest は stdout/assertion/cleanup を含むか。
- 実装の網羅テストを documentation comment に押し込んでいないか。
