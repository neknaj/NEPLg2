---
id: ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD
title: ".n.md tests rely on return values instead of stdout assertion reports"
area: TEST
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-29
updated: 2026-05-14
target: "tests/**/*.n.md, stdlib/**/*.nepl, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/std/test.nepl"
---

# ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD: .n.md tests rely on return values instead of stdout assertion reports

## 概要

`.n.md` の assertion 系 test が `main` の返す `i32` だけで可否を表し、stdout に検査内容を出さないケースが多い。

`main` の `i32` は runner では exit code 相当として扱えるが、失敗時に「どの assertion が、どの expected/actual で落ちたか」を fixture の期待値として確認できない。selfhost でも同じ `.n.md` を使うには、assertion report を stdout に出し、exit code は可否だけを表す運用に統一する必要がある。

## 対象

- `tests/**/*.n.md, stdlib/**/*.nepl, nodesrc/tests.js, nodesrc/run_doctest.js, stdlib/std/test.nepl`

## 根拠

- 2026-04-29 時点の調査では、`tests` / `tutorials` / `stdlib` の doctest 1481 件中、`ret:` を持つものは 719 件、stdout 期待値を持つものは 98 件である。
- `ret:` だけで stdout/stderr を持たない doctest は 710 件ある。
- `std/test`、`checks_exit_code`、`assert_*` などを使う assertion 系 doctest は 227 件ある。そのうち 116 件は `ret:` だけで、stdout report を期待していない。
- `checks_exit_code` は 171 箇所で使われている一方、`checks_print_report` は 101 箇所であり、検査結果の表示を伴わない assertion suite が残っている。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` は stdout が未指定の `std/test` case について `FAIL:` 行を検出する ad hoc な保険を持つが、これは成功時の詳細 report を仕様として固定するものではない。

## 問題

- exit code 相当の `i32` は 0/1 しか表現せず、失敗内容の情報量が不足する。
- `ret:` が「言語仕様としての戻り値期待」と「テスト成功/失敗の exit code 期待」を兼ねており、`.n.md` manifest の意味が曖昧である。
- stdout report を fixture に固定しないため、Rust runner と selfhost runner の assertion 表示、集約順、failure formatting の差異を検出できない。
- `std/test` を使っていても `checks_print_report` を呼ばない test があり、CI 上の失敗時に詳細確認がしにくい。

## 影響

- selfhost runner へ `.n.md` を共通利用するとき、Rust 側と selfhost 側が同じ exit code を返しても、失敗 detail や report format の互換性が確認できない。
- test failure の原因調査が runner log や local reproduction に依存し、`.n.md` 単体を読んでも期待される assertion report が分からない。
- 将来 `ret:` の意味を拡張すると、言語戻り値 test と exit code test が衝突する。

## 修正方針

- `.n.md` manifest に `exit_code:` を追加し、process / WASI / selfhost CLI の終了可否は `exit_code:` で表す。`ret:` は言語レベルの戻り値を検証する場合に限定する。
- assertion suite は stdout に deterministic な report を出す。標準形は `std/test` の report helper を通し、最後に `test_report_exit_code` 相当の helper で 0/1 を返す。
- `std/test` を import する assertion-style doctest について、stdout report なしの `ret:` だけ運用を runner か lint で検出する。
- 既存 fixture は一括置換ではなく、`std/test` 再設計後の安定 API に合わせて段階的に移行する。
- `core` target のように stdout を持たない層は、assertion report 必須の対象から分ける。core-only の primitive semantics は `ret:` または compile diagnostic で扱い、std stdout report と混同しない。

## 検証

- `parser.ts` / `parser.js` の metadata parser が `exit_code:` と `diag_code:` を保持する regression を追加する。
- `nodesrc/tests.js` と `nodesrc/run_doctest.js` が同じ expectation logic で `exit_code` / `stdout` / `stderr` / `diag_code` を検査することを確認する。
- `std/test` を使う代表 fixture を stdout report + exit code 期待に移行し、失敗時に stdout diff で assertion detail が見えることを確認する。

## 2026-04-29 進捗メモ

`ISS-20260429T105448222Z-N-MD-RUNNER-LACKS-EXIT-CODE-METADATA-5A5AEFD1` で、`.n.md` parser / focused runner / aggregate runner に `exit_code:` metadata を追加した。

この issue の残りは、既存 assertion suite の `ret:` 代用を stdout report + `exit_code:` へ移行することと、stdout report 省略を検出する lint / runner policy の追加である。

## 2026-04-29 進捗メモ 2

`ISS-20260429T102809685Z-STDLIB-ASSERT-API-MIXES-ASSERTION-RE-0F17011A` で、`std/test` に `AssertionStatus` / `AssertionKind` / `TestAssertion` / `TestReport` ベースの structured report API を追加し、`tests/stdlib/std_test_collect.n.md` を canonical `test_report_*` + `stdout:` + `exit_code:` fixture へ更新した。

この issue の残りは、既存の `checks_*` / `check_*` 中心の assertion suite を canonical `test_report_*` API へ移行し、report 省略を lint / runner policy で拒否することである。

## 2026-04-30 direct std/test assertion discard subcase

`tests/stdlib/hash_collection_rehash.n.md` の HashMap doctest 修正中に、direct assertion へ `std/test::assert_eq_i32` を使って戻り値を捨てると `TestAssertion` owner obligation が残ることを確認した。

原因:

- `std/test::assert_eq_i32` は report 集約用 API で、`TestAssertion` を返す。
- 返した `TestAssertion` は `checks_push` / `test_report_add` / `run_checks` などに渡して消費する必要がある。
- 即時失敗型の assertion を書きたい doctest では `core/test::assert_eq_i32` を import すべきで、`std/test` の assertion 値を単に捨てるのは API 誤用である。

今回の HashMap rehash doctest は `core/test` に切り替えて解消した。残件として、`std/test` を import して `assert_*` / `check_*` の戻り値を report へ集約しない fixture を lint で検出する方針を追加する。

## 2026-04-30 std/test assertion discard source policy

`ISS-20260429T231611047Z-STD-TEST-ASSERTION-DISCARD-SOURCE-PO-B9226736` で、direct discard subcase の source policy を追加した。

この policy は `std/test` を import する `.n.md` / NEPL doc-comment doctest で、`assert_*` / `check_*` を semicolon-terminated bare statement として捨てる書き方を禁止する。helper 関数が assertion を末尾式として返し、caller が report へ集約する書き方は許可する。

policy 追加時に見つかった既存の direct discard は、該当 stdlib doc-comment doctest 側も `checks_push` / `checks_exit_code` へ移行した。

この issue 全体の残件は、既存 assertion suite の `ret:` 代用を stdout report + `exit_code:` へ段階的に移行すること、および report 省略のより広い lint / runner policy を整備することである。

## 2026-05-13 BitSet stdout report migration

`stdlib/tests/bitset.n.md` と `tests/stdlib/bitset_collections.n.md` の BitSet focused doctest 6 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `assert` / `assert_eq_i32` を `test_report_push` で集約し、最後に `test_report_print_stdout` と `test_report_exit_code` で stdout report と exit code を分離した。
- `contains` / `remove` / `fill` / `len` / owner 回収の各観測結果を assertion label として stdout に残すようにした。

検証:

- `node nodesrc\run_doctest.js -i stdlib\tests\bitset.n.md -n 1`: pass
- `node nodesrc\run_doctest.js -i stdlib\tests\bitset.n.md -n 2`: pass
- `node nodesrc\run_doctest.js -i stdlib\tests\bitset.n.md -n 3`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\bitset_collections.n.md -n 1`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\bitset_collections.n.md -n 2`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\bitset_collections.n.md -n 3`: pass

