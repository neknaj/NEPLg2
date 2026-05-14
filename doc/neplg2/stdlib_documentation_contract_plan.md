# NEPLg2 stdlib documentation contract plan

作成日: 2026-05-12

関連 issue:

- [ISS-20260512T142631679Z-STDLIB-DOCUMENTATION-CONTRACT-IS-NOT-1FB48841](../../issues/items/ISS-20260512T142631679Z-STDLIB-DOCUMENTATION-CONTRACT-IS-NOT-1FB48841.md)
- [ISS-20260513T212824962Z-STDLIB-DOCUMENTATION-STYLE-GUIDE-NEE-DE6542E2](../../issues/items/ISS-20260513T212824962Z-STDLIB-DOCUMENTATION-STYLE-GUIDE-NEE-DE6542E2.md)
- [ISS-20260513T213429992Z-GENERAL-STDLIB-DOCUMENTATION-AUDIT-L-E7BDE73F](../../issues/items/ISS-20260513T213429992Z-GENERAL-STDLIB-DOCUMENTATION-AUDIT-L-E7BDE73F.md)

## 目的

NEPLg2 の stdlib は、コードだけでなく executable documentation も API 契約の一部として扱う。ファイル分割で 1 ファイルの肥大化を避けることは必要だが、肥大化対策としてドキュメントコメントや doctest を削ることは禁止する。

この文書は、`stdlib/core`、`stdlib/alloc`、`stdlib/std` の module / function / enum / struct / trait に対して、どの程度の documentation contract を必須にするか、現在の不足をどう測定し、どの順序でゼロへ近づけるかを定める。

## 契約

具体的な書き方、見出し、ルビ、計算量、doctest の責務分離は [NEPLg2 stdlib documentation style guide](./stdlib_documentation_style_guide.md) を正とする。この文書は coverage baseline と移行計画を扱い、style guide は「十分丁寧で親切な documentation comment」をどう書くかを扱う。

### module

すべての `.nepl` file は、先頭に `//:` の module documentation を持つ。

module documentation は次を説明する。

- module が集める機能と責務。
- facade か実装本体か。
- raw memory / owner / effect / target 依存がある場合、その境界。
- 主要 submodule の役割。
- module-level doctest。facade でも import 経路が生きていることを示す最小 doctest を置く。

### declaration

`fn`、`struct`、`enum`、`trait` は、それぞれ直前に `//:` documentation を持つ。

関数 documentation は次を説明する。

- 目的。
- 実装アルゴリズムまたは delegation 先。
- 所有権、borrow、raw memory、effect、panic / error の注意点。
- 計算量。O(1) でも明記する。
- 使用例または compile-fail / should-panic を含む doctest。

`struct` / `enum` / `trait` documentation は次を説明する。

- 値が表す概念。
- field / variant / capability の意味。
- 所有権と Copy / Clone / Drop の関係。
- match や constructor の最小 doctest。

### 禁止事項

- ファイルを短くする目的で documentation comment を削除してはならない。
- facade 化した file から implementation body を移す場合でも、facade の module doc と import-path doctest は残す。
- unsafe/raw helper の documentation を薄くしてはならない。むしろ raw memory boundary、caller obligation、計算量を明示する。
- doctest が一時的に重い場合、削除ではなく `tests/stdlib` への大型 regression 分離や focused doctest への分割で対応する。

## 現状監査

2026-05-12 時点で `nodesrc/test_stdlib_documentation_contract.js` と同じ規則で確認した結果:

| 項目 | 件数 |
| --- | ---: |
| 対象 `.nepl` file | 385 |
| module doc missing | 0 |
| module doctest missing | 309 |
| declaration | 1745 |
| declaration doc missing | 547 |
| declaration doctest missing | 1032 |

評価:

