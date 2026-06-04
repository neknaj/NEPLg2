#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), "utf8").replace(/\r\n/g, "\n");
}

const moveStateFacadePath = "stdlib/neplg2/core/resource/move_state.nepl";
const initCellPath = "stdlib/neplg2/core/resource/init/cell.nepl";
const proofFactModel = read("stdlib/neplg2/core/proof/fact/model.nepl");
const proofCellConsumers = [
    "stdlib/neplg2/core/proof/api/resource.nepl",
    "stdlib/neplg2/core/proof/evidence.nepl",
    "stdlib/neplg2/core/proof/fact/model.nepl",
    "stdlib/neplg2/core/proof/obligation.nepl",
    "stdlib/neplg2/core/proof/refutation.nepl",
    "stdlib/neplg2/core/proof/solver/resource.nepl",
];
const moveStateFacade = read(moveStateFacadePath);
const initCell = read(initCellPath);

assert.match(
    moveStateFacade,
    /^pub #import "\.\/init\/cell" as \*$/m,
    "legacy move_state facade must re-export resource/init/cell",
);
assert.doesNotMatch(
    moveStateFacade,
    /^(?:pub\s+)?(?:struct|enum|fn|impl)\s+/m,
    "legacy move_state facade must not own Resource cell implementation",
);
assert.match(initCell, /pub enum SelfhostResourceCellState:/, "Resource cell state must live in init/cell");
assert.match(initCell, /pub enum SelfhostResourceCellEventKind:/, "Resource cell event kind must live in init/cell");
assert.match(
    initCell,
    /match\s+left:/,
    "Resource cell equality must stay as typed enum matching, not numeric tags",
);
assert.doesNotMatch(
    initCell,
    /"[A-Za-z0-9_.:-]+"/,
    "Resource cell model must not depend on string codes or module names",
);
assert.doesNotMatch(
    initCell,
    /#import "neplg2\/core\/proof"/,
    "Resource init cell model must remain a fact/obligation payload model, not a proof engine",
);
assert.doesNotMatch(
    initCell,
    /#import "neplg2\/core\/resource\/move_state"/,
    "Resource init cell implementation must not import the legacy facade",
);
assert.match(
    proofFactModel,
    /#import "neplg2\/core\/resource\/init\/cell" as \*/,
    "proof fact model must import Resource cell payloads from the final init/cell path",
);
assert.doesNotMatch(
    proofFactModel,
    /#import "neplg2\/core\/resource\/move_state" as \*/,
    "proof fact model must not depend on the legacy move_state facade",
);
for (const rel of proofCellConsumers) {
    const source = read(rel);
    assert.match(
        source,
        /#import "neplg2\/core\/resource\/init\/cell" as \*/,
        `${rel} must import Resource cell payloads from the final init/cell path`,
    );
    assert.doesNotMatch(
        source,
        /#import "neplg2\/core\/resource\/move_state" as \*/,
        `${rel} must not depend on the legacy move_state facade`,
    );
}

console.log("selfhost resource tree split contract passed");
