#!/usr/bin/env node
"use strict";

const { execFileSync, spawnSync } = require("child_process");

const tracked = execFileSync("git", ["ls-files", "--", "editors/zed/target"], {
    encoding: "utf8",
})
    .split(/\r?\n/)
    .filter(Boolean);

if (tracked.length > 0) {
    console.error("editors/zed/target must not contain tracked build artifacts.");
    for (const file of tracked.slice(0, 20)) {
        console.error(`- ${file}`);
    }
    if (tracked.length > 20) {
        console.error(`... ${tracked.length - 20} more tracked artifact(s)`);
    }
    process.exit(1);
}

const ignored = spawnSync(
    "git",
    ["check-ignore", "-q", "--", "editors/zed/target/neplg2-zed-build-output"],
    { stdio: "ignore" },
);

if (ignored.status !== 0) {
    console.error("editors/zed/target must be ignored as generated Cargo output.");
    process.exit(1);
}

console.log("zed extension target artifacts are untracked and ignored");
