#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const paintSource = fs.readFileSync(path.join(repoRoot, "examples", "gui_paint.nepl"), "utf8");

assert.match(
    paintSource,
    /struct\s+PaintCell:[\s\S]*slot\s+%i32[\s\S]*color\s+%PaintColor/,
    "PaintCell must keep a stroke cell and its selected color together",
);

assert.match(
    paintSource,
    /struct\s+PaintModel:[\s\S]*slot0\s+%Option PaintCell[\s\S]*slot1\s+%Option PaintCell[\s\S]*slot2\s+%Option PaintCell[\s\S]*color\s+%PaintColor[\s\S]*count\s+%i32/,
    "PaintModel must represent absent stroke cells with Option and keep an explicit count",
);

assert.match(
    paintSource,
    /enum\s+PaintUpdateErrorKind:[\s\S]*CapacityFull[\s\S]*PointerOutsideCanvas/,
    "Paint update failures must be represented with an enum",
);

assert.match(
    paintSource,
    /struct\s+PaintUpdateError:[\s\S]*model\s+%PaintModel[\s\S]*kind\s+%PaintUpdateErrorKind/,
    "PaintUpdateError must preserve the model owner on update failure",
);

assert.match(
    paintSource,
    /fn\s+paint_set_cell_result\s+%fn PaintModel fn i32 fn i32 Result PaintModel PaintUpdateError/,
    "paint_set_cell_result must return Result instead of silently overwriting state",
);

assert.match(
    paintSource,
    /fn\s+paint_update_event_result\s+%fn PaintModel fn GuiWebEvent Result PaintModel PaintUpdateError/,
    "paint_update_event_result must expose update errors to the event loop",
);

assert.match(
    paintSource,
    /match\s+slot:[\s\S]*Option::Some\s+cell:[\s\S]*paint_cell_color\s+&cell[\s\S]*Option::None:[\s\S]*Result::Ok\s+unit/,
    "paint_present_slot must draw typed cells and skip Option::None explicitly",
);

assert.doesNotMatch(
    paintSource,
    /paint_model_new\s+255|eq\s+slot\s+255|next_slot|paint_next_slot|slot[012]\s+%i32|paint_set_cell\s+%fn PaintModel fn i32 fn i32 PaintModel|paint_update_event\s+%fn PaintModel fn GuiWebEvent PaintModel/,
    "paint model must not use sentinel stroke slots or non-Result update helpers",
);

process.stdout.write(JSON.stringify({
    ok: true,
    checks: [
        "Paint model uses Option PaintCell storage instead of numeric sentinel slots",
        "Paint update failures use Result with enum error kind and owner recovery",
        "Paint rendering uses the stored stroke color instead of current palette state",
    ],
}, null, 2) + "\n");
