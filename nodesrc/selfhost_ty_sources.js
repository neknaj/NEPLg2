"use strict";

const fs = require("node:fs");
const path = require("node:path");

const TY_FACADE = "stdlib/neplg2/core/ty/ty.nepl";
const TY_ROOT_REEXPORT_FILES = [
    "stdlib/neplg2/core/ty/ty/id.nepl",
    "stdlib/neplg2/core/ty/ty/kind.nepl",
    "stdlib/neplg2/core/ty/ty/record.nepl",
    "stdlib/neplg2/core/ty/ty/arena.nepl",
    "stdlib/neplg2/core/ty/ty/eq.nepl",
    "stdlib/neplg2/core/ty/ty/key.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_policy.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_artifact_word_codec.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload_codec.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_reader.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl",
    "stdlib/neplg2/core/ty/ty/stage0.nepl",
];
const TY_SPLIT_FILES = [
    "stdlib/neplg2/core/ty/ty/id.nepl",
    "stdlib/neplg2/core/ty/ty/kind.nepl",
    "stdlib/neplg2/core/ty/ty/kind/model.nepl",
    "stdlib/neplg2/core/ty/ty/kind/eq.nepl",
    "stdlib/neplg2/core/ty/ty/kind/name.nepl",
    "stdlib/neplg2/core/ty/ty/record.nepl",
    "stdlib/neplg2/core/ty/ty/arena.nepl",
    "stdlib/neplg2/core/ty/ty/eq.nepl",
    "stdlib/neplg2/core/ty/ty/key.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_layout.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_producer.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_policy.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_source.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_artifact_word_codec.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_canonical_key_payload_codec.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_store.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_artifact.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_index.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_decoded.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_reader.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_payload_reader.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_serializer.nepl",
    "stdlib/neplg2/core/ty/ty/memo_trait_proof_preseed.nepl",
    "stdlib/neplg2/core/ty/ty/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readTySource(repoRoot) {
    return [TY_FACADE, ...TY_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    TY_FACADE,
    TY_ROOT_REEXPORT_FILES,
    TY_SPLIT_FILES,
    readRepoFile,
    readTySource,
};
