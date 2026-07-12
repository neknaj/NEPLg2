#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const privateHelper = "selfhost_memo_call_backend_private_cache_resource_ir_operation_kind_inventory_runtime_stage0";

const source = `#entry main
#indent 4
#target std
#import "neplg2/core/codegen/memo_call_backend_private_cache_proof_gate" as gate

fn main %fn void i32 \\void:
    gate::${privateHelper}
    0
`;

async function main() {
    const distDir = path.resolve(__dirname, "..", "web", "dist");
    const { api } = await loadCompilerFromDist(distDir);
    try {
        compileWithLocalStdlib(api, { source });
    } catch (error) {
        const message = String(error?.message || error);
        if (message.includes("resolve.identifier.undefined") && message.includes(privateHelper)) {
            console.log("selfhost memo_call merged inventory helper remains private");
            return;
        }
        throw error;
    }
    throw new Error(`merged inventory private helper unexpectedly compiled: ${privateHelper}`);
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
