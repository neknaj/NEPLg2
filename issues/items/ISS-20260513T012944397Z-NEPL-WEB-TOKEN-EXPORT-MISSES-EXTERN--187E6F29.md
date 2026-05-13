---
id: ISS-20260513T012944397Z-NEPL-WEB-TOKEN-EXPORT-MISSES-EXTERN--187E6F29
title: "nepl-web token export misses extern visibility field"
area: web
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: nepl-web/src/lib.rs
---

# ISS-20260513T012944397Z-NEPL-WEB-TOKEN-EXPORT-MISSES-EXTERN--187E6F29: nepl-web token export misses extern visibility field

## 概要

`trunk build` fails because `token_extra` matches `TokenKind::DirExtern` without the `vis` field added by extern visibility support.

## 対象

- `nepl-web/src/lib.rs`

## 根拠

- `TokenKind::DirExtern` now carries `vis`, `module`, `name`, `func`, and `signature`.
- `nepl-web` exposes token details to the playground/editor, so token export must stay exhaustive with the compiler token model.

## 問題

`nepl-web/src/lib.rs` still used the old `DirExtern { module, name, func, signature }` pattern. Rust compilation therefore rejected the web target before doctest/playground validation could reach deploy artifacts.

## 影響

`web/dist` cannot be rebuilt, so doctest and playground validation may run against stale compiler artifacts.

## 修正方針

Include `vis` in the `DirExtern` token pattern and expose it in the token detail string.

## 検証

- `cargo check --manifest-path nepl-web\Cargo.toml`
- `trunk build`
