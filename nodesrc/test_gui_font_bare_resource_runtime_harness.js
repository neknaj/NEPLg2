#!/usr/bin/env node
"use strict";
const assert = require("node:assert/strict");
const path = require("node:path");
const { runSingle } = require("./run_test");
const RESOURCE_PATH = "fonts/RuntimeFixture.ttf";
const HANDLE = 37;
const BYTES = Uint8Array.from([0, 1, 0, 0, 0x52, 0x55, 0x4e, 0x54]);
const SOURCE = String.raw`#entry main
#indent 4
#target std
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "platforms/gui/bare/font_resource_provider" as *
#import "std/gui" as *
fn embedded %fn GuiFontResourceSource bool \source:
    match source:
        GuiFontResourceSource::EmbeddedBlob:
            true
        _:
            false
fn main %impure fn void i32 \void:
    let path %GuiFontResourcePath unwrap_ok gui_font_resource_path_result "fonts/RuntimeFixture.ttf"
    let request %GuiFontResourceRequest unwrap_ok gui_font_resource_request path none none GuiFontDecodePolicy::SfntOnly
    match gui_bare_font_resource_request_bytes &request:
        Result::Err _:
            1
        Result::Ok resource:
            let valid %bool and embedded gui_font_resource_bytes_source &resource eq gui_font_resource_bytes_len &resource 8
            gui_font_resource_bytes_free resource
            if valid:
                then 0
                else 2
`;
function host() {
    const backing = Uint8Array.from(BYTES); const calls = []; let snapshot = null; let closed = false;
    const memory = context => new Uint8Array(context.getMemory().buffer);
    return {
        importsFactory(context) { return { nepl_gui_bare: {
            font_resource_open(ptr, len, policy) { calls.push("open"); assert.equal(Buffer.from(memory(context).subarray(ptr, ptr + len)).toString("utf8"), RESOURCE_PATH); assert.equal(policy, 1); snapshot = Uint8Array.from(backing); backing.fill(0xa5); return HANDLE; },
            font_resource_byte_len(handle) { calls.push("byte_len"); assert.equal(handle, HANDLE); return snapshot.length; },
            font_resource_read_bytes(handle, ptr, len) { calls.push("read"); assert.equal(handle, HANDLE); assert.equal(len, snapshot.length); assert.notDeepEqual(snapshot, backing); memory(context).set(snapshot, ptr); return len; },
            font_resource_close(handle) { calls.push("close"); assert.equal(handle, HANDLE); assert.equal(closed, false); closed = true; return 0; },
        } }; },
        verify(result) { assert.equal(result.ok, true, result.error); assert.equal(result.exit_code, 0); assert.deepEqual(calls, ["open", "byte_len", "read", "close"]); assert.equal(closed, true); },
    };
}
async function run() { const fake = host(); const result = await runSingle({ id: "gui-font-bare-resource/runtime-success", source: SOURCE, file: path.resolve(__dirname, "..", "tests", "gui_font_bare_resource_runtime_harness.nepl"), distHint: path.resolve(__dirname, "..", "web", "dist"), forceStdlibVfs: true, runtimeImportsFactory: fake.importsFactory }); fake.verify(result); return { ok: true }; }
if (require.main === module) run().then(value => process.stdout.write(JSON.stringify(value) + "\n")).catch(error => { console.error(error.stack || error); process.exit(1); });
module.exports = { run };