この issue はまだ open のまま継続する。BitSet 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 AdjacencyMatrix stdout report migration

`stdlib/tests/adjacency_matrix.n.md` と `tests/stdlib/adjacency_matrix_collections.n.md` の AdjacencyMatrix focused doctest 7 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `insert` / `remove` / `contains` / `clear` / `free` / update error owner recovery の観測結果を assertion label として stdout に残すようにした。
- non-positive length の diagnostic check は `assert_str_eq` で `CapacityExceeded` の expected / actual を stdout に固定した。

検証:

- `node nodesrc\run_doctest.js -i stdlib\tests\adjacency_matrix.n.md -n 1`: pass
- `node nodesrc\run_doctest.js -i stdlib\tests\adjacency_matrix.n.md -n 2`: pass
- `node nodesrc\run_doctest.js -i stdlib\tests\adjacency_matrix.n.md -n 3`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\adjacency_matrix_collections.n.md -n 1`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\adjacency_matrix_collections.n.md -n 2`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\adjacency_matrix_collections.n.md -n 3`: pass
- `node nodesrc\run_doctest.js -i tests\stdlib\adjacency_matrix_collections.n.md -n 4`: pass
- `node nodesrc\tests.js -i stdlib\tests\adjacency_matrix.n.md -i tests\stdlib\adjacency_matrix_collections.n.md --no-tree -o tmp\agent1-adjacency-report-tests.json -j 2 --assert-io`: total=7, passed=7

この issue はまだ open のまま継続する。AdjacencyMatrix 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 Fenwick stdout report migration

`stdlib/tests/fenwick.n.md` と `tests/stdlib/fenwick_collections.n.md` の Fenwick focused doctest 6 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `len` / `sum_prefix` / `sum_range` / `free` / add error owner recovery の観測結果を assertion label として stdout に残すようにした。
- negative length の diagnostic check は `assert_str_eq` で `CapacityExceeded` の expected / actual を stdout に固定した。
- Fenwick の public `add` API が `std/test` / `std/stdio` 内部 arithmetic を汚染しないよう、Fenwick API は `fw::...` の qualified call として使う形にした。

検証:

- `node nodesrc\tests.js -i stdlib\tests\fenwick.n.md -i tests\stdlib\fenwick_collections.n.md --no-tree -o tmp\agent1-fenwick-report-tests.json -j 2 --assert-io`: total=6, passed=6

この issue はまだ open のまま継続する。Fenwick 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 SparseSet stdout report migration

`stdlib/tests/sparse_set.n.md` と `tests/stdlib/sparse_set_collections.n.md` の SparseSet focused doctest 5 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `contains` / `insert` / `remove` / `clear` / `free` / zero universe / reallocation の観測結果を assertion label として stdout に残すようにした。
- `len` と `universe_len` は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\sparse_set.n.md -i tests\stdlib\sparse_set_collections.n.md --no-tree -o tmp\agent1-sparse-set-report-tests.json -j 2 --assert-io`: total=5, passed=5

この issue はまだ open のまま継続する。SparseSet 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 DisjointSet stdout report migration

`stdlib/tests/disjoint_set.n.md` と `tests/stdlib/disjoint_set_collections.n.md` の DisjointSet focused doctest 7 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `same` / `len` / `size` / zero-length creation / invalid index / free-after-union / owner recovery の観測結果を assertion label として stdout に残すようにした。
- `len` と component size は `assert_eq_i32` で expected / actual を stdout に固定した。
- negative length の diagnostic check は `assert_str_eq` で `CapacityExceeded` の expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\disjoint_set.n.md -i tests\stdlib\disjoint_set_collections.n.md --no-tree -o tmp\agent1-disjoint-set-report-tests.json -j 2 --assert-io`: total=7, passed=7

この issue はまだ open のまま継続する。DisjointSet 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 SegmentTree stdout report migration

`stdlib/tests/segment_tree.n.md` と `tests/stdlib/segment_tree_collections.n.md` の SegmentTree focused doctest 6 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `len` / full sum / range sum / update-free-reallocate / invalid range / update error owner recovery の観測結果を assertion label として stdout に残すようにした。
- sum と owner length は `assert_eq_i32` で expected / actual を stdout に固定した。
- negative length の diagnostic check は `assert_str_eq` で `CapacityExceeded` の expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\segment_tree.n.md -i tests\stdlib\segment_tree_collections.n.md --no-tree -o tmp\agent1-segment-tree-report-tests.json -j 2 --assert-io`: total=6, passed=6

