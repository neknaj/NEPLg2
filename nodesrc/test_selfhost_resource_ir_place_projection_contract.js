const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(
    path.join(__dirname, "..", "stdlib", "neplg2", "core", "codegen", "resource_ir_place_projection.nepl"),
    "utf8",
);

function ordered(tokens, message) {
    let cursor = 0;
    for (const token of tokens) {
        const next = source.indexOf(token, cursor);
        assert.notEqual(next, -1, `${message}: missing ${token}`);
        cursor = next + token.length;
    }
}

ordered(
    [
        "enum SelfhostResourceIrResourceOffset:",
        "Known %i32",
        "Symbolic %i32",
        "ScaledSymbolic %SelfhostResourceIrScaledPlaceLink",
        "Offset %SelfhostResourceIrPlaceOffsetLink",
        "ScaledOffset %SelfhostResourceIrScaledPlaceOffsetLink",
        "Unknown",
        "enum SelfhostResourceIrPlaceProjection:",
        "Field %SelfhostResourceIrIndexedProjection",
        "TupleField %SelfhostResourceIrIndexedProjection",
        "EnumPayload %i32",
        "Deref",
        "StorageOffset %SelfhostResourceIrResourceOffset",
    ],
    "Rust projection and offset variants must remain exhaustive and variant-native",
);
assert.equal((source.match(/offset %i64/g) || []).length, 2, "Offset and ScaledOffset must both retain signed i64 payloads");

ordered(
    [
        "ResourceOffset::Known value:",
        "ge value 0",
        "ResourceOffset::Symbolic place_index:",
        "ge place_index 0",
        "ResourceOffset::ScaledSymbolic value:",
        "and ge value.place_index 0 ge value.scale 0",
        "ResourceOffset::Offset value:",
        "ge value.place_index 0",
        "ResourceOffset::ScaledOffset value:",
        "and ge value.place_index 0 ge value.scale 0",
        "ResourceOffset::Unknown:",
    ],
    "offset structural validation must be variant-specific",
);

ordered(
    [
        "selfhost_resource_ir_place_projection_is_inventory_supported",
        "SelfhostResourceIrPlaceProjection::StorageOffset offset:",
        "SelfhostResourceIrResourceOffset::Unknown:",
        "false",
        "_:",
        "true",
    ],
    "Unknown storage offsets must remain modeled but be classified unsupported at inventory scope",
);

assert.doesNotMatch(source, /ResourceIrEnumerated|ResourceLoweringTraversalProduced|RequestEvidenceProven/);
console.log("selfhost Resource IR Place projection contract ok");
