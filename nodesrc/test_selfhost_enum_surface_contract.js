#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const relPath = "stdlib/neplg2/core/resolve/type_resolver/enum_surface.nepl";
const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
const facade = fs.readFileSync(path.join(repoRoot, "stdlib/neplg2/core/resolve/type_resolver.nepl"), "utf8").replace(/\r\n/g, "\n");

function topLevelBlock(kind, name) {
    const escaped = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const match = source.match(new RegExp(`^(?:pub\\s+)?${kind}\\s+${escaped}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?(?:struct|enum|fn|impl)\\s+|\\n#|\\n//: neplg2:test|\\n$)`, "m"));
    assert.ok(match, `missing top-level ${kind} ${name}`);
    return match[0];
}

assert.match(facade, /pub #import "\.\/type_resolver\/enum_surface" as \*/, "type resolver facade must export enum surface materialization");
assert.match(source, /pub struct SelfhostResolvedEnumMemberId:[\s\S]*nominal_id %SelfhostNamedTypeId[\s\S]*variant_ordinal %i32/, "resolved enum member identity must separate nominal identity from definition-local ordinal");
const memberId = topLevelBlock("pub fn", "selfhost_resolved_enum_session_member_id");
assert.match(memberId, /selfhost_resolved_enum_session_definition_get[\s\S]*variant_ordinal[\s\S]*definition\.variant_count[\s\S]*lt definition\.variant_start 0[\s\S]*2147483647[\s\S]*selfhost_resolved_enum_session_variant_get[\s\S]*SelfhostResolvedEnumMemberId definition\.nominal_id variant_ordinal/, "member identity must be issued only from an overflow-safe same-session definition range membership");
assert.doesNotMatch(memberId, /str|string_slice|name_span|canonical/, "member identity producer must not derive identity from source spelling");
const memberLookup = topLevelBlock("pub fn", "selfhost_resolved_enum_session_member_lookup_result");
assert.match(memberLookup, /&SelfhostSourceText[\s\S]*query_span\.file_id[\s\S]*query_file_id[\s\S]*string_access::len query_source[\s\S]*selfhost_resolved_enum_surface_utf8_boundary[\s\S]*selfhost_resolved_enum_member_definition_find_loop[\s\S]*selfhost_resolved_enum_member_query_lookup_loop/, "member lookup must bind the query span to its source owner before same-session lookup");
assert.doesNotMatch(memberLookup, /split|alias|qualified|string_slice/, "member lookup must accept resolved nominal evidence instead of interpreting qualifier spelling");
assert.match(memberLookup, /selfhost_resolved_enum_member_definition_count_loop[\s\S]*definition_count[\s\S]*not eq definition_count 1/, "member lookup must reject duplicate nominal definitions");
assert.match(memberLookup, /SelfhostResolvedNominalDeclarationKind::Enum[\s\S]*selfhost_resolved_enum_session_constructor_kind/, "member lookup must rejoin enum definitions to same-session constructor membership");
assert.match(source, /pub struct SelfhostResolvedEnumSession:[\s\S]*source %str[\s\S]*tokens %Vec SelfhostToken[\s\S]*constructors %SelfhostTypeConstructorTable[\s\S]*definitions %Vec SelfhostResolvedEnumDefinition[\s\S]*variants %Vec SelfhostResolvedEnumVariantHeader/, "module session must own one parse origin, constructor identity space, dense definitions, and ordered variants");
assert.doesNotMatch(source, /impl (?:Clone|Copy) for SelfhostResolvedEnumSession/, "module session must remain move-only");
assert.match(source, /pub struct SelfhostResolvedEnumVariantHeader:[\s\S]*name_span %SelfhostSourceSpan[\s\S]*payload %Option SelfhostSyntaxRange/, "variant records must keep only source spans and optional payload syntax");
assert.doesNotMatch(topLevelBlock("struct", "SelfhostResolvedEnumVariantHeader"), /name %str/, "variant records must not retain source-backed spelling owners");
assert.match(source, /pub enum SelfhostResolvedEnumSurfaceErrorKind:[\s\S]*ItemHeaderOriginMismatch[\s\S]*HeaderOriginMismatch[\s\S]*BodyOriginMismatch[\s\S]*DeclarationNameUnavailable[\s\S]*OutOfMemory[\s\S]*ConstructorTable/, "origin, unavailable-name, allocation, and constructor failures must remain distinct");

const producer = topLevelBlock("fn", "selfhost_resolved_enum_module_session_materialize_with_file_id_result");
const modeProducer = topLevelBlock("fn", "selfhost_resolved_enum_module_session_materialize_mode_with_file_id_result");
assert.match(producer, /selfhost_resolved_enum_module_session_materialize_mode_with_file_id_result constructor_seed source file_id false/, "public materializer must delegate to the closed internal mode with attach disabled");
assert.match(modeProducer, /lex_all_with_file_id source file_id[\s\S]*selfhost_parse_module_tokens source &tokens[\s\S]*selfhost_resolved_enum_module_collect_loop[\s\S]*selfhost_resolved_enum_session_attach_tokens/, "internal mode producer must lex with the VFS file identity, parse, scan the complete module, and retain its token origin");
assert.match(producer, /SelfhostTypeConstructorTable[\s\S]*str[\s\S]*i32 Result SelfhostResolvedEnumSession/, "public producer must accept only a constructor namespace seed, source, and file identity");
assert.doesNotMatch(producer, /SelfhostModuleAst|&Vec SelfhostToken|SelfhostModuleItem/, "public producer must not accept forgeable external AST, token, or item inputs");
const compatibilityProducer = topLevelBlock("fn", "selfhost_resolved_enum_module_session_materialize_result");
assert.match(compatibilityProducer, /selfhost_resolved_enum_module_session_materialize_with_file_id_result constructor_seed source 0/, "single-file compatibility producer must delegate with file ID zero");
assert.doesNotMatch(source, /pub fn selfhost_resolved_enum_session_(?:append_result|new|from_constructor_table)/, "single-item or reopenable session APIs must not be public");
assert.doesNotMatch(source, /pub fn selfhost_resolved_enum_surface_materialize_result/, "legacy single-item materialization must not remain public");
assert.doesNotMatch(source, /pub fn selfhost_resolved_enum_session_nominal_id/, "ambiguous first-definition nominal accessor must not remain public");
assert.doesNotMatch(source, /pub fn selfhost_resolved_enum_variant_name_eq/, "variant spelling must not be compared against an external source");
const nameEq = topLevelBlock("fn", "selfhost_resolved_enum_session_variant_name_eq");
assert.match(nameEq, /&SelfhostResolvedEnumSession[\s\S]*variant_idx[\s\S]*field::get_ref session "source"/, "variant spelling lookup must remain bound to the session-owned source and variant index");
const append = topLevelBlock("fn", "selfhost_resolved_enum_session_append_result");
assert.match(append, /SelfhostModuleItemKind::EnumDecl[\s\S]*item\.declaration[\s\S]*item\.declaration_body[\s\S]*selfhost_resolved_enum_surface_materialize_header_result/, "session append must start from actual parser EnumDecl evidence");
assert.doesNotMatch(append, /selfhost_named_type_id_new/, "session append must not forge a raw table-local nominal ID");
const origin = topLevelBlock("fn", "selfhost_resolved_enum_surface_validate_origin");
assert.match(origin, /item\.span[\s\S]*header\.header_span[\s\S]*span_inside[\s\S]*selfhost_parser_declaration_body_range[\s\S]*selfhost_resolved_enum_surface_body_eq/, "item, header, keyword/head, and exact body ranges must share one parser origin");

const finish = topLevelBlock("fn", "selfhost_resolved_enum_surface_finish");
const finishDefinition = topLevelBlock("fn", "selfhost_resolved_enum_surface_finish_definition");
assert.match(finish, /selfhost_type_constructor_table_add_checked[\s\S]*selfhost_type_constructor_add_result_nominal_id[\s\S]*selfhost_type_constructor_add_result_into_table[\s\S]*selfhost_resolved_enum_surface_finish_definition/, "normal session identity and table owner must come from the same checked constructor add result");
assert.match(finishDefinition, /SelfhostResolvedEnumDefinition[\s\S]*v::push definitions definition[\s\S]*SelfhostResolvedEnumSession source tokens constructors next_definitions variants attach_existing/, "shared finalizer must retain the selected constructor table and nominal in one session owner");
assert.doesNotMatch(finish, /selfhost_named_type_id_new/, "session finish must not reconstruct nominal identity from an ordinal");
assert.match(source, /constructor_seed.*namespace prefixであり、parser origin証明ではありません/, "constructor seed must be documented as a namespace prefix rather than parser-origin evidence");

const scan = topLevelBlock("fn", "selfhost_resolved_enum_surface_scan_variants");
assert.match(scan, /selfhost_body_segment_list_from_envelope[\s\S]*selfhost_resolved_enum_surface_variant_scan_loop[\s\S]*selfhost_body_segment_list_free/, "variant scan must reuse parser body segmentation and close its temporary owner");
const variant = topLevelBlock("fn", "selfhost_resolved_enum_surface_variant_from_segment");
assert.match(variant, /SelfhostBodySegmentKind::BlockIntro[\s\S]*VariantSegmentKindInvalid[\s\S]*TokenKind::Ident[\s\S]*DuplicateVariant[\s\S]*TokenKind::Percent[\s\S]*TokenKind::LAngle[\s\S]*InvalidPayloadIntroducer/, "enum variants must be flat identifier-led segments with closed payload introducers");
assert.match(source, /GenericBoundsUnsupported/, "generic bounds must be explicitly rejected by this shallow surface slice");
assert.match(source, /VariantSpanUnavailable[\s\S]*DeclarationNameUnavailable[\s\S]*OutOfMemory/, "variant/name boundary failures and allocation failures must remain distinct");
assert.doesNotMatch(source, /WrongItemKind/, "non-enum module items must be skipped rather than rejected");
assert.match(source, /fn check_no_enum_module[\s\S]*definition_len &enum_session 0[\s\S]*variant_len &enum_session 0/, "a module without enums must produce an empty successful session");

const free = topLevelBlock("fn", "selfhost_resolved_enum_session_free");
assert.match(free, /v::free[\s\S]*v::free[\s\S]*v::free[\s\S]*selfhost_type_constructor_table_free/, "session cleanup must close variants, definitions, tokens, and constructor table");
assert.match(source, /variant record は source span だけを保持/, "span-only variant ownership must be documented");
assert.match(source, /Resource production origin ではありません/, "enum session must explicitly remain outside Resource production authority");
assert.doesNotMatch(source, /production origin を(発行|生成|返)/, "enum surface transaction must not claim Resource production authority");

console.log("selfhost enum surface contract passed");
