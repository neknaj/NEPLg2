#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const { migrateText } = require("./neplg21_syntax_migrate");

assert.equal(
  migrateText("fn main <()->i32> ():\n    0\n"),
  "fn main %fn void i32 \\void:\n    0\n",
  "legacy zero-argument function syntax must migrate to void marker syntax",
);

assert.equal(
  migrateText("fn make %fn unit fn unit unit \\unit:\n    unit_id\n"),
  "fn make %fn void fn void unit \\void:\n    unit_id\n",
  "legacy unit zero-argument markers inside nested function types must migrate together",
);

assert.equal(
  migrateText("fn id_unit %fn unit unit \\a:\n    a\n"),
  "fn id_unit %fn unit unit \\a:\n    a\n",
  "new unary unit argument syntax must not be rewritten as a zero-argument function",
);

assert.equal(
  migrateText("fn main %fn void i32:\n    0\n"),
  "fn main %fn void i32 \\void:\n    0\n",
  "missing zero-argument lambda marker can be repaired for valid void function types",
);

assert.equal(
  migrateText("fn bad %fn void void:\n    unit\n"),
  "fn bad %fn void void:\n    unit\n",
  "void used as a result type is invalid source and must not be silently repaired",
);

console.log("NEPLg2.1 syntax migrator regression passed");
