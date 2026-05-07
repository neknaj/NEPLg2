---
id: ISS-20260507T153515441Z-ZED-EXTENSION-BUILD-ARTIFACTS-ARE-TR-B7D814F1
title: "Zed extension build artifacts are tracked in git"
area: tools
status: open
resolved: false
priority: P2
type: maintenance
created: 2026-05-07
updated: 2026-05-07
target: "editors/zed/target, editors/zed/.gitignore"
---

# ISS-20260507T153515441Z-ZED-EXTENSION-BUILD-ARTIFACTS-ARE-TR-B7D814F1: Zed extension build artifacts are tracked in git

## 概要

editors/zed/target contains 945 tracked Cargo build artifacts. Generated target output is mixed into the source tree and can drift independently from source changes.

## 対象

- `editors/zed/target, editors/zed/.gitignore`

## 根拠

- 未記入

## 問題

editors/zed/target contains 945 tracked Cargo build artifacts. Generated target output is mixed into the source tree and can drift independently from source changes.

## 影響

Full reviews and source policy scans are noisy, checkout size grows unnecessarily, and stale build products can hide whether the Zed extension is reproducible from source. This violates the project policy that generated artifacts must not be treated as maintained source.

## 修正方針

Remove editors/zed/target from git, add a scoped ignore rule for the Zed extension target directory, and keep only source files, grammar, package metadata, and docs tracked.

## 検証

git ls-files editors/zed/target returns no files. Building the Zed extension recreates target/ as ignored output. Source review file lists no longer include generated Cargo artifacts.
