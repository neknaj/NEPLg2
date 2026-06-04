#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function withoutComments(text) {
    return text
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//"))
        .join("\n");
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertMatch(text, pattern, message) {
    assert(pattern.test(text), `${message}: expected ${pattern}`);
}

function assertNoMatch(text, pattern, message) {
    assert(!pattern.test(text), `${message}: forbidden ${pattern}`);
}

const windowSource = read("stdlib/std/gui/window.nepl");
const windowImpl = withoutComments(windowSource);
const hostImpl = withoutComments(read("stdlib/std/gui/host.nepl"));
const runtimeImpl = withoutComments(read("stdlib/std/gui/runtime.nepl"));
const webInputImpl = withoutComments(read("stdlib/platforms/gui/web/input.nepl"));
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const guiWebInputTests = read("tests/stdlib/gui_web_input.n.md");

assertMatch(
    windowImpl,
    /\bstruct\s+GuiOpaqueIdProof:[\s\S]*raw\s+%i32/,
    "std/gui/window must keep opaque id construction proof private",
);
assertNoMatch(
    windowImpl,
    /\bpub\s+struct\s+GuiOpaqueIdProof\b/,
    "opaque id proof must not be public",
);

for (const typeName of ["WindowId", "SurfaceId", "FrameId"]) {
    assertMatch(
        windowImpl,
        new RegExp(`pub\\s+struct\\s+${typeName}:[\\s\\S]*raw\\s+%i32[\\s\\S]*proof\\s+%GuiOpaqueIdProof`),
        `${typeName} must require a private proof field in addition to the raw id`,
    );
}

for (const [name, typeName] of [
    ["window_id", "WindowId"],
    ["surface_id", "SurfaceId"],
    ["frame_id", "FrameId"],
]) {
    assertMatch(
        windowImpl,
        new RegExp(`fn\\s+${name}_unchecked\\s+%fn\\s+i32\\s+${typeName}`),
        `${name}_unchecked must exist only as a private module helper`,
    );
    assertNoMatch(
        windowImpl,
        new RegExp(`pub\\s+fn\\s+${name}_unchecked\\b`),
        `${name}_unchecked must not be public`,
    );
    assertMatch(
        windowImpl,
        new RegExp(`pub\\s+fn\\s+${name}_result\\s+%fn\\s+i32\\s+Result\\s+${typeName}\\s+GuiError[\\s\\S]*gt\\s+raw\\s+0[\\s\\S]*Result::Err\\s+GuiError::InvalidCommand`),
        `${name}_result must reject 0 and negative raw ids with a typed GUI error`,
    );
    assertMatch(
        windowImpl,
        new RegExp(`pub\\s+fn\\s+${name}\\s+%fn\\s+i32\\s+Result\\s+${typeName}\\s+GuiError[\\s\\S]*${name}_result\\s+raw`),
        `${name} compatibility helper must be checked and return Result`,
    );
}

assertMatch(
    hostImpl,
    /default_window\s+%Option\s+WindowId/,
    "GuiHost.default_window must model absence with Option WindowId",
);
assertMatch(
    hostImpl,
    /pub\s+fn\s+gui_host_headless\s+%fn\s+void\s+GuiHost[\s\S]*Option::None/,
    "headless GuiHost must not manufacture WindowId 0",
);
assertMatch(
    hostImpl,
    /pub\s+fn\s+gui_host_default_window\s+%fn\s+&GuiHost\s+Option\s+WindowId/,
    "GuiHost default window accessor must return Option WindowId",
);

assertMatch(
    runtimeImpl,
    /match\s+window_id_result\s+target:[\s\S]*Result::Ok\s+window:[\s\S]*GuiRuntimeCommand::RequestRedraw\s+window[\s\S]*Result::Err\s+_:[\s\S]*GuiError::InvalidCommand/,
    "runtime must validate redraw target ids before emitting host commands",
);
assertMatch(
    runtimeImpl,
    /match\s+window_id_result\s+target:[\s\S]*Result::Ok\s+window:[\s\S]*window_title_update\s+window\s+title[\s\S]*Result::Err\s+_:[\s\S]*GuiError::InvalidCommand/,
    "runtime must validate title target ids before emitting host commands",
);

assertMatch(
    webInputImpl,
    /#import\s+"std\/gui\/window"\s+as\s+\*/,
    "web input boundary must use std/gui WindowId contract",
);
assertMatch(
    webInputImpl,
    /pub\s+struct\s+GuiWebEvent:[\s\S]*window_id\s+%WindowId/,
    "GuiWebEvent must carry a typed WindowId",
);
assertMatch(
    webInputImpl,
    /pub\s+fn\s+gui_web_event_window_id\s+%fn\s+&GuiWebEvent\s+WindowId/,
    "GuiWebEvent window accessor must return WindowId",
);
assertMatch(
    webInputImpl,
    /let\s+raw_window\s+%i32\s+gui_web_last_event_window_id_raw[\s\S]*match\s+window_id_result\s+raw_window:[\s\S]*Result::Err\s+_:[\s\S]*Result::Err\s+GuiError::InvalidCommand/,
    "web input must validate raw host window ids at the host boundary",
);

assertMatch(guiStdTests, /is_invalid_window_id\s+0/, "std/gui doctests must cover WindowId 0 rejection");
assertMatch(guiStdTests, /is_invalid_surface_id\s+-1/, "std/gui doctests must cover negative SurfaceId rejection");
assertMatch(guiStdTests, /frame_id_roundtrip_ok\s+7/, "std/gui doctests must cover valid FrameId roundtrip");
assertMatch(guiStdTests, /is_none\s+gui_host_default_window\s+&headless/, "std/gui doctests must cover headless default window None");
assertMatch(
    guiStdTests,
    /neplg2:test\[compile_fail\][\s\S]*let\s+_id\s+%WindowId\s+WindowId\s+0/,
    "std/gui doctests must prove raw WindowId constructor is not the public contract",
);
assertMatch(
    guiWebInputTests,
    /let\s+host_window\s+%WindowId\s+unwrap_ok\s+window_id_result\s+3[\s\S]*GuiWebEvent\s+host_window\s+point/,
    "web input tests must construct GuiWebEvent with a checked WindowId",
);

console.log("stdlib GUI opaque id contract passed");
