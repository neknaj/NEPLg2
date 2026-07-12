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
        "enum SelfhostResourceIrUsizeNarrowErrorKind:",
        "FieldIndexNegative",
        "FieldIndexOutOfRange",
        "FieldOffsetBytesNegative",
        "FieldOffsetBytesOutOfRange",
        "TupleFieldIndexNegative",
        "TupleFieldIndexOutOfRange",
        "TupleFieldOffsetBytesNegative",
        "TupleFieldOffsetBytesOutOfRange",
        "ResourceOffsetKnownNegative",
        "ResourceOffsetKnownOutOfRange",
        "ScaledSymbolicScaleNegative",
        "ScaledSymbolicScaleOutOfRange",
        "ScaledOffsetScaleNegative",
        "ScaledOffsetScaleOutOfRange",
        "selfhost_resource_ir_usize_narrow_result",
        "lt value extend_s 0",
        "gt value extend_s 2147483647",
        "SelfhostResourceIrUsizeNarrowResult::Ok wrap value",
    ],
    "producer-facing usize inputs must use one checked i64-to-supported-i32 narrowing authority with field-specific errors",
);

for (const constructor of [
    "selfhost_resource_ir_field_projection_from_usize_i64",
    "selfhost_resource_ir_tuple_field_projection_from_usize_i64",
    "selfhost_resource_ir_known_offset_from_usize_i64",
    "selfhost_resource_ir_scaled_symbolic_offset_from_usize_i64",
    "selfhost_resource_ir_scaled_offset_from_usize_i64",
]) {
    assert.match(source, new RegExp(`pub fn ${constructor}`), `${constructor} must expose the checked materializer boundary`);
}

function topLevelBlock(src, kind, name) {
    const start = src.indexOf(`${kind} ${name}`);
    assert.notEqual(start, -1, `missing ${kind} ${name}`);
    const next = src.indexOf(`\n${kind} `, start + 1);
    const nextPublic = src.indexOf(`\npub ${kind} `, start + 1);
    const ends = [next, nextPublic].filter((value) => value !== -1);
    return src.slice(start, ends.length === 0 ? src.length : Math.min(...ends));
}
assert.match(source, /let too_large %i64 add max extend_s 1/, "runtime smoke must construct 2147483648 without overflowing i32");
assert.match(source, /selfhost_resource_ir_usize_narrowing_stage0 \(\)/, "public projection smoke must execute narrowing boundaries");
assert.match(source, /i64[^\n]*Rust usize\u5168\u57df[^\n]*\u3067\u306f\u306aく/, "module contract must not claim that i64 represents the complete Rust usize domain");

const narrowingStage = topLevelBlock(source, "fn", "selfhost_resource_ir_usize_narrowing_stage0");
for (const token of [
    "let zero %i64 extend_s 0",
    "let max %i64 extend_s 2147483647",
    "let too_large %i64 add max extend_s 1",
    "selfhost_resource_ir_projection_narrow_result_is_valid field_zero",
    "selfhost_resource_ir_projection_narrow_result_is_valid tuple_max",
    "selfhost_resource_ir_offset_narrow_result_is_valid known_zero",
    "selfhost_resource_ir_offset_narrow_result_is_valid scaled_symbolic_max",
    "selfhost_resource_ir_offset_narrow_result_is_valid scaled_offset_max",
]) {
    assert.match(narrowingStage, new RegExp(token.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")), `narrowing stage0 missing ${token}`);
}
assert.match(
    topLevelBlock(source, "fn", "selfhost_resource_ir_place_projection_stage0"),
    /selfhost_resource_ir_usize_narrowing_stage0 \(\)/,
    "public projection stage0 must execute the checked narrowing stage",
);
for (let code = 1; code <= 14; code += 1) {
    assert.match(narrowingStage, new RegExp(`(?:projection|offset)_narrow_result_error_code_eq[^\\n]* ${code}(?: and|\\n)`), `narrowing stage0 must exact-match field-specific error code ${code}`);
}

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
