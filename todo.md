cast関連の実装中 fnのalias用法

<...> の中(型注釈や型引数として読む場所)で`::` PathSep を許可

stdlib/nmの実装

# targetの追加,再設計
現状: wasm か wasi
変更後: nasmを追加, wasip1 wasip2 wasix に変更
包含関係を上手く処理できるように注意すること
定義する側と、使用する側で、包含関係の判定処理が異なることなどに注意すること
```
if[target=wasm]
if[target=wasm&wasip1]
if[target=wasm&wasip1&wasip2]
if[target=wasm&wasip1&wasix]
if[target=nasm]
if[target=nasm|wasm]
if[target=nasm|(wasm&wasip1)]
if[target=nasm|(wasm&wasip1&wasip2)]
if[target=nasm|(wasm&wasip1&wasix)]
```
こんな感じ

NASM targetの追加
stdlib/coreとstdlib/allocはNASMとWASMの両方に対応させる
stdlib/stdはNASMとWASM&WASIP1の両方に対応させる
WASIp2やWASIXが必要な機能はstdlib/platformsで扱う

targetのエイリアスの追加

coreはnasm|wasm
stdはnasm|(wasm&wasip1)
```
if[target=core]
if[target=std]
```