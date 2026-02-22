# LLVM IR Setup (clang 21.1.0)

This document describes the minimum setup to use LLVM IR with `clang 21.1.0` on Linux/WSL.

## Requirements

- `clang` 21.1.0
- `llvm-as` 21.1.0
- `llc` 21.1.0
- `lli` 21.1.0

Version check:

```bash
clang --version
llvm-as --version
llc --version
lli --version
```

## Environment variables for `llvm-sys`/`inkwell`

When using LLVM 21 in Rust (`llvm-sys`/`inkwell`), set the prefix to LLVM 21.1.0:

```bash
export LLVM_SYS_211_PREFIX=/opt/llvm-21.1.0
```

Optional:

```bash
export PATH=/opt/llvm-21.1.0/bin:$PATH
```

## LLVM IR workflow (quick check)

Create C source:

```bash
mkdir -p tmp/llvm_ir
cat > tmp/llvm_ir/hello.c << 'EOF'
#include <stdio.h>
int add(int a, int b) { return a + b; }
int main(void) { printf("sum=%d\n", add(20, 22)); return 0; }
EOF
```

Generate LLVM IR:

```bash
clang -S -emit-llvm -O0 tmp/llvm_ir/hello.c -o tmp/llvm_ir/hello.ll
```

Run IR directly:

```bash
lli tmp/llvm_ir/hello.ll
```

Expected output:

```text
sum=42
```

Compile IR to object and link:

```bash
llc -relocation-model=pic -filetype=obj tmp/llvm_ir/hello.ll -o tmp/llvm_ir/hello.o
clang tmp/llvm_ir/hello.o -o tmp/llvm_ir/hello
tmp/llvm_ir/hello
```

Expected output:

```text
sum=42
```

## Notes

- `-relocation-model=pic` avoids linker warnings related to PIE/text relocations on Linux.
- This is the current baseline (`clang 21.1.0 + linux native`) for upcoming LLVM IR target work in `nepl-cli`.