この issue はまだ open のまま継続する。SegmentTree 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 Queue stdout report migration

`stdlib/tests/queue.n.md` と `tests/stdlib/queue_collections.n.md` の Queue focused doctest 4 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- push / len / peek / pop / pop_front / grow / clear / empty pop の観測結果を assertion label として stdout に残すようにした。
- length checks は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\queue.n.md -i tests\stdlib\queue_collections.n.md --no-tree -o tmp\agent1-queue-report-tests.json -j 2 --assert-io`: total=4, passed=4

この issue はまだ open のまま継続する。Queue 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 Deque stdout report migration

`stdlib/tests/deque.n.md` と `tests/stdlib/deque_collections.n.md` の Deque focused doctest 4 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- push_front / push_back / peek_front / peek_back / pop_front / pop_back / grow / clear の観測結果を assertion label として stdout に残すようにした。
- length checks は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\deque.n.md -i tests\stdlib\deque_collections.n.md --no-tree -o tmp\agent1-deque-report-tests.json -j 2 --assert-io`: total=4, passed=4

この issue はまだ open のまま継続する。Deque 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 RingBuffer stdout report migration

`stdlib/tests/ringbuffer.n.md` と `tests/stdlib/ringbuffer_collections.n.md` の RingBuffer focused doctest 4 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- push / len / peek / pop / pop_front / grow / clear の観測結果を assertion label として stdout に残すようにした。
- length checks は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\ringbuffer.n.md -i tests\stdlib\ringbuffer_collections.n.md --no-tree -o tmp\agent1-ringbuffer-report-tests.json -j 2 --assert-io`: total=4, passed=4

この issue はまだ open のまま継続する。RingBuffer 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 BinaryHeap stdout report migration

`stdlib/tests/binary_heap.n.md` と `tests/stdlib/binary_heap_collections.n.md` の BinaryHeap focused doctest 8 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `push` / `peek` / `pop` / `pop_max` / borrowed observer / grow / zero-capacity cleanup の観測結果を assertion label として stdout に残すようにした。
- length check は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\binary_heap.n.md -i tests\stdlib\binary_heap_collections.n.md --no-tree -o tmp\agent1-binary-heap-report-tests.json -j 1 --assert-io --dist web/dist`: total=8, passed=8

この issue はまだ open のまま継続する。BinaryHeap 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 BloomFilter stdout report migration

`stdlib/tests/bloom_filter.n.md` と `tests/stdlib/bloom_filter_collections.n.md` の BloomFilter focused doctest 4 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `insert` / `contains` / `len` / `clear` / `free` / invalid length rejection の観測結果を assertion label として stdout に残すようにした。
- length check は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\bloom_filter.n.md -i tests\stdlib\bloom_filter_collections.n.md --no-tree -o tmp\agent1-bloom-filter-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4

この issue はまだ open のまま継続する。BloomFilter 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 CountingBloomFilter stdout report migration

`stdlib/tests/counting_bloom_filter.n.md` と `tests/stdlib/counting_bloom_filter_collections.n.md` の CountingBloomFilter focused doctest 5 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `insert` / `remove` / `contains` / `len` / `clear` / `free` / non-positive length rejection の観測結果を assertion label として stdout に残すようにした。
- length check は `assert_eq_i32` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\counting_bloom_filter.n.md -i tests\stdlib\counting_bloom_filter_collections.n.md --no-tree -o tmp\agent1-counting-bloom-filter-report-tests.json -j 1 --assert-io --dist web/dist`: total=5, passed=5

この issue はまだ open のまま継続する。CountingBloomFilter 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-13 Vec collections stdout report migration

`tests/stdlib/vec_collections.n.md` の Vec focused doctest 3 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- zero-capacity storage / grow reallocation / merge-sort scratch cleanup / negative capacity rejection の観測結果を assertion label として stdout に残すようにした。
- capacity / length / error kind は `assert_eq_i32` または `assert_str_eq` で expected / actual を stdout に固定した。

検証:

- `node nodesrc\tests.js -i tests\stdlib\vec_collections.n.md --no-tree -o tmp\agent1-vec-collections-report-tests.json -j 1 --assert-io --dist web/dist`: total=3, passed=3

この issue はまだ open のまま継続する。Vec collections 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 List collections stdout report migration

`tests/stdlib/list_collections.n.md` の List focused doctest 3 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `reverse` / empty reverse / `map` / `filter` の観測結果を assertion label として stdout に残すようにした。
- map/filter の success branch は従来通り owner cleanup を行い、結果の可否だけを report へ集約した。

検証:

- `node nodesrc\tests.js -i tests\stdlib\list_collections.n.md --no-tree -o tmp\agent1-list-collections-report-tests.json -j 1 --assert-io --dist web/dist`: total=3, passed=3

この issue はまだ open のまま継続する。List collections 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 HashSet stdout report migration

`stdlib/tests/hashset.n.md` の HashSet focused doctest 2 件を、`ret: 0` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `len` / `contains` / duplicate insert / remove / missing remove / free-after-insert の観測結果を assertion label として stdout に残すようにした。
- length check は `assert_eq_i32` で expected / actual を stdout に固定し、bool check は `assert` で成功条件を report へ集約した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\hashset.n.md --no-tree -o tmp\agent1-hashset-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。HashSet 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 HashMap stdout report migration

`stdlib/tests/hashmap.n.md` の HashMap focused doctest 1 件を、stdout 期待なしの `checks_*` report から canonical `std/test` report + manifest stdout expectation へ移行した。

移行内容:

- doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `len` / `contains` / `get` / update / remove / missing remove / free-after-insert の観測結果を assertion label として stdout に残すようにした。
- 旧 `checks_*` API への集約をやめ、`test_report_new` / `test_report_push` / `test_report_print_stdout` / `test_report_exit_code` に統一した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\hashmap.n.md --no-tree -o tmp\agent1-hashmap-report-tests.json -j 1 --assert-io --dist web/dist`: total=1, passed=1

