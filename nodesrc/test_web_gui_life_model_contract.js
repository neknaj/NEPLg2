#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const lifeSource = fs.readFileSync(path.join(repoRoot, "examples", "gui_life.nepl"), "utf8");

function structBody(source, name) {
    const lines = source.split(/\r?\n/);
    const header = `struct ${name}:`;
    const start = lines.findIndex((line) => line.trim() === header);
    assert.notEqual(start, -1, `${name} struct must exist`);
    const body = [];
    for (let i = start + 1; i < lines.length; i += 1) {
        const line = lines[i];
        if (line.trim() === "") {
            continue;
        }
        if (!line.startsWith("    ")) {
            break;
        }
        body.push(line.trim());
    }
    return body.join("\n");
}

function assertLifeModelHasOnlyScalarState(source) {
    const body = structBody(source, "LifeModel");
    assert.match(
        body,
        /generation\s+%i32[\s\S]*animate\s+%bool[\s\S]*cell_size\s+%i32/,
        "LifeModel must keep generation, animate, and cell_size scalar fields",
    );
    assert.doesNotMatch(
        body,
        /%BitSet|\bcells\b|\bboard\b/,
        "LifeModel must not store the BitSet board owner inside a user-defined struct",
    );
}

assert.match(
    lifeSource,
    /#import\s+"alloc\/collections\/bitset"\s+as\s+bitset/,
    "Life example must store the board in a real NEPL BitSet, not in TypeScript simulation",
);

assertLifeModelHasOnlyScalarState(lifeSource);

assert.throws(
    () => assertLifeModelHasOnlyScalarState(lifeSource.replace("cell_size %i32", "cell_size %i32\n    cells %BitSet")),
    /must not store the BitSet board owner/,
    "Life source policy must reject a BitSet owner added to LifeModel",
);

assert.match(
    lifeSource,
    /enum\s+LifeCellState:[\s\S]*Dead[\s\S]*Alive[\s\S]*fn\s+life_cell_next_state\s+%fn LifeCellState fn i32 LifeCellState/,
    "Life cell transition must be expressed as a typed enum rule",
);

assert.match(
    lifeSource,
    /fn\s+life_board_neighbor_count\s+%impure fn &BitSet impure fn i32 impure fn i32 impure fn i32 Result i32 GuiError/,
    "Life board update must compute neighbour counts from the BitSet board",
);

assert.match(
    lifeSource,
    /fn\s+life_board_next_generation\s+%impure fn &BitSet impure fn i32 Result BitSet GuiError/,
    "Life Next and animation must build a new board generation",
);

assert.match(
    lifeSource,
    /gui_web_stdout_animation_timer[\s\S]*gui_web_event_timer[\s\S]*timer_event_timer_id[\s\S]*should_tick[\s\S]*life_board_next_generation[\s\S]*set model life_model_next_generation model/,
    "Life animation must use the GUI timer event path and advance the model generation",
);

assert.doesNotMatch(
    lifeSource,
    /struct\s+LifeBoard:|board\s+%BitSet|life_present_glider_phase|life_present_patterns|life_animation_tick|life_phase|life_wrap_positive/,
    "Life example must not regress to owner-backed aggregate board structs or hardcoded pattern rendering",
);

process.stdout.write(JSON.stringify({
    ok: true,
    checks: [
        "Life board storage is a real NEPL BitSet owner",
        "Life next generation uses typed cell state and neighbour counting",
        "Life animation uses GuiEvent::Timer instead of timeout fallback ticks",
    ],
}, null, 2) + "\n");
