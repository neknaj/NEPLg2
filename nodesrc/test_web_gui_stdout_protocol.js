#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { pathToFileURL } = require("node:url");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

async function loadProtocolModule() {
    const modulePath = path.resolve(__dirname, "..", "web", "dist_ts", "gui-preview", "stdout-protocol.js");
    return import(pathToFileURL(modulePath).href);
}

async function runWebGuiStdoutProtocolRegression() {
    const protocol = await loadProtocolModule();
    const parser = new protocol.GuiWebStdoutProtocolParser();

    let events = parser.pushText([
        "normal output\n",
        "NEPLG2_GUI_FRAME_BEGIN 7 64 32 NEPL stdout frame\n",
        "NEPLG2_GUI_FILL_RECT 1 2 3 4 5 6 7 255\n",
        "NEPLG2_GUI_TEXT_RUN 8 9 10 center 11 12 13 255 Count 1\n",
        "NEPLG2_GUI_ACTION_RECT 18 20 30 40 9\n",
        "NEPLG2_GUI_FRAME_END\n",
        "after frame\n",
    ].join(""));
    assert.equal(events.length, 3);
    assert.equal(events[0].kind, "text");
    assert.equal(events[0].text, "normal output\n");
    assert.equal(events[1].kind, "frame");
    assert.equal(events[1].frame.windowId, 7);
    assert.equal(events[1].frame.title, "NEPL stdout frame");
    assert.equal(events[1].frame.commands.length, 2);
    assert.equal(events[1].frame.inputTargets.length, 1);
    assert.equal(events[1].frame.commands[0].kind, "fill-rect");
    assert.equal(events[1].frame.commands[1].kind, "text-run");
    assert.equal(events[1].frame.commands[1].text, "Count 1");
    assert.equal(events[1].frame.inputTargets[0].kind, "action-rect");
    assert.equal(events[1].frame.inputTargets[0].actionId, 9);
    assert.equal(events[2].kind, "text");
    assert.equal(events[2].text, "after frame\n");

    parser.reset();
    events = parser.pushText("NEPLG2_GUI_SESSION_STATE counter:1\nNEPLG2_GUI_ANIMATE_MS 16\n");
    assert.equal(events.length, 2);
    assert.equal(events[0].kind, "session-state");
    assert.equal(events[0].state, "counter:1");
    assert.equal(events[1].kind, "animation-timer");
    assert.equal(events[1].intervalMs, 16);

    parser.reset();
    events = [
        ...parser.pushText("NEPLG2_GUI_FRAME_BEGIN 2 10"),
        ...parser.pushText(" 12 Chunked\nNEPLG2_GUI_FILL_RECT 0 0 1 1 1 2 3 4\n"),
        ...parser.pushText("NEPLG2_GUI_FRAME_END\n"),
    ];
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, "frame");
    assert.equal(events[0].frame.width, 10);
    assert.equal(events[0].frame.height, 12);

    parser.reset();
    events = parser.pushText("NEPLG2_GUI_FILL_RECT 0 0 1 1 1 2 3 4\n");
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, "error");
    assert.equal(events[0].error.kind, "invalid-frame-state");

    parser.reset();
    events = parser.pushText("NEPLG2_GUI_FRAME_BEGIN 1 10 10 Bad\nNEPLG2_GUI_FILL_RECT 0 0 1 1 300 2 3 4\n");
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, "error");
    assert.equal(events[0].error.kind, "invalid-color");
    assert.equal(events[0].error.path, "$.color.red");
    events = parser.pushText("NEPLG2_GUI_FRAME_END\n");
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, "error");
    assert.equal(events[0].error.kind, "invalid-frame-state");

    parser.reset();
    events = parser.pushText("NEPLG2_GUI_FRAME_BEGIN 1 10 10 Bad\nplain text inside frame\nNEPLG2_GUI_FRAME_END\n");
    assert.equal(events.length, 2);
    assert.equal(events[0].kind, "error");
    assert.equal(events[0].error.kind, "unsupported-protocol-line");
    assert.equal(events[1].kind, "error");
    assert.equal(events[1].error.kind, "invalid-frame-state");

    parser.reset();
    events = parser.pushText("NEPLG2_GUI_FRAME_BEGIN 1 10 10 Bad\nNEPLG2_GUI_ACTION_RECT 0 0 1 1 0\n");
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, "error");
    assert.equal(events[0].error.kind, "invalid-action-rect");
    assert.equal(events[0].error.path, "$.actionId");

    parser.reset();
    events = parser.pushText("NEPLG2_GUI_FRAME_BEGIN 1 10 10 Missing end\n");
    events.push(...parser.flush());
    assert.equal(events.length, 1);
    assert.equal(events[0].kind, "error");
    assert.equal(events[0].error.kind, "invalid-frame-state");

    const protocolSource = readRepoFile("web", "src", "gui-preview", "stdout-protocol.ts");
    const shellSource = readRepoFile("web", "src", "terminal", "shell.ts");
    const panelSource = readRepoFile("web", "src", "gui-preview", "panel.ts");
    assert.match(protocolSource, /GuiWebStdoutProtocolParser/);
    assert.match(protocolSource, /NEPLG2_GUI_FRAME_BEGIN/);
    assert.match(protocolSource, /NEPLG2_GUI_ACTION_RECT/);
    assert.match(protocolSource, /NEPLG2_GUI_SESSION_STATE/);
    assert.match(protocolSource, /GuiWebStdoutProtocolErrorKind/);
    assert.match(shellSource, /GuiWebStdoutProtocolParser/);
    assert.match(shellSource, /presentGuiWebRuntimeFrame/);
    assert.match(shellSource, /message\.fd === 1/, "GUI stdout protocol must only parse stdout fd=1");
    assert.match(panelSource, /renderGuiPreviewFrameToCanvas/);
    assert.match(panelSource, /GuiPreviewDebugSink/);
    assert.match(panelSource, /waiting-for-frame/);
    assert.doesNotMatch(panelSource, /waiting for host frame/);
    assert.doesNotMatch(panelSource, /metricsEl|gui-preview-metrics|host commands/);
    assert.doesNotMatch(panelSource, /createGuiPreviewScene|summarizeGuiPreviewScene|renderGuiPreviewSceneToCanvas/);
    for (const [name, source] of [
        ["stdout-protocol.ts", protocolSource],
        ["shell.ts", shellSource],
    ]) {
        assert.doesNotMatch(source, /createGuiPreviewScene/, `${name} must not simulate NEPL GUI examples`);
        assert.doesNotMatch(source, /JSON\.parse/, `${name} must use the typed line protocol instead of raw JSON parsing`);
    }
    assert.doesNotMatch(protocolSource, /\|\s*undefined/, "stdout-protocol.ts must not expose undefined in protocol DTOs");

    return {
        ok: true,
        checks: [
            "NEPL stdout GUI protocol decodes chunked frame output into typed command frames",
            "NEPL stdout GUI protocol decodes action hit targets without drawing simulation",
            "NEPL stdout GUI protocol decodes session state and animation timer events",
            "protocol errors are explicit discriminated error values",
            "Web shell presents NEPL-emitted frames without TS example simulation",
        ],
    };
}

if (require.main === module) {
    runWebGuiStdoutProtocolRegression()
        .then((result) => process.stdout.write(JSON.stringify(result, null, 2) + "\n"))
        .catch((error) => {
            console.error(error && error.stack ? error.stack : String(error));
            process.exit(1);
        });
}

module.exports = {
    runWebGuiStdoutProtocolRegression,
};