この issue はまだ open のまま継続する。HashMap 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 string-key HashMap/HashSet stdout report migration

`stdlib/tests/hashmap_str.n.md` と `stdlib/tests/hashset_str.n.md` の string-key collection focused doctest 4 件を、戻り値コードだけで assertion failure を表す形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `len` / `contains` / `get` / update / remove / missing remove / free-after-string-insert の観測結果を assertion label として stdout に残すようにした。
- `HashMap<str, i32>` と `HashSet<str>` の concat key lookup を expected / actual または bool label として固定し、string key equality の典型例を runner output で検証できるようにした。

検証:

- `node nodesrc\tests.js -i stdlib\tests\hashmap_str.n.md -i stdlib\tests\hashset_str.n.md --no-tree -o tmp\agent1-hash-string-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4

この issue はまだ open のまま継続する。string-key HashMap/HashSet 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 List stdout report migration

`stdlib/tests/list.n.md` の List focused doctest 2 件を、`ret: 0` と stdout 期待なしの `checks_*` report から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `len` / `get` / `head` / `tail` / `reverse` / `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` の観測結果を assertion label として stdout に残すようにした。
- 旧 `checks_*` API への集約をやめ、`test_report_new` / `test_report_push` / `test_report_print_stdout` / `test_report_exit_code` に統一した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\list.n.md --no-tree -o tmp\agent1-list-stdlib-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。List 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Vec stdout report migration

`stdlib/tests/vec.n.md` の Vec focused doctest 6 件を、`ret: 0` と stdout 期待なしの `checks_*` report から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `is_empty` / `data_ptr` / `len` / `get` / `replace` / out-of-range access / `u8` storage / `map` / `filter` / `fold` / `reduce` / `find` / `any` / `all` / `partition` / `take_while` / `drop_while` / `count` の観測結果を assertion label として stdout に残すようにした。
- `unwrap_ok` / `uwok` を使う doctest に `core/result` の明示 import を追加し、helper 解決を暗黙の import に依存しない形にした。

検証:

- `node nodesrc\tests.js -i stdlib\tests\vec.n.md --no-tree -o tmp\agent1-vec-stdlib-report-tests.json -j 1 --assert-io --dist web/dist`: total=6, passed=6

この issue はまだ open のまま継続する。Vec 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Option/Result stdout report migration

`stdlib/tests/option.n.md` と `stdlib/tests/result.n.md` の focused doctest 2 件を、`ret: 0` と stdout 出力なしの `checks_exit_code` から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `Option` の `is_some` / `is_none` / `unwrap` / `unwrap_or` / `and_then` / shared reference copy と、`Result` の `is_ok` / `is_err` / `unwrap_or` / `unwrap_ok` / `unwrap_err` / `and_then` を assertion label として stdout に残すようにした。
- 旧 `checks_exit_code checks` だけで合否を返す書き方をやめ、`test_report_print_stdout` で成功時の assertion detail も fixture 化した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\option.n.md -i stdlib\tests\result.n.md --no-tree -o tmp\agent1-option-result-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。Option/Result 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Cast/Math stdout report migration

`stdlib/tests/cast.n.md` と `stdlib/tests/math.n.md` の focused doctest 2 件を、`ret: 0` と stdout 期待なしの `checks_*` report から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `cast` の bool / i32 / u8 conversion と、`math` の arithmetic / bit operation / comparison を assertion label として stdout に残すようにした。
- `cast.n.md` は bool の負条件を `not` で表すようになったため、`core/math` の明示 import を追加した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\cast.n.md -i stdlib\tests\math.n.md --no-tree -o tmp\agent1-cast-math-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。Cast/Math 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Stack stdout report migration

`stdlib/tests/stack.n.md` の Stack focused doctest 9 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `new` / `push` / `len` / `peek` / `pop` / empty pop / alias pipe API / `get` keeps stack / `pop_top` keeps stack の観測結果を assertion label として stdout に残すようにした。
- 旧 fixture の「成功なら戻り値 1」をやめ、report の failed count から exit code を返す形へ統一した。

検証:

- `node nodesrc\tests.js -i stdlib\tests\stack.n.md --no-tree -o tmp\agent1-stack-stdlib-report-tests.json -j 1 --assert-io --dist web/dist`: total=9, passed=9

この issue はまだ open のまま継続する。Stack 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Stack collections stdout report migration

`tests/stdlib/stack_collections.n.md` の Stack collections focused doctest 9 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `new` / `push` / `len` / `peek` / `pop` / empty pop / `get` keeps stack / `pop_top` keeps stack / grow-clear-free reallocation の観測結果を assertion label として stdout に残すようにした。
- `stdlib/tests/stack.n.md` と同じ canonical report 方針へ揃え、collections regression 側でも report format を固定した。

検証:

- `node nodesrc\tests.js -i tests\stdlib\stack_collections.n.md --no-tree -o tmp\agent1-stack-collections-report-tests.json -j 1 --assert-io --dist web/dist`: total=9, passed=9

この issue はまだ open のまま継続する。Stack collections 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 ByteBuf result stdout report migration

`tests/stdlib/bytebuf_result.n.md` の ByteBuf result focused doctest 6 件を、`ret: 1` による合否だけの表現から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- ByteBuf / std io / fs / streamio の result roundtrip と allocation failure propagation を assertion label として stdout に残すようにした。
- `Result` pattern を使う doctest に `core/result` を明示 import し、report helper と合わせて依存関係を fixture 上で明確にした。

検証:

- `node nodesrc\tests.js -i tests\stdlib\bytebuf_result.n.md --no-tree -o tmp\agent1-bytebuf-result-report-tests.json -j 1 --assert-io --dist web/dist`: total=6, passed=6

この issue はまだ open のまま継続する。ByteBuf result 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 ByteBuilder stdout report migration

`tests/stdlib/byte_builder.n.md` の ByteBuilder focused doctest 3 件を、`ret: 0` と stdout 期待なしの旧 `checks_*` report から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- WASM header byte、unsigned LEB128 known vector、capacity growth 後の byte 保持を assertion label として stdout に残すようにした。
- growth case は ordinary doctest から `store_u8` / `mem_ptr_addr` を直接使う形をやめ、`io_bytebuf_from_str_result` と `byte_builder_push_bytebuf` の public API 経由で capacity growth を検証する形へ直した。これにより `resource.raw.memory_outside_boundary` を回避するために静的検査を緩めず、テスト側を現在の raw-memory boundary 方針へ合わせた。

検証:

- `node nodesrc\tests.js -i tests\stdlib\byte_builder.n.md --no-tree -o tmp\agent1-byte-builder-report-tests.json -j 1 --assert-io --dist web/dist`: total=3, passed=3

この issue はまだ open のまま継続する。ByteBuilder 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 CapacityStack stdout report migration

`tests/stdlib/capacity_stack.n.md` の capacity / stack-depth focused doctest 6 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- recursive depth、Vec growth length、large memory block store/load、StringBuilder length、enum Vec + recursion mix を assertion label として stdout に残すようにした。
- 移行前の focused run では 3/6 が既に失敗していた。原因は `Vec` observer の古い by-value 呼び出し、ordinary doctest からの direct raw memory block 操作、`Vec<T: Copy>` 化後も enum payload を Copy として明示していない stale fixture だった。
- memory block case は raw address 直接操作をやめ、`RegionToken` と `MemPtr` public API の同一関数内 store/load として書き直した。helper 関数に切り出すと Resource IR が initialized cell proof を関数境界越しに保持できないため、検査を弱めず証明可能な形に寄せた。
- enum payload は所有資源を持たない2値 enum として `Clone` / `Copy` を明示し、現行 `Vec<T: Copy>` 境界に合わせた。

検証:

- `node nodesrc\tests.js -i tests\stdlib\capacity_stack.n.md --no-tree -o tmp\agent1-capacity-stack-before.json -j 1 --assert-io --dist web/dist`: total=6, passed=3, failed=3 before fix
- `node nodesrc\tests.js -i tests\stdlib\capacity_stack.n.md --no-tree -o tmp\agent1-capacity-stack-report-tests.json -j 1 --assert-io --dist web/dist`: total=6, passed=6

この issue はまだ open のまま継続する。CapacityStack 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Math overload stdout report migration

`tests/stdlib/math.n.md` の math / cast overload focused doctest 5 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。既存の `cast_ambiguous_without_expected_type` skip 診断ケースは変更していない。

移行内容:

- 実行される各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- i32 / i64 / i128 arithmetic overload、qualified math facade re-export、numeric cast roundtrip を assertion label として stdout に残すようにした。
- 期待値は旧 `ret:` の値と同じ数値を `assert_eq_i32` で固定し、成功時にも何を検証したかが runner output に残る形にした。

検証:

- `node nodesrc\tests.js -i tests\stdlib\math.n.md --no-tree -o tmp\agent1-math-overload-report-tests.json -j 1 --assert-io --dist web/dist`: total=6, passed=6

この issue はまだ open のまま継続する。Math overload 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Trait text stdout report migration

`tests/stdlib/traits_text.n.md` の trait capability / text representation focused doctest 3 件を、戻り値コードまたは stdout なし `checks_*` report から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `Clone` generic bound、`Stringify` の i32 / bool / u8 表示、`Debug` の str quote / i32 / u8 表示を assertion label として stdout に残すようにした。
- 直前に修正した `Debug for u8` の `core/cast` import 退行を検出できるよう、`debug u8` assertion を追加した。

検証:

- `node nodesrc\tests.js -i tests\stdlib\traits_text.n.md --no-tree -o tmp\agent1-traits-text-report-tests.json -j 1 --assert-io --dist web/dist`: total=3, passed=3

この issue はまだ open のまま継続する。Trait text 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Numerics stdout report migration

`tests/stdlib/numerics.n.md` の numerics focused doctest 11 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- i32 decimal / hex literal、f32 literal、u8 wrapping add/sub/mul、u8 division/remainder、u8 comparison、bitwise、shift、f32 comparison を assertion label として stdout に残すようにした。
- division/remainder、bitwise、shift、comparison 系は合計値だけでなく個別の演算結果や比較結果も assertion として固定し、壊れた性質を runner output から特定できるようにした。

検証:

- `node nodesrc\tests.js -i tests\stdlib\numerics.n.md --no-tree -o tmp\agent1-numerics-before.json -j 1 --assert-io --dist web/dist`: total=11, passed=11 before migration
- `node nodesrc\tests.js -i tests\stdlib\numerics.n.md --no-tree -o tmp\agent1-numerics-report-tests.json -j 1 --assert-io --dist web/dist`: total=11, passed=11

この issue はまだ open のまま継続する。Numerics 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Drop overwrite stdout report migration

`tests/compiler/drop_overwrite.n.md` の Drop 型 local overwrite 回帰 doctest 1 件を、戻り値 0 だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- この fixture の主目的は runtime の Drop 順序観測ではなく、nodesrc 経路で `set` による旧値 drop と新値代入の HIR 展開が compile / run できることの確認なので、`drop overwrite exit marker` という到達 assertion を stdout に残す形にした。
- stdout report のため `#target std` に移しつつ、`#no_prelude` と明示 import による最小依存は維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\drop_overwrite.n.md --no-tree -o tmp\agent1-drop-overwrite-report-tests.json -j 1 --assert-io --dist web/dist`: total=1, passed=1

