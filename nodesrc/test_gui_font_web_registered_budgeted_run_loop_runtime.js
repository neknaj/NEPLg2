#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");

async function run() {
    let requested = null;
    let requestCount = 0;
    const events = [
        { kind: 1, windowId: 3 },
        { kind: 6, windowId: 4, timerId: 7, tick: 1 },
        { kind: 6, windowId: 3, timerId: 8, tick: 1 },
        { kind: 6, windowId: 3, timerId: 7, tick: -1 },
        { kind: 6, windowId: 3, timerId: 7, tick: 1 },
    ];
    let current = null;
    async function runContract(name, expression, requestStatus, expected) {
        const result = await runSingle({
        id: "gui-font-web-registered-budgeted-run-loop/runtime",
        source: `#entry main
#indent 4
#target std
#import "platforms/gui/web/font_registered_budgeted_run_loop_test" as * with tests

fn main %impure fn void i32 \\void:
    ${expression}
`,
        file: path.resolve(__dirname, "..", "tests", "gui_font_web_registered_budgeted_run_loop.nepl"),
        distHint: path.resolve(__dirname, "..", "web", "dist"),
        forceStdlibVfs: true,
        runtimeImportsFactory() {
            return { nepl_gui_web: {
                request_timer(windowId, timerId, intervalMs, repeating) { requestCount += 1; requested = { windowId, timerId, intervalMs, repeating }; return requestStatus; },
                poll_event_kind() { current = events.shift(); return current.kind; },
                last_event_window_id() { return current.windowId; },
                last_event_action_id() { return 1; },
                last_event_timer_id() { return current.timerId || 0; },
                last_event_timer_tick() { return current.tick == null ? 0 : current.tick; },
            } };
        },
        });
        assert.equal(result.ok, true, `${name}: ${result.error && result.error.slice(-12000)}`);
        assert.equal(result.return_value, expected, name);
        return result.return_value;
    }
    const ready = await runContract("ready", "gui_font_web_registered_budgeted_run_loop_test_ready_contract unit", 0, 6);
    const failure = await runContract("failure", "gui_font_web_registered_budgeted_run_loop_test_schedule_failure_contract 1", -1, 7);
    const requestsAfterFailure = requestCount;
    const zero = await runContract("zero", "gui_font_web_registered_budgeted_run_loop_test_schedule_failure_contract 0", 0, 7);
    assert.equal(requestCount, requestsAfterFailure, "zero delay must not reach host");
    const wake = await runContract("wake", "gui_font_web_registered_budgeted_run_loop_test_contract unit", 0, 9);
    assert.deepEqual(requested, { windowId: 3, timerId: 7, intervalMs: 1, repeating: 0 });
    return { ok: true, ready, failure, zero, wake, requested };
}

if (require.main === module) run().then((value) => process.stdout.write(`${JSON.stringify(value)}\n`)).catch((error) => { console.error(error.stack || error); process.exit(1); });

module.exports = { run };
