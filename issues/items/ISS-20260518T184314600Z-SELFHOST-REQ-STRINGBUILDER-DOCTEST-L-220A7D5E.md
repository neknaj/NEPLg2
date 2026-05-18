---
id: ISS-20260518T184314600Z-SELFHOST-REQ-STRINGBUILDER-DOCTEST-L-220A7D5E
title: "selfhost_req StringBuilder doctest leaks nested copy temporary owner"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/selfhost_req.n.md, nepl-core/src/resource/**"
---

# ISS-20260518T184314600Z-SELFHOST-REQ-STRINGBUILDER-DOCTEST-L-220A7D5E: selfhost_req StringBuilder doctest leaks nested copy temporary owner

## 概要

`tests/stdlib/selfhost_req.n.md::doctest#5` は `StringBuilder` を `sb_append_i32` に通したあと `sb_build` で `str` へ確定し、`len` で観測するだけの要件確認である。Resource IR はこの経路で、`sb_build` 後の `str` ではなく `sb_append_i32` 呼び出しに渡す copy state-only temporary 内の nested raw owner leaf を `resource.owner.leak` として報告していた。

## 対象

- `nepl-core/src/resource/lower_temporary_scope.rs`
- `nepl-core/src/resource/owner_summary_consumed.rs`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/selfhost_req.n.md`

## 根拠

- `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/agent1-selfhost-req-hash-update-error.json -j 1 --dist web/dist --assert-io`: total=6, passed=5, failed=1。
- 失敗箇所は `tests/stdlib/selfhost_req.n.md::doctest#5` で、`set sb sb_append_i32 sb 404` の Resource IR lowering が `read %sb -> tmp` を作り、`tmp` の nested owner leaf を呼び出し後に閉じられない temporary として残していた。
- 同じ consumed 判定を単純に広げると、`fs_read_fd_bytes` / `stdio_read_all_bytes_result` の `Result::Ok` payload owner が parameter return として materialize された後に再消費され、既存 regression `resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup` を壊すことも確認した。

## 問題

Resource IR の temporary scope 判定は copy 型そのものが `str` かどうかだけを見ており、copy aggregate の内部に owner-backed `str` leaf がある場合を見落としていた。また owner summary の consumed parameter 判定は `Moved` / `Freed` / `NoFreeObligation` だけを consumed として扱い、関数内の分岐で「返される可能性と消費される可能性がある」`MaybeFreed(storage=parameter)` を、返却 source との相関なしに表現できていなかった。

このため、`StringBuilder` wrapper のように入力 owner が callee 内部で別 owner に詰め替えられる経路では raw owner leaf が消費された事実を summary に残せず、逆に `MaybeFreed` を無条件に consumed とみなすと `Result` payload の parameter return と衝突して二重消費になる。

## 影響

`selfhost_req` が self-host 実装前の focused regression gate として使えない。さらに、temporary owner leak を doctest 側で回避すると、copy aggregate 内の owner leaf と `Result` payload owner return の相関を Resource IR が証明できない問題を隠してしまう。

## 修正方針

- copy state-only temporary scope は、型全体ではなく `owner_leaf_projections_mapped` で抽出した owner leaf 型を見て、nested `str` owner leaf を持つ copy aggregate にも scope を挿入する。
- owner summary の consumed 判定は、`MaybeFreed(storage=parameter)` を consumed 候補として扱う。ただし同じ summary/path でその parameter source が返却されている場合は consumed に入れない。
- この相関により、StringBuilder wrapper の「入力 owner は返却されず内部で別 owner へ詰め替えられる」経路を消費として記録しつつ、fs/stdio の「Result::Ok payload が parameter return として返る」経路の二重消費は防ぐ。
- 個別 stdlib 関数名の許可ではなく、Resource IR の owner state と returned source summary から汎用的に証明する。

## 関連 doc

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md): Stage 6 Resource IR owner summary / owner carrier proof の一部として扱う。

## 修正結果

- `copy_state_only_temporary_needs_resource_scope` を nested owner leaf 対応にした。
- `consumed_owner_parameters` で returned source を除外したまま `MaybeFreed(storage=parameter)` を consumed として扱うようにした。
- `resource_ir_owner_check_accepts_string_builder_build_wrapper_str_observer` を追加し、StringBuilder wrapper、`sb_build`、`len` observer の経路を Resource IR gate と full compile の両方で固定した。
- `ISS-20260513T151511232Z-RESOURCE-OWNER-ALIAS-RESOLUTION-CAN--CB5B7B73` で守っていた fs/stdio scratch cleanup regression が再破壊されないことも確認した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_string_builder_build_wrapper_str_observer -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir string_builder -- --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_accepts_stdio_string_temporaries -- --exact --nocapture`
- `cargo test -p nepl-core --test resource_ir resource_ir_compiler_accepts_vec_get_copy_str_option_return -- --exact --nocapture`
- `cargo check -p nepl-core`
- `trunk build`
- `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/agent1-selfhost-req-stringbuilder-owner.json -j 1 --dist web/dist --assert-io`: total=6, passed=6
