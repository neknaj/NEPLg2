#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function read(rel) {
    return fs.readFileSync(path.join(repoRoot, rel), 'utf8');
}

const types = read('stdlib/alloc/encoding/json/types.nepl');
const builders = read('stdlib/alloc/encoding/json/builders.nepl');
const access = read('stdlib/alloc/encoding/json/access.nepl');
const serialize = read('stdlib/alloc/encoding/json/serialize.nepl');
const typedTests = read('tests/stdlib/json_typed_values.n.md');
const stdlibTests = read('stdlib/tests/json.n.md');

assert.match(types, /pub struct JsonArray:\s*\r?\n\s+items <str>/, 'JsonArray must be a typed JSON fragment');
assert.match(types, /pub struct JsonObject:\s*\r?\n\s+members <str>/, 'JsonObject must be a typed JSON fragment');
assert.match(types, /Array <JsonArray>/, 'JsonValue::Array must hold JsonArray');
assert.match(types, /Object <JsonObject>/, 'JsonValue::Object must hold JsonObject');
assert.doesNotMatch(types, /Array <Vec<JsonValue>>/, 'JsonValue::Array must not depend on Vec<JsonValue>');
assert.doesNotMatch(types, /Object <Vec<JsonMember>>/, 'JsonValue::Object must not depend on Vec<JsonMember>');

assert.doesNotMatch(builders, /#import "alloc\/collections\/vec"/, 'JSON builders must not import Vec');
assert.doesNotMatch(builders, /\bnew<Json(?:Value|Member)>/, 'JSON builders must not construct Vec<Json*>');
assert.doesNotMatch(builders, /\bpush<Json(?:Value|Member)>/, 'JSON builders must not push into Vec<Json*>');
assert.match(builders, /json_array_new <\(\)->Result<JsonArray, StdErrorKind>>/, 'json_array_new must return JsonArray');
assert.match(builders, /json_object_new <\(\)->Result<JsonObject, StdErrorKind>>/, 'json_object_new must return JsonObject');
assert.match(builders, /json_serialize value/, 'JSON builders must append serialized value fragments');

assert.match(access, /json_as_array <\(JsonValue\)->Option<JsonArray>>/, 'json_as_array must return JsonArray');
assert.match(access, /json_as_object <\(JsonValue\)->Option<JsonObject>>/, 'json_as_object must return JsonObject');

assert.doesNotMatch(serialize, /#import "core\/mem/, 'JSON serialize must not import raw memory modules');
assert.doesNotMatch(serialize, /\bmem_ptr_addr\b/, 'JSON serialize must not expose raw addresses');
assert.doesNotMatch(serialize, /\bload<Json(?:Value|Member)>/, 'JSON serialize must not raw-load Json payloads');
assert.match(serialize, /json_serialize_array <\(JsonArray\)->str>/, 'array serializer must consume JsonArray');
assert.match(serialize, /json_serialize_object <\(JsonObject\)->str>/, 'object serializer must consume JsonObject');

for (const [name, text] of [
    ['tests/stdlib/json_typed_values.n.md', typedTests],
    ['stdlib/tests/json.n.md', stdlibTests],
]) {
    assert.doesNotMatch(text, /Vec<Json(?:Value|Member)>/, `${name} must not exercise JSON through Vec<Json*>`);
}

console.log('json builder fragment contract passed');