この issue はまだ open のまま継続する。Drop overwrite 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Char cast stdout report migration

`tests/compiler/char_cast.n.md` の char / code point 明示 cast 回帰 doctest 2 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `char` 変数から code point への明示 `cast` と、checked code point から `char` への明示 `cast` を assertion label として stdout に残すようにした。
- 暗黙変換を許さない char 型境界を、成功時にも runner output から確認できる形にした。

検証:

- `node nodesrc\tests.js -i tests\compiler\char_cast.n.md --no-tree -o tmp\agent1-char-cast-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。Char cast 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Generic impl trait args stdout report migration

`tests/compiler/generic_impl_trait_args.n.md` の generic impl trait argument 正常系 doctest 1 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。compile_fail 診断 fixture は既存どおり `type.impl.target_not_concrete` を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- concrete impl target が trait argument 側だけに現れる type parameter を量化できることを、`concrete impl generic trait arg dispatch` assertion として stdout に残すようにした。
- 抽象化機能の回帰として、許可される generic impl dispatch と拒否される generic target 診断の両方を同じ file で維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\generic_impl_trait_args.n.md --no-tree -o tmp\agent1-generic-impl-trait-args-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。Generic impl trait args 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Ret string stdout report migration

`tests/compiler/ret_string_example.n.md` の string return doctest 1 件を、runner の `ret: "hello"` 復号だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- 文字列値 `"hello"` を NEPL 側の `assert_str_eq` で検証し、`returned string value` assertion として stdout に残すようにした。
- runner の戻り値復号結果に依存せず、プログラム自身が観測した `str` 値を report へ出す方針に合わせた。

検証:

- `node nodesrc\tests.js -i tests\compiler\ret_string_example.n.md --no-tree -o tmp\agent1-ret-string-report-tests.json -j 1 --assert-io --dist web/dist`: total=1, passed=1

この issue はまだ open のまま継続する。Ret string 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 str/i32 boundary stdout report migration

`tests/compiler/str_i32_boundaries.n.md` の string literal 正常系 doctest 1 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。compile_fail 診断 fixture 2 件は `type.annotation.mismatch` / `type.return.mismatch` の境界固定が目的なので変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- string literal が raw i32 ではなく `str` として扱われ、`str_eq` で比較できることを `string literal equality result` assertion として stdout に残すようにした。
- str/i32 境界の拒否ケースと許可ケースを同じ file で維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\str_i32_boundaries.n.md --no-tree -o tmp\agent1-str-i32-boundaries-report-tests.json -j 1 --assert-io --dist web/dist`: total=3, passed=3

