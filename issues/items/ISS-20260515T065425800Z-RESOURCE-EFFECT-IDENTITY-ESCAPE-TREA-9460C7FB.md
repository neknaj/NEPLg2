---
id: ISS-20260515T065425800Z-RESOURCE-EFFECT-IDENTITY-ESCAPE-TREA-9460C7FB
title: "Resource effect identity escape treats owner-protected returns as raw pointer leaks"
area: core
status: fixed
resolved: true
priority: P0
type: bug
created: 2026-05-15
updated: 2026-05-15
target: "nepl-core/src/resource/effect_check.rs, nepl-core/src/resource/effect_return_escape.rs, nepl-core/src/resource/place_utils.rs, nepl-core/src/resource/mod.rs, nepl-core/tests/resource_ir.rs, nodesrc/test_resource_checker_responsibility.js"
---

# ISS-20260515T065425800Z-RESOURCE-EFFECT-IDENTITY-ESCAPE-TREA-9460C7FB: Resource effect identity escape treats owner-protected returns as raw pointer leaks

## 概要

After the checked MemPtr provenance work, GitHub Actions run 25903807106 reports resource.raw.identity_escape for pure safe APIs such as byte_builder_finish, StringBuilder append/build helpers, and ansi_text_style_code. The returned values are typed owner carriers or language str values, not public raw addresses; the current effect check only asks whether the returned aggregate is in the raw identity group.

## 対象

- `nepl-core/src/resource/effect_check.rs, nepl-core/src/resource/place_utils.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- GitHub Actions run `25903807106` の examples artifact で、`examples/counter.nepl` / `examples/counter2.nepl` / `examples/fib.nepl` が `byte_builder_finish` / `sb_build_result` の `resource.raw.identity_escape` により compile failure になった。
- 同じ run で `examples/stdio.nepl` が `ansi_text_style_code` の `str` 返却を internal Alloc identity escape として拒否した。
- 調査すると、`ResourceOp::Construct` が入力 identity を aggregate 全体へ粗く merge し、`Result::Ok(ByteBuf)` の payload identity が `Ok` payload 直下だけでなく、戻り値直下の field projection としても見えていた。
- さらに `str` の内部 raw representation へ降りる projection を public raw address leaf と同一視していたため、言語レベルの owned `str` 返却まで raw pointer leak と誤判定していた。

## 問題

After the checked MemPtr provenance work, GitHub Actions run 25903807106 reports resource.raw.identity_escape for pure safe APIs such as byte_builder_finish, StringBuilder append/build helpers, and ansi_text_style_code. The returned values are typed owner carriers or language str values, not public raw addresses; the current effect check only asks whether the returned aggregate is in the raw identity group.

## 影響

Safe stdlib APIs that allocate and return owned values are rejected, while weakening the diagnostic would hide real MemPtr/i32 raw address escapes. Static-check Stage 6 cannot progress unless the compiler distinguishes unprotected raw address leaves from owner-protected storage identities.

## 修正方針

Make RawAddressEscapeFromInternalAlloc projection-aware and type-aware: report only i32/MemPtr raw identity leaves that are not under RegionToken/owned storage protection, keep summaries intact for provenance propagation, and add regressions for ByteBuilder/StringBuilder/string-style owner returns plus forged raw pointer returns.

## 検証

Run focused Resource IR effect tests, cargo check -p nepl-core, focused examples/stdio/counter doctests, nodesrc issue validation, and trunk build.

## 修正内容

- `ResourceOp::Construct` の raw identity propagation を、aggregate 全体への粗い merge ではなく field / enum payload projection へ限定した。
- `RawAddressEscapeFromInternalAlloc` の返却判定を `effect_return_escape.rs` に分離し、projection-aware / type-aware にした。
- `i32` / `MemPtr` として public surface に出る raw identity は引き続き `resource.raw.identity_escape` として報告する。
- `RegionToken` 配下と `str` 配下の raw identity projection は owner / language value の内部表現として保護されているため、public raw pointer leak として扱わない。
- `nodesrc/test_resource_checker_responsibility.js` に新規 module を監視対象として追加し、`place_utils.rs` の肥大化を避けた。

## 回帰テスト

- `resource_ir_effect_check_accepts_owned_str_return_identity`: `concat_result` で作った owned `str` を pure 関数から返しても raw identity escape にしない。
- `resource_ir_effect_check_accepts_byte_builder_finish_owner_return`: `ByteBuilder -> ByteBuf` owner return を raw pointer leak と誤診断しない。
- `resource_ir_effect_check_accepts_string_builder_build_str_return`: `StringBuilder -> str` build path を raw pointer leak と誤診断しない。
- `resource_ir_effect_check_rejects_mem_ptr_return_identity_escape`: `MemPtr` として internal allocation identity が返る場合は引き続き拒否する。

## 2026-05-15 修正確認

- `cargo test -p nepl-core --test resource_ir resource_ir_effect_check -- --nocapture`: 29/29 passed。
- `cargo test -p nepl-core --lib resource_effect_gate -- --nocapture`: 9/9 passed。
- `cargo check -p nepl-core`: passed。
- `node nodesrc/test_resource_checker_responsibility.js`: passed。
- `node nodesrc/test_static_check_boundary_responsibility.js`: passed。
- `trunk build`: passed。
- `node nodesrc/tests.js -i examples/counter.nepl -i examples/counter2.nepl -i examples/fib.nepl -i examples/stdio.nepl --no-tree -o tmp/agent1-protected-owner-identity-examples.json -j 1 --dist web/dist`: 5/5 passed。
