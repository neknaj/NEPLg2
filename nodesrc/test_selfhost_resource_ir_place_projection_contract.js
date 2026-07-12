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
        "struct SelfhostResourceIrVariantStableSymbol:",
        "schema_version %i32",
        "identity %i32",
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
        "EnumPayload %SelfhostResourceIrVariantStableSymbol",
        "Deref",
        "StorageOffset %SelfhostResourceIrResourceOffset",
    ],
    "Rust projection and offset variants must remain exhaustive and variant-native",
);
assert.equal((source.match(/offset %i64/g) || []).length, 2, "Offset and ScaledOffset must both retain signed i64 payloads");

ordered(
    [
        "selfhost_resource_ir_variant_stable_symbol_is_valid",
        "and eq symbol.schema_version 1 not eq symbol.identity 0",
        "selfhost_resource_ir_variant_stable_symbol_eq",
        "and selfhost_resource_ir_variant_stable_symbol_is_valid left and selfhost_resource_ir_variant_stable_symbol_is_valid right and eq left.schema_version right.schema_version eq left.identity right.identity",
        "SelfhostResourceIrPlaceProjection::EnumPayload variant_symbol:",
        "selfhost_resource_ir_variant_stable_symbol_is_valid variant_symbol",
    ],
    "EnumPayload must reject placeholder symbols and use one typed comparison authority",
);

ordered(
    [
        "SelfhostResourceIrVariantStableSymbol 1 41",
        "SelfhostResourceIrVariantStableSymbol 1 43",
        "SelfhostResourceIrVariantStableSymbol 1 0",
        "enum_placeholder_rejected",
        "enum_unknown_schema_rejected",
        "enum_symbol_equal",
        "enum_symbols_distinct",
        "enum_placeholders_not_equal",
        "enum_unknown_schemas_not_equal",
    ],
    "projection stage0 must cover accepted, placeholder, equal, and distinct variant symbols",
);

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