この issue はまだ open のまま継続する。str/i32 boundary 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Reference codegen stdout report migration

`tests/compiler/reference_codegen.n.md` の reference / Clone codegen 回帰 doctest 3 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- scalar address-of then deref、reference 経由の i32 clone、generic `MemPtr` Clone impl 解決を assertion label として stdout に残すようにした。
- report 出力のため `#target std` に移しつつ、各 case の本質である reference lowering / Clone dispatch / generic MemPtr impl resolution の観測値を維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\reference_codegen.n.md --no-tree -o tmp\agent1-reference-codegen-report-tests.json -j 1 --assert-io --dist web/dist`: total=3, passed=3

この issue はまだ open のまま継続する。Reference codegen 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Drop stdout report migration

`tests/compiler/drop.n.md` の Drop capability / auto drop insertion 回帰 doctest 4 件を、戻り値 0 だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- simple let / nested scope / if branch / multiple binding の各 compile-run 到達点を assertion label として stdout に残すようにした。
- nested scope と if branch は、旧 fixture では捨てていた branch result も assertion 化し、auto drop epilogue が block/branch value を壊していないことをより直接確認するようにした。
- runtime の Drop 順序詳細は Rust integration test 側の責務として維持し、この `.n.md` では nodesrc 経路の compile / run と return value 保持を固定した。

検証:

- `node nodesrc\tests.js -i tests\compiler\drop.n.md --no-tree -o tmp\agent1-drop-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4

この issue はまだ open のまま継続する。Drop 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Block semicolon return stdout report migration

`tests/compiler/block_semicolon_return.n.md` の block / semicolon 正常系 doctest 4 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。compile_fail 診断 fixture 6 件は parser / type checker の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- block last expression、unit context trailing semicolon、if branch value、single-line let semicolon の観測値を assertion label として stdout に残すようにした。
- `;` による unit 化の拒否ケースと許可ケースを同じ file で維持し、成功側も runner output から意味を読める形にした。

検証:

- `node nodesrc\tests.js -i tests\compiler\block_semicolon_return.n.md --no-tree -o tmp\agent1-block-semicolon-return-report-tests.json -j 1 --assert-io --dist web/dist`: total=10, passed=10

この issue はまだ open のまま継続する。Block semicolon return 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 List dot map stdout report migration