- module doc は全 file に存在するため、先頭 documentation の方針はおおむね守られている。
- module doctest は不足が大きい。facade / implementation file の import-path regression が十分ではない。
- declaration doc / doctest は未達であり、「すべての関数や module や enum や struct に十分な documentation と doctest」を満たしているとは言えない。
- 既に documentation が厚い file と薄い file の差が大きく、局所的な source policy だけでは全体方針を維持できない。

## 2026-05-13 style guide 監査

`stdlib/kp/kpgraph.nepl` を代表例として確認した。module doc は密行列表現、BFS、0-index/1-index、O(n^2) memory を示しており、利用者にとって有用な情報を持つ。一方で、helper 関数や owner-bearing struct の declaration doc は薄い箇所があり、raw memory、解放責務、失敗時 owner contract、典型例の置き方をさらに明確にする必要がある。

このため、coverage baseline だけではなく、文章品質と doctest の役割を定める style guide を追加した。今後 baseline を下げる時は、単に `neplg2:test` を増やすのではなく、style guide の観点で利用者が読む価値のある documentation を増やす。

## 2026-05-13 一般 stdlib 追加監査

`kp` は特殊な performance layer であり、一般 stdlib の documentation 方針を `kpgraph` だけから決めると raw/performance 境界の説明へ偏る。そのため、`alloc/hash/sha256.nepl`、`core/result.nepl`、`alloc/string/storage.nepl`、`alloc/io/bytebuf.nepl`、`std/test/types.nepl`、`std/streamio/scanner/state.nepl` を追加確認した。

評価:

- `alloc/hash/sha256.nepl` は facade と submodule 責務、incremental hash state、finalize の state 消費、内部 buffer 解放、計算量の説明が良い。一方で facade の import-path doctest と digest lifecycle 例が不足している。
- `core/result.nepl` は `Copy` 制約、`should_panic`、`compile_fail` が良い。一方で旧見出しと `ret:` 中心 doctest が残り、新方針の stdout/assertion 例へ移す必要がある。
- `alloc/string/storage.nepl` は raw storage layout と owner 境界の説明が厚い。内部 helper でも raw/safety 条件を薄くしてはいけない。
- `alloc/io/bytebuf.nepl` は module doctest と owner doc が良いが、小さい public helper の declaration doc が不足している。
- `alloc/collections` は `Vec` が facade / storage state / doctest / move-after-use compile_fail の基準例になる。一方で bitset / adjacency_matrix / fenwick / binary_heap などの layout、storage、order helper には declaration doc 欠落が多く、collection 固有の owner flow、slot invariant、index formula、algorithm complexity を明文化する必要がある。
- `std/test/types.nepl` は enum / struct で test report を表す方向が良いが、stable renderer output と owner-consuming helper の契約をさらに書く必要がある。
- `std/streamio/scanner/state.nepl` は scanner の copy / close 規則と typed cursor storage の方向が良いが、`ByteBuf` owner と cursor storage を分ける理由、`scanner_from_bytes` の成功/失敗時 owner 消費、token slice helper の memory-safety contract はさらに厚くする必要がある。

この監査により、Stage 1 から Stage 3 では `kp` ではなく通常利用される `core` / `alloc` / `std` API を優先して baseline を下げる。特に `sha256` のような facade は実装本体を持たなくても module-level doctest を置き、`ByteBuf` や `StreamScanner` のような owner-bearing helper は小さな public API でも doc を必須にする。`alloc/collections` は `Vec` を基準例にしつつ、bitset / graph / tree / heap / map / queue 系の helper へ同じ水準を広げる。

## source policy

`nodesrc/test_stdlib_documentation_contract.js` を追加し、現状の不足数を baseline として固定する。

この policy は次を拒否する。

- module doc が 1 件でも消える。
- module doctest missing 数が baseline より増える。
- declaration doc missing 数が baseline より増える。
- declaration doctest missing 数が baseline より増える。

これは最終状態ではない。baseline は「悪化を止めるための凍結線」であり、計画の進行に合わせて数を下げ、最終的には missing 数を 0 にする。

