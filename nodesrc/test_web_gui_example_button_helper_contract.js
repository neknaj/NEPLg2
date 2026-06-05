#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function sliceBetween(source, startPattern, endPattern) {
    const start = source.search(startPattern);
    assert.notEqual(start, -1, `missing start pattern ${startPattern}`);
    const rest = source.slice(start);
    const end = rest.search(endPattern);
    assert.notEqual(end, -1, `missing end pattern ${endPattern}`);
    return rest.slice(0, end);
}

function assertSharedButtonCall(source, label) {
    assert.match(source, /gui_web_button_config/, `${label} must construct GuiWebButtonConfig`);
    assert.match(source, /gui_web_stdout_button/, `${label} must use gui_web_stdout_button`);
}

function assertNoLocalButtonEmission(functionSource, label) {
    assert.doesNotMatch(functionSource, /gui_web_stdout_fill_rect[\s\S]*gui_web_stdout_text_run[\s\S]*gui_web_stdout_action_rect/, `${label} must not hand-roll fill/text/action emission`);
}

function runWebGuiExampleButtonHelperContractRegression() {
    const stdoutSource = readRepoFile("stdlib", "platforms", "gui", "web", "stdout_protocol.nepl");
    const counterSource = readRepoFile("examples", "gui_counter.nepl");
    const lifeSource = readRepoFile("examples", "gui_life.nepl");
    const mandelbrotSource = readRepoFile("examples", "gui_mandelbrot.nepl");
    const calculatorSource = readRepoFile("examples", "gui_calculator.nepl");
    const scientificSource = readRepoFile("examples", "gui_scientific_calculator.nepl");
    const paintSource = readRepoFile("examples", "gui_paint.nepl");
    const breakoutSource = readRepoFile("examples", "gui_breakout.nepl");

    assert.match(stdoutSource, /pub struct GuiWebButtonConfig/);
    assert.match(stdoutSource, /pub fn gui_web_button_config /);
    assert.match(stdoutSource, /pub fn gui_web_stdout_button /);
    assert.match(stdoutSource, /gui_web_stdout_fill_rect[\s\S]*gui_web_stdout_text_run[\s\S]*gui_web_stdout_action_rect/);
    assert.doesNotMatch(stdoutSource, /CanvasRenderingContext2D|HTMLCanvasElement|document\.|window\./);

    for (const [label, source] of [
        ["counter", counterSource],
        ["life", lifeSource],
        ["mandelbrot", mandelbrotSource],
        ["calculator", calculatorSource],
        ["scientific calculator", scientificSource],
        ["paint", paintSource],
        ["breakout", breakoutSource],
    ]) {
        assertSharedButtonCall(source, label);
    }

    assertNoLocalButtonEmission(
        sliceBetween(calculatorSource, /fn calculator_present_button\b/, /fn calculator_present_buttons\b/),
        "calculator_present_button",
    );
    assertNoLocalButtonEmission(
        sliceBetween(scientificSource, /fn sci_present_button\b/, /fn sci_present_buttons\b/),
        "sci_present_button",
    );
    assertNoLocalButtonEmission(
        sliceBetween(lifeSource, /fn life_present_button\b/, /fn life_present_status\b/),
        "life_present_button",
    );
    assertNoLocalButtonEmission(
        sliceBetween(mandelbrotSource, /fn mandelbrot_present_button\b/, /fn mandelbrot_present_status\b/),
        "mandelbrot_present_button",
    );
    assertNoLocalButtonEmission(
        sliceBetween(breakoutSource, /fn breakout_present_control\b/, /fn breakout_present_controls\b/),
        "breakout_present_control",
    );

    assert.doesNotMatch(counterSource, /fn counter_present_action_rect\b/);
    assert.doesNotMatch(counterSource, /fn counter_present_button_label\b/);
    assert.match(paintSource, /let clear_config %GuiWebButtonConfig/);
    assert.match(paintSource, /gui_web_stdout_button clear_config/);

    return {
        ok: true,
        checks: [
            "Web stdout protocol exposes a typed button helper",
            "GUI examples use shared button presentation instead of hand-rolled fill/text/action emission",
            "Button helper keeps Canvas and DOM details out of NEPL stdout protocol",
        ],
    };
}

if (require.main === module) {
    try {
        const result = runWebGuiExampleButtonHelperContractRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runWebGuiExampleButtonHelperContractRegression,
};