`tests/compiler/list_dot_map.n.md` の namespace / alias map 回帰 doctest 4 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `result::map`、`list::map`、star alias 経由の `result map`、star alias 経由の `vec map` を assertion label として stdout に残すようにした。
- `list::get` は現行 API に合わせて `&ys` を渡すように修正し、`list::map` が返した owning list は `list::free` で明示的に閉じた。Resource IR owner 検査を緩めず、fixture 側の所有権責務を正した。

検証:

- `node nodesrc\tests.js -i tests\compiler\list_dot_map.n.md --no-tree -o tmp\agent1-list-dot-map-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4

この issue はまだ open のまま継続する。List dot map 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Block if semantics stdout report migration

`tests/compiler/block_if_semantics.n.md` の block / match / semicolon 正常系 doctest 4 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。trailing semicolon による return mismatch の compile_fail fixture 1 件は、型検査の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- epilogue drop、match arm local、semicolon なし複数行、同一行の複数 semicolon の観測値を assertion label として stdout に残すようにした。
- 連続式の正常系は `block:` 内で値を受け、最後の式値が保持されることを report に固定した。

検証:

- `node nodesrc\tests.js -i tests\compiler\block_if_semantics.n.md --no-tree -o tmp\agent1-block-if-semantics-report-tests.json -j 1 --assert-io --dist web/dist`: total=5, passed=5

この issue はまだ open のまま継続する。Block if semantics 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Match enum wildcard stdout report migration

`tests/compiler/match_enum_wildcard_patterns.n.md` の enum wildcard 正常系 doctest 2 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。wildcard_not_last / duplicate_arm の compile_fail fixture 2 件は、match arm 検査の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- payload なし enum variant の default wildcard selection と、payload enum の default wildcard selection を assertion label として stdout に残すようにした。
- `#target std` 化で stdlib の `Outcome` 型と衝突したため、payload fixture の local enum を `LocalOutcome` / `Value` / `Missing` に改名した。wildcard payload default の検査意図は維持している。

検証:

- `node nodesrc\tests.js -i tests\compiler\match_enum_wildcard_patterns.n.md --no-tree -o tmp\agent1-match-enum-wildcard-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4

この issue はまだ open のまま継続する。Match enum wildcard 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Match literal patterns stdout report migration

`tests/compiler/match_literal_patterns.n.md` の literal pattern 正常系 doctest 9 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。duplicate literal / non-exhaustive / wildcard_not_last / unsupported pattern の compile_fail fixture 4 件は、match pattern 検査の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- i32 literal matching/default、bool literal、char literal、Unicode scalar、char literal as i32/u8 code point、integer context argument、escape literal の観測値を assertion label として stdout に残すようにした。
- 成功時も、どの literal pattern / context-sensitive char lowering が壊れたかを runner output から読める形にした。

検証:

- `node nodesrc\tests.js -i tests\compiler\match_literal_patterns.n.md --no-tree -o tmp\agent1-match-literal-report-tests.json -j 1 --assert-io --dist web/dist`: total=13, passed=13
- 実行時間は約96秒。timeout ではなく 13 doctest の個別 compile が主因であり、今回の変更による runtime hang ではない。

この issue はまだ open のまま継続する。Match literal patterns 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Overload nested generic push stdout report migration

`tests/compiler/overload_nested_generic_push.n.md` の nested generic overload 正常系 doctest 2 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。

移行内容:

- 各 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- `Vec<Result<(),str>>` への direct `push` と pipe `push` の overload 解決結果を assertion label として stdout に残すようにした。
- `free` は report 構築前に実行し、Resource IR owner obligation を閉じたうえで観測値だけを report へ渡す形を維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\overload_nested_generic_push.n.md --no-tree -o tmp\agent1-overload-nested-generic-push-report-tests.json -j 1 --assert-io --dist web/dist`: total=2, passed=2

この issue はまだ open のまま継続する。Overload nested generic push 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Prelude Copy stdout report migration

`tests/compiler/prelude_copy.n.md` の prelude / Copy capability 正常系 doctest 3 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。`#no_prelude` が Copy trait supply を無効化する compile_fail fixture 1 件は、拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- default prelude の Copy/Clone supply、`#prelude std/prelude_base` + `#no_prelude` の明示 prelude 優先、generic `MemPtr<T>` Copy impl を assertion label として stdout に残すようにした。
- stdout report 出力のため正常系は `std` target に移したが、`core/traits/copy` を直接 import しない前提は維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\prelude_copy.n.md --no-tree -o tmp\agent1-prelude-copy-report-tests.json -j 1 --assert-io --dist web/dist`: total=4, passed=4

この issue はまだ open のまま継続する。Prelude Copy 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Block single-line stdout report migration

`tests/compiler/block_single_line.n.md` の single-line `block` 正常系 doctest 20 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。single-line block に multiline if を含められない compile_fail fixture 1 件は、parser の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- literal / arithmetic / let / multiple statements / nested block / argument position / if branch / while body / semicolon unit / shadowing / mutation / type annotation / tuple element / pipe source / match arm / trailing comment / unit block / deeply nested block の観測値を assertion label として stdout に残すようにした。
- 旧 `ret:` の値だけではなく、どの single-line block 構文規則が壊れたかを runner output から読める形にした。

検証:

- `node nodesrc\tests.js -i tests\compiler\block_single_line.n.md --no-tree -o tmp\agent1-block-single-line-report-tests.json -j 1 --assert-io --dist web/dist`: total=21, passed=21
- 実行時間は約200秒。timeout ではなく 21 doctest の個別 compile が主因であり、今回の変更による runtime hang ではない。

この issue はまだ open のまま継続する。Block single-line 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Sizeof stdout report migration

`tests/compiler/sizeof.n.md` の `size_of<T>` 正常系 doctest 8 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。generic parameter の `.` 必須を固定する compile_fail fixture 1 件は、parser の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- primitive layout、generic function、generic wrapper struct、multi-field struct、algebraic type、nested generic struct、collection struct、diag struct の `size_of<T>` 観測値を assertion label として stdout に固定した。
- report 出力のため正常系は `std` target に移したが、既存の `size_of<T>` の期待値と拒否境界は維持している。

検証:

- `node nodesrc\tests.js -i tests\compiler\sizeof.n.md --no-tree -o tmp\agent1-sizeof-report-tests.json -j 1 --assert-io --dist web/dist`: total=9, passed=9

この issue はまだ open のまま継続する。Sizeof 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Generics stdout report migration

`tests/compiler/generics.n.md` の generics 正常系 doctest 16 件を、戻り値の数値だけで検証する形から canonical `std/test` report へ移行した。generic parameter syntax / type mismatch / arity mismatch の compile_fail fixture 8 件は、型検査と parser の拒否境界を固定するため変更していない。

移行内容:

- 正常系 doctest に `neplg2:test[stdio, normalize_newlines]`、`exit_code: 0`、deterministic `stdout:` を追加した。
- identity multi-instantiation、generic enum match、generic struct construction、multi type parameter function、context inference、nested generic payload、pipe into generic などの観測値を assertion label として stdout に固定した。
- `std/test` 導入で std 側の `Option` と衝突した正常系 local enum は `LocalOption` に改名し、generic enum / payload / inference の検査意図は維持した。compile_fail 側は `core` / `#no_prelude` の拒否境界を変えないため既存名のまま残した。