## 実装計画

### Stage 0: baseline freeze

目的: documentation を削って進める退行を止める。

作業:

- `nodesrc/test_stdlib_documentation_contract.js` を source policy runner に追加する。
- `note.n.md` と issue に baseline 数を記録する。
- documentation 削除が必要に見える場合は、削除ではなく module split / doctest relocation / doc rewrite のどれかとして扱う。

完了条件:

- source policy runner が documentation contract baseline を実行する。
- baseline より悪化した場合に CI warning と local failure が出る。

### Stage 1: core declaration docs

目的: `stdlib/core` の primitive / trait / mem / result / option / math を最初に固める。

作業:

- `core/mem.nepl` は memory safety contract と compiler responsibility boundary を明記する。
- `core/traits/*` は trait capability の意味、Copy / Clone / Drop / Eq / Ord / Hash / Serialize などの静的検査上の意味を明記する。
- `core/math/**` は backend intrinsic、wrap / signed / unsigned / NaN / saturation の規則を明記する。

完了条件:

- `stdlib/core` の declaration doc missing が 0。
- public function / enum / struct / trait の doctest が揃う。

### Stage 2: alloc ownership docs

目的: owned storage、collection、string、json、hash、io buffer の所有権契約を明文化する。

作業:

- `Vec` / map / set / heap / queue / stack 系は free obligation、borrowed observer、mutation、drop order、partial initialization の説明を持つ。
- `alloc/string` は byte length / UTF-8 / raw storage / builder ownership を file ごとに明記する。
- `alloc/io` は `ByteBuf` / writer trait の owner transfer と cleanup obligation を明記する。

完了条件:

- `stdlib/alloc` の declaration doc missing が 0。
- owner-bearing API は compile-fail または should-panic を含む doctest を必要に応じて持つ。

### Stage 3: std target / IO docs

目的: WASI / filesystem / stdio / stream IO の target effect と owner cleanup を明文化する。

作業:

- `std/fs` / `std/stdio` / `std/env` は external I/O effect、scratch buffer、fd lifecycle を説明する。
- `std/streamio` は scanner / writer の state ownership と buffer boundary を説明する。

完了条件:

- `stdlib/std` の declaration doc missing が 0。
- stdout / stderr / stdin / fs effect を持つ public API に実行 doctestか、環境依存なら明確な skip doctest と理由がある。

### Stage 4: doctest zero gap

目的: declaration doctest missing を 0 にする。

作業:

- 小さい pure function は近傍 doctest。
- 重い collection integration は API 近傍の小 doctestと `tests/stdlib` の広い regression に分割する。
- compile-fail / should-panic / stdout assertion を必要に応じて使い、exit code だけに依存しない。

完了条件:

- module doctest missing = 0。
- declaration doctest missing = 0。
- `nodesrc/test_stdlib_documentation_contract.js` の baseline がすべて 0。

## 進捗状況

- `stdlib/core`: documentation coverage 未完了。先頭 module doc は存在するが、declaration doctest missing が多い。
- `stdlib/alloc`: documentation coverage 未完了。`alloc/hash/sha256.nepl` と `alloc/string/storage.nepl` は設計意図の説明が厚いが、facade doctest と public helper doc の不足がある。`alloc/collections` は `Vec` が比較的良い基準例だが、layout/storage/order helper、owner-preserving update error、slot invariant、計算量 doc の不足を優先的に下げる。
- `stdlib/std`: documentation coverage 未完了。text/stdio/streamio など一部は厚いが、target I/O の doctest 方針が一貫していない。`std/test/types.nepl` の stable output contract と `std/streamio/scanner/state.nepl` の header helper doc を優先する。
- `nodesrc/test_stdlib_documentation_contract.js`: Stage 0 baseline policy として追加。
- `run_source_policy_regressions.js`: documentation contract policy を実行対象へ追加。

