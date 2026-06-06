#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const readme = fs.readFileSync(path.join(repoRoot, "README.md"), "utf8").replace(/\r\n/g, "\n");

function mustInclude(needle) {
    assert.ok(readme.includes(needle), `README must include current NEPLg2.1 fact: ${needle}`);
}

mustInclude("現在の NEPL は **NEPLg2.1** です。");
mustInclude("NEPLg3 は完全に検討段階で未着手です。");
mustInclude("NEPLg3 self-host は未着手です。");
mustInclude("#import \"std/stdio\" as *");
mustInclude("fn main %impure fn void unit \\void:");
mustInclude("let label %str grade score");
mustInclude("式指向");
mustInclude("前置記法");
mustInclude("%T expr");
mustInclude("型式も前置");
mustInclude("部分適用を導入しません");
mustInclude("0 引数関数の marker は `void`");
mustInclude("呼び出し側の explicit generic postfix を使いません");
mustInclude("nepl-language/");
mustInclude("nepl-lsp/");
mustInclude("nepl-gui-native/");
mustInclude("stdlib/neplg2/");
mustInclude("doc/neplg2/self_host_neplg21_compiler_design.md");
mustInclude("doc/neplg2/gui_standard_library_spec.md");

assert.doesNotMatch(
    readme,
    /use\s+core::/,
    "README NEPL sample must use #import, not the old use core:: syntax",
);

assert.doesNotMatch(
    readme,
    /現(?:在|行)[^\n]{0,40}NEPLg2\.0/,
    "README must not describe NEPLg2.0 as the current implementation",
);

assert.doesNotMatch(
    readme,
    /現在の\s*NEPL\s*は[^\n。]*NEPLg3/,
    "README must never say the current NEPL is NEPLg3, even on a line that also mentions draft status",
);

assert.doesNotMatch(
    readme,
    /(?:現在進めている self-host|正規の設計入口)[^\n]*stdlib\/neplg3\/|stdlib\/neplg3\/[^\n]*(?:現在進めている self-host|正規の設計入口)/,
    "README must not route active self-host work to stdlib/neplg3",
);

const activeNeplg3Lines = readme
    .split("\n")
    .filter((line) => line.includes("NEPLg3"))
    .filter((line) => /現行|現在|実装済み|進行中|self-host 実装|正規の設計入口/.test(line))
    .filter((line) => !/未着手|検討|扱いません|ではありません|ではない|authority ではありません/.test(line));
assert.deepEqual(
    activeNeplg3Lines,
    [],
    `README must not present NEPLg3 as current or active implementation:\n${activeNeplg3Lines.join("\n")}`,
);

const activeStdlibNeplg3Lines = readme
    .split("\n")
    .filter((line) => line.includes("stdlib/neplg3/"))
    .filter((line) => /現行|現在|実装済み|進行中|self-host|セルフホスト/.test(line))
    .filter((line) => !/未着手|検討|扱いません|ではありません|ではない|authority ではありません/.test(line));
assert.deepEqual(
    activeStdlibNeplg3Lines,
    [],
    `README must not describe stdlib/neplg3 as the active self-host source tree:\n${activeStdlibNeplg3Lines.join("\n")}`,
);

console.log("README current NEPLg2.1 reality contract passed");