検証:

- `node nodesrc\tests.js -i tests\compiler\generics.n.md --no-tree -o tmp\agent1-generics-report-tests.json -j 1 --assert-io --dist web/dist`: total=24, passed=24
- 実行時間は約162秒。timeout ではなく 24 doctest の個別 compile が主因であり、今回の変更による runtime hang ではない。

この issue はまだ open のまま継続する。Generics 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Move/effect stdout report migration

`tests/compiler/move_effect.n.md` の pure/effect / Copy capability 正常系 doctest 11 件を canonical `std/test` report へ移行した。大量の compile_fail fixture は effect / Resource IR / raw memory の拒否境界を固定するため変更していない。

移行内容:

- local `set` が pure のまま扱えること、Copy impl がある struct / generic struct / enum の再利用、Copy 値 borrow 中の再利用、capability を持たない marker/clone-shaped trait が Copy/Clone 扱いされないこと、`str` / unit の Copy trait impl による再利用を stdout report に固定した。
- `std/test` 導入で `#no_prelude` の local `Clone` / `Copy` capability trait test 2 件は stdlib 側の canonical trait universe が混入し、元の検査対象を壊すことを確認した。そのためこの 2 件は `core` / `#no_prelude` / `ret: 0` のまま維持した。stdout report 化よりも isolated capability semantics の保持を優先した判断である。

検証:

- `node nodesrc\tests.js -i tests\compiler\move_effect.n.md --no-tree -o tmp\agent1-move-effect-report-tests.json -j 1 --assert-io --dist web/dist`: total=113, passed=113
- 実行時間は約354秒。timeout ではなく 113 doctest の個別 compile が主因であり、今回の変更による runtime hang ではない。

この issue はまだ open のまま継続する。Move/effect の孤立 `core` / `#no_prelude` capability test 2 件は意図的に ret 形式で残し、それ以外の `ret:` 依存 fixture と report 省略検出 policy が残っている。

## 2026-05-14 Move check stdout report migration

`tests/compiler/move_check.n.md` の move / borrow / lifetime 成功系 doctest 14 件を canonical `std/test` report へ移行した。compile_fail fixture 38 件は Resource IR / borrow / lifetime の拒否境界を固定するため変更していない。

移行内容:

- single move、non-Copy reassign、Copy reassign、shared/unique borrow last-use release、temporary call-argument borrow、Copy owner reuse、distinct field move、copy deref、match payload borrow、loop accumulator reinit、borrowed field projection の成功境界を assertion label として stdout に固定した。
- 各 case は検査対象の move / borrow / consume 操作を実行した後に `actual` を report へ流す形にし、report 化が検査対象の resource operation を迂回しないようにした。
- compile_fail 側の `resource.cell.*` / `resource.borrow.*` / `resource.borrow.return_escape` 診断境界はそのまま維持した。

検証:

- `node nodesrc\tests.js -i tests\compiler\move_check.n.md --no-tree -o tmp\agent1-move-check-report-tests.json -j 1 --assert-io --dist web/dist`: total=52, passed=52
- 実行時間は約233秒。timeout ではなく 52 doctest の個別 compile が主因であり、今回の変更による runtime hang ではない。

この issue はまだ open のまま継続する。Move check 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。

## 2026-05-14 Type annotation stdout report migration

`tests/compiler/typeannot.n.md` の type annotation 成功系 doctest 15 件を canonical `std/test` report へ移行した。

移行内容:

- literal、nested expression、let annotation、block expression、nested annotation、function call、complex expression、if expression、while condition、generic `Option<i32>`、deeply nested call、block/call/pipe/function literal mixed cases の観測値を assertion label として stdout に固定した。
- 各 case は型注釈を通した後の値を `actual` または直接 assertion input とし、runner の `ret:` 復号だけでなく NEPL 側の検査結果として stdout に残すようにした。
- この file には compile_fail fixture がないため、成功境界の stdout report 化に集中した。

検証:

- `node nodesrc\tests.js -i tests\compiler\typeannot.n.md --no-tree -o tmp\agent1-typeannot-report-tests.json -j 1 --assert-io --dist web/dist`: total=15, passed=15
- 実行時間は約155秒。timeout ではなく 15 doctest の個別 compile が主因であり、今回の変更による runtime hang ではない。

この issue はまだ open のまま継続する。Type annotation 以外の `ret:` 依存 fixture と、report 省略を検出する lint / runner policy が残っている。
