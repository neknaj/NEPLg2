#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");
const monoPath = "stdlib/neplg2/core/mono/mono.nepl";
const mono = legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, monoPath), "utf8").replace(/\r\n/g, "\n"));

function topLevelBlock(src, kind, name) {
    const lines = src.split("\n");
    const decl = kind === "fn"
        ? new RegExp(`^(?:pub\\s+)?fn\\s+${name}\\s`)
        : new RegExp(`^(?:pub\\s+)?${kind}\\s+${name}`);
    const start = lines.findIndex((line) => decl.test(line));
    assert.notEqual(start, -1, `${kind} ${name} not found`);
    const topLevel = /^(?:pub\s+)?(?:fn|struct|enum|impl)\s+/;
    let end = lines.length;
    for (let i = start + 1; i < lines.length; i += 1) {
        if (topLevel.test(lines[i])) {
            end = i;
            break;
        }
    }
    return lines.slice(start, end).join("\n");
}

assert.match(mono, /#import "core\/option" as \*/, "mono instance absence must use Option");
assert.doesNotMatch(mono, /\bfn\s+selfhost_mono_instance_id_invalid\b/, "mono instance IDs must not expose an invalid constructor");
assert.doesNotMatch(mono, /\bfn\s+selfhost_mono_instance_id_is_valid\b/, "mono instance IDs must not rely on validity checks for absence");
assert.doesNotMatch(mono, /\bselfhost_mono_instance_id_new\s+-1\b/, "mono instance IDs must not construct -1 sentinels");
assert.match(mono, /\bpub\s+struct\s+SelfhostMonoInstanceRecord:/, "mono must model assigned cache records as a typed struct");
assert.match(mono, /#import "alloc\/collections\/vec" as v/, "mono cache storage must use the typed Vec facade");
assert.match(mono, /\bpub\s+enum\s+SelfhostMonoInstanceCacheInternError:/, "mono cache intern failures must use a typed enum");

const pending = topLevelBlock(mono, "fn", "selfhost_mono_instance_id_pending");
assert.match(pending, /Option<SelfhostMonoInstanceId>/, "pending state must return Option<SelfhostMonoInstanceId>");
assert.match(pending, /\bnone\b/, "pending state must be Option::None");
assert.doesNotMatch(pending, /\bSelfhostMonoInstanceId\b\s+-1\b|\bselfhost_mono_instance_id_new\s+-1\b/, "pending state must not use an invalid ID payload");

const assigned = topLevelBlock(mono, "fn", "selfhost_mono_instance_id_assigned");
assert.match(assigned, /Option<SelfhostMonoInstanceId>/, "assigned state must return Option<SelfhostMonoInstanceId>");
assert.match(assigned, /\bsome\s+instance_id\b/, "assigned state must wrap the stable ID in Some");

const stage0 = topLevelBlock(mono, "fn", "selfhost_mono_stage0");
assert.match(stage0, /\blet\s+pending\s+<Option<SelfhostMonoInstanceId>>\s+selfhost_mono_instance_id_pending\b/, "stage0 must exercise pending typed absence");
assert.match(stage0, /\bmatch\s+assigned:/, "stage0 must inspect assigned state through Option matching");
assert.match(stage0, /\bOption::Some\s+assigned_id:/, "stage0 must handle assigned Some payload");
assert.match(stage0, /\bOption::None:/, "stage0 must handle assigned None payload");
assert.match(stage0, /\bis_none\s+pending\b/, "stage0 must verify pending is None");
assert.match(stage0, /\bselfhost_mono_instance_record_new\s+key0\s+instance_id\b/, "stage0 must exercise typed mono instance records");
assert.match(stage0, /\bselfhost_mono_instance_record_matches_key\s+record\s+key1\b/, "stage0 must compare records through the typed key helper");

const record = topLevelBlock(mono, "struct", "SelfhostMonoInstanceRecord");
assert.match(record, /\bkey\s+<SelfhostMonoInstanceKey>/, "mono instance records must store the full typed key");
assert.match(record, /\binstance_id\s+<SelfhostMonoInstanceId>/, "mono instance records must store the assigned instance id");

const recordMatches = topLevelBlock(mono, "fn", "selfhost_mono_instance_record_matches_key");
assert.match(recordMatches, /\bselfhost_mono_instance_key_eq\s+record\.key\s+key\b/, "record lookup must use full key equality");
assert.doesNotMatch(recordMatches, /\bselfhost_mono_instance_key_seed\b/, "record lookup must not use seed equality as identity");

const cache = topLevelBlock(mono, "struct", "SelfhostMonoInstanceCache");
assert.match(cache, /\brecords\s+<Vec<SelfhostMonoInstanceRecord>>/, "mono cache storage must keep typed key/value records");
assert.doesNotMatch(cache, /Vec<SelfhostMonoInstanceKey>|Vec<SelfhostMonoInstanceId>/, "mono cache storage must not split keys and ids into parallel Vecs");

const internResult = topLevelBlock(mono, "struct", "SelfhostMonoInstanceCacheInternResult");
assert.match(internResult, /\bcache\s+<SelfhostMonoInstanceCache>/, "mono cache intern result must return the cache owner");
assert.match(internResult, /\binstance_id\s+<SelfhostMonoInstanceId>/, "mono cache intern result must return a typed instance id");

const internError = topLevelBlock(mono, "enum", "SelfhostMonoInstanceCacheInternError");
assert.match(internError, /\bInvalidKey\s+<SelfhostMonoInstanceKey>/, "mono cache intern invalid-key failure must keep the rejected typed key");
assert.match(internError, /\bStorage\s+<StdErrorKind>/, "mono cache intern storage failure must keep the stdlib storage error kind as enum payload");
assert.doesNotMatch(internError, /\bstr\b|\"invalid|\"storage/i, "mono cache intern errors must not be string sentinels");

const cacheRecordAt = topLevelBlock(mono, "fn", "selfhost_mono_instance_cache_record_at");
assert.match(cacheRecordAt, /Option<SelfhostMonoInstanceRecord>/, "cache record lookup must use typed Option absence");
assert.match(cacheRecordAt, /\bv::get<SelfhostMonoInstanceRecord>\s+records\s+instance_id\.index\b/, "cache record lookup must index the typed record table with the instance id payload");

const cacheLookup = topLevelBlock(mono, "fn", "selfhost_mono_instance_cache_lookup");
assert.match(cacheLookup, /Option<SelfhostMonoInstanceId>/, "cache key lookup must return Option<SelfhostMonoInstanceId>");
assert.match(cacheLookup, /\bselfhost_mono_instance_cache_lookup_loop\s+cache\s+key\s+0\s+selfhost_mono_instance_cache_len\s+cache\b/, "cache key lookup must delegate to the typed storage scan");

const cacheLookupLoop = topLevelBlock(mono, "fn", "selfhost_mono_instance_cache_lookup_loop");
assert.match(cacheLookupLoop, /\bselfhost_mono_instance_record_matches_key\s+record\s+key\b/, "cache lookup must compare the full typed record key");
assert.match(cacheLookupLoop, /\bsome\s+record\.instance_id\b/, "cache lookup must return the assigned id payload from the record");
assert.doesNotMatch(cacheLookupLoop, /\bselfhost_mono_instance_key_seed\b/, "cache lookup must not use mangle seed as identity");

const cacheIntern = topLevelBlock(mono, "fn", "selfhost_mono_instance_cache_intern");
assert.match(cacheIntern, /Result<SelfhostMonoInstanceCacheInternResult,\s*SelfhostMonoInstanceCacheInternError>/, "cache intern must return an owner-carrying typed result with typed intern errors");
assert.match(cacheIntern, /\bnot\s+selfhost_mono_instance_key_is_valid\s+key\b/, "cache intern must reject invalid keys before storage lookup");
assert.match(cacheIntern, /\bselfhost_mono_instance_cache_free\s+cache\b/, "cache intern must release cache owner when rejecting an invalid key");
assert.match(cacheIntern, /\bselfhost_mono_instance_cache_intern_error_invalid_key\s+key\b/, "cache intern must report invalid keys through the typed error enum");
assert.match(cacheIntern, /\bmatch\s+selfhost_mono_instance_cache_lookup\s+&cache\s+key:/, "cache intern must check existing keys before allocating");
assert.match(cacheIntern, /\bselfhost_mono_instance_record_new\s+key\s+instance_id\b/, "cache intern must store key/id as a typed record");
assert.match(cacheIntern, /\bv::push<SelfhostMonoInstanceRecord>\s+records\s+record\b/, "cache intern must append typed records to storage");
assert.match(cacheIntern, /\bselfhost_mono_instance_cache_intern_error_storage\s+error\b/, "cache intern must wrap storage failures in the typed error enum");
assert.doesNotMatch(cacheIntern, /Result<SelfhostMonoInstanceCacheInternResult,\s*StdErrorKind>/, "cache intern must not collapse invalid key and storage errors into StdErrorKind");
assert.doesNotMatch(cacheIntern, /\bSelfhostMonoInstanceId\b\s+-1\b|\bselfhost_mono_instance_id_new\s+-1\b/, "cache intern must not reintroduce invalid IDs");

assert.match(stage0, /\bselfhost_mono_instance_cache_new\b/, "stage0 must exercise typed mono cache storage creation");
assert.match(stage0, /\bselfhost_mono_instance_cache_intern\s+cache0\s+key0\b/, "stage0 must intern an instance key through cache storage");
assert.match(stage0, /\bselfhost_mono_instance_cache_lookup\s+&cache2\s+key1\b/, "stage0 must verify cache lookup through Option");
assert.match(stage0, /\bselfhost_mono_instance_cache_free\s+cache2\b/, "stage0 must free typed cache storage");

console.log("selfhost mono instance absence regression passed");
