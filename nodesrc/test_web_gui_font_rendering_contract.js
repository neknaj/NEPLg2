#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function withoutComments(text) {
    return text
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//"))
        .join("\n");
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertMatch(text, pattern, message) {
    assert(pattern.test(text), `${message}: expected ${pattern}`);
}

function assertNoMatch(text, pattern, message) {
    assert(!pattern.test(text), `${message}: forbidden ${pattern}`);
}

const spec = read("doc/neplg2/gui_font_rendering_spec.md");
const detailedDesign = read("doc/neplg2/gui_font_rendering_detailed_design.md");
const implementationPlan = read("doc/neplg2/gui_font_rendering_implementation_plan.md");
const redesignPlan = read("doc/neplg2/gui_redesign_implementation_plan.md");

const coreFont = read("stdlib/core/gui/font.nepl");
const coreFontImpl = withoutComments(coreFont);
const renderStyle = read("stdlib/core/gui/render_style.nepl");
const renderStyleImpl = withoutComments(renderStyle);
const fontResource = read("stdlib/std/gui/font_resource.nepl");
const fontResourceImpl = withoutComments(fontResource);
const coreGuiFacade = read("stdlib/core/gui.nepl");
const coreGuiPrelude = read("stdlib/core/gui/prelude.nepl");
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiCoreTests = read("tests/stdlib/gui_core.n.md");
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const webFontResourceVfs = read("web/src/gui-font/font-resource-vfs.ts");
const webMain = read("web/src/main.ts");
const webPanelManager = read("web/src/workspace/panel-manager.ts");
const webTerminal = read("web/src/terminal/terminal.ts");
const webShell = read("web/src/terminal/shell.ts");
const webVfs = read("web/src/runtime/vfs.ts");
const webIndex = read("web/index.html");
const webFontResourceBehaviorTest = read("nodesrc/test_web_gui_font_resource_vfs_behavior.js");

assertMatch(
    spec,
    /GuiGlyphPaint:[\s\S]*shadows\s+GuiShadowRef[\s\S]*GuiShadowRef:[\s\S]*NoShadow[\s\S]*SingleShadow\s+GuiShadow[\s\S]*ShadowRun\s+GuiShadowRunId/,
    "font spec must use GuiShadowRef for no_alloc and alloc-backed multi-shadow contract",
);
assertMatch(
    detailedDesign,
    /F2 の `GuiFontResourceRequest` constructor は request shape だけを検査する[\s\S]*face_count[\s\S]*F4/,
    "font detailed design must defer collection face_count validation until parser or registry phase",
);
assertMatch(
    implementationPlan,
    /GuiFontResourcePath[\s\S]*GuiResourceHash[\s\S]*F2 は request shape だけを検査する/,
    "font implementation plan must use typed path/hash and keep F2 validation narrow",
);
assertMatch(
    spec,
    /Formal font renderer[\s\S]*MockTextMeasurer[\s\S]*fallback[\s\S]*してはならない/,
    "font spec must quarantine fixed-cell measurers away from formal font renderer fallback",
);
assertMatch(
    spec,
    /canonical 表記[\s\S]*fonts\/HackGenConsoleNF-Regular\.ttf[\s\S]*\/fonts\/HackGenConsoleNF-Regular\.ttf[\s\S]*suffix match[\s\S]*authority として使ってはならない/,
    "font spec must separate canonical resource path from Web VFS path and forbid suffix authority",
);
assertMatch(
    redesignPlan,
    /Phase 7:[\s\S]*font and 2D renderer contract slice[\s\S]*GuiShadowRef[\s\S]*resource hash と path は専用 value/,
    "GUI redesign plan must route font and render style work through typed Phase 7 contracts",
);
assertMatch(
    detailedDesign,
    /Web Playground は startup で mount promise を開始し[\s\S]*`neplg2 run`[\s\S]*失敗した場合は `GuiFontResourceMountError`[\s\S]*Compile-only path は runtime font bytes を必要としないため mount 完了を待たない/,
    "font detailed design must define Web run-time mount waiting without blocking compile-only path",
);
assertMatch(
    detailedDesign,
    /Native では packaged resource directory または configured resource root[\s\S]*suffix scan[\s\S]*Bare では embedded blob table[\s\S]*未設定の環境では filesystem probing を行わず unsupported/,
    "font detailed design must define Native resource root and Bare embedded blob behavior",
);
assertMatch(
    implementationPlan,
    /Phase F3:[\s\S]*canonical resource path `fonts\/HackGenConsoleNF-Regular\.ttf`[\s\S]*`web\/src\/gui-font\/font-resource-vfs\.ts`[\s\S]*HackGen 専用 API[\s\S]*binary\/read-only file の compile overlay 混入を禁止/,
    "font implementation plan must include F3 bundled resource routing and source policy scope",
);

assertMatch(
    coreFontImpl,
    /pub\s+enum\s+GuiWritingMode:[\s\S]*HorizontalLtr[\s\S]*HorizontalRtl[\s\S]*VerticalRl[\s\S]*VerticalLr/,
    "core/gui/font must expose all required writing modes as enum variants",
);
assertMatch(
    coreFontImpl,
    /pub\s+enum\s+GuiFontErrorKind:[\s\S]*InvalidFontSize[\s\S]*InvalidFaceIndex[\s\S]*FaceIndexRequired[\s\S]*MissingFontResource[\s\S]*UnsupportedFontContainer[\s\S]*MissingGlyph[\s\S]*InvalidGlyphPaint/,
    "core/gui/font must expose font error categories as enum values",
);
assertMatch(
    coreFontImpl,
    /pub\s+fn\s+gui_font_size_result\s+%fn\s+i32\s+fn\s+i32\s+Result\s+GuiFontSize\s+GuiError[\s\S]*gt\s+px_den\s+0[\s\S]*GuiError::InvalidCommand/,
    "core/gui/font must reject invalid font size denominator with Result",
);
assertNoMatch(
    coreFontImpl,
    /^\s*#import\s+"(?:alloc|std|platforms)\//m,
    "core/gui/font must stay no_alloc and not import alloc/std/platforms",
);

assertMatch(
    renderStyleImpl,
    /pub\s+enum\s+GuiShadowRef:[\s\S]*NoShadow[\s\S]*SingleShadow\s+%GuiShadow[\s\S]*ShadowRun\s+%GuiShadowRunId/,
    "core/gui/render_style must use GuiShadowRef instead of Vec in core",
);
assertMatch(
    renderStyleImpl,
    /pub\s+fn\s+gui_glyph_paint_result[\s\S]*and\s+is_none\s+fill\s+is_none\s+stroke[\s\S]*GuiError::InvalidCommand/,
    "core/gui/render_style must reject glyph paint with neither fill nor stroke",
);
assertNoMatch(
    renderStyleImpl,
    /^\s*#import\s+"(?:alloc|std|platforms)\//m,
    "core/gui/render_style must stay no_alloc and not import alloc/std/platforms",
);

assertMatch(
    fontResourceImpl,
    /pub\s+struct\s+GuiFontResourceRequest:[\s\S]*path\s+%GuiFontResourcePath[\s\S]*face_index\s+%Option\s+i32[\s\S]*expected_hash\s+%Option\s+GuiResourceHash[\s\S]*decode_policy\s+%GuiFontDecodePolicy/,
    "std/gui/font_resource must expose typed font resource request fields",
);
assertMatch(
    fontResourceImpl,
    /pub\s+fn\s+gui_font_resource_path_result\s+%fn\s+str\s+Result\s+GuiFontResourcePath\s+GuiError[\s\S]*gt\s+len\s+path\s+0[\s\S]*GuiError::InvalidCommand/,
    "std/gui/font_resource must reject empty resource paths",
);
assertMatch(
    fontResourceImpl,
    /gui_font_face_index_is_valid[\s\S]*Option::Some\s+index:[\s\S]*ge\s+index\s+0/,
    "std/gui/font_resource must reject negative face indexes at request-shape boundary",
);
assertNoMatch(
    fontResourceImpl,
    /\b(?:DOM|Canvas|FontFace|CoreText|DirectWrite|fontconfig|HWND|UIKit|AndroidView|HtmlCanvas)\b/,
    "std/gui/font_resource must not expose concrete font or platform handles",
);
assertNoMatch(
    fontResourceImpl,
    /\b(?:MockTextMeasurer|HostTextMeasurer|host_text_measurer|host_text_measurer_fixed|measure_text)\b/,
    "std/gui/font_resource must not depend on fixed-cell text measurement fallback",
);

assertMatch(coreGuiFacade, /#import\s+"\.\/gui\/font"\s+as\s+@merge/, "core/gui facade must export font contract");
assertMatch(coreGuiFacade, /#import\s+"\.\/gui\/render_style"\s+as\s+@merge/, "core/gui facade must export render style contract");
assertMatch(coreGuiPrelude, /#import\s+"\.\/font"\s+as\s+@merge/, "core/gui prelude must export font contract");
assertMatch(coreGuiPrelude, /#import\s+"\.\/render_style"\s+as\s+@merge/, "core/gui prelude must export render style contract");
assertMatch(stdGuiFacade, /#import\s+"\.\/gui\/font_resource"\s+as\s+\*/, "std/gui facade must export font resource boundary");
assertMatch(guiCoreTests, /gui_core_font_metrics_and_glyph_paint_contract/, "gui_core doctests must cover font metrics and glyph paint");
assertMatch(guiStdTests, /gui_font_resource_request_is_typed_boundary/, "gui_std doctests must cover font resource request boundary");

assertMatch(
    webFontResourceVfs,
    /export type GuiFontResourcePathErrorReason[\s\S]*Empty[\s\S]*Absolute[\s\S]*Backslash[\s\S]*EmptySegment[\s\S]*DotSegment[\s\S]*ParentSegment[\s\S]*VfsPathMismatch/,
    "Web font resource VFS must classify invalid resource paths as typed reasons",
);
assertMatch(
    webFontResourceVfs,
    /export const GUI_FONT_RESOURCE_ROOT = 'fonts';[\s\S]*HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH = `\$\{GUI_FONT_RESOURCE_ROOT\}\/HackGenConsoleNF-Regular\.ttf`/,
    "Web font resource VFS must keep canonical paths slashless",
);
assertNoMatch(
    webFontResourceVfs,
    /GUI_FONT_RESOURCE_ROOT\s*=\s*'\/fonts'/,
    "Web font resource root must not use VFS absolute path as canonical resource path",
);
assertMatch(
    webFontResourceVfs,
    /resourcePath: HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH[\s\S]*vfsPath: `\/\$\{HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH\}`[\s\S]*sourceUrl: '\.\/src\/fonts\/HackGenConsoleNF-Regular\.ttf'/,
    "Web bundled font manifest must map HackGen fixture to canonical path and VFS path explicitly",
);
assertMatch(
    webFontResourceVfs,
    /resourcePath: HACKGEN_LICENSE_RESOURCE_PATH[\s\S]*vfsPath: `\/\$\{HACKGEN_LICENSE_RESOURCE_PATH\}`[\s\S]*sourceUrl: '\.\/src\/fonts\/HackGen-LICENSE\.txt'/,
    "Web bundled font manifest must include the license text as a normal resource",
);
assertMatch(
    webFontResourceVfs,
    /export function normalizeGuiFontResourcePath[\s\S]*rawPath\.startsWith\('\/'\)[\s\S]*rawPath\.includes\('\\\\'\)[\s\S]*segment === '\.'[\s\S]*segment === '\.\.'/,
    "Web font resource path normalization must reject absolute, backslash, dot, and parent segments",
);
assertMatch(
    webFontResourceVfs,
    /export function guiFontResourceVfsPath[\s\S]*return `\/\$\{path\}`/,
    "Web font resource VFS path must be derived from canonical path with a single leading slash",
);
for (const errorKind of [
    "FetchUnavailable",
    "InvalidResourcePath",
    "NetworkError",
    "HttpError",
    "InvalidBytes",
    "InvalidText",
    "VfsWriteFailed",
]) {
    assertMatch(
        webFontResourceVfs,
        new RegExp(`kind: '${errorKind}'`),
        `Web font resource mount must expose typed ${errorKind} error`,
    );
}
assertMatch(
    webFontResourceVfs,
    /options: \{ fetch\?: GuiFontResourceFetch; resources\?: readonly GuiBundledFontResource\[\] \}[\s\S]*const resources = options\.resources \?\? BUNDLED_GUI_FONT_RESOURCES/,
    "Web font resource mount must support injected resource manifests without adding HackGen-specific APIs",
);
assertMatch(
    webFontResourceVfs,
    /vfs\.writeFile\(payload\.vfsPath, payload\.content, \{ force: true \}\)[\s\S]*vfs\.setReadOnly\(payload\.vfsPath, true\)[\s\S]*rollbackMountedFontResource\(vfs, payload\.vfsPath\)[\s\S]*rollbackMountedFontResource\(vfs, mountedPath\)/,
    "Web font resource mount must write read-only VFS files and roll back partial writes",
);
assertNoMatch(
    webFontResourceVfs,
    /\b(?:FontFace|CanvasRenderingContext2D|document|localStorage|indexedDB)\b/,
    "Web font resource VFS must not depend on browser font or persistent storage APIs",
);
assertNoMatch(
    webFontResourceVfs,
    /\b(?:fallback|Fallback)\b/,
    "Web font resource VFS must not describe hidden fallback behavior",
);
assertMatch(
    webMain,
    /const guiFontResourceMountPromise: Promise<GuiFontResourceMountResult> = mountBundledGuiFontResources\(vfs\)/,
    "Web main must start GUI font resource mounting at startup",
);
assertMatch(
    webMain,
    /beforeWasmExecution: guiFontResourceExecutionPreflight/,
    "Web main must pass GUI font resource preflight into terminal runtime",
);
assertMatch(
    webMain,
    /async function runCurrentFile\(\)[\s\S]*await ensureGuiFontResourcesForRun\(\)[\s\S]*executeCommand\(`neplg2 run -i \$\{activePath\}`\)/,
    "Web run path must await GUI font resource mount before execution",
);
assertMatch(
    webMain,
    /function compileCurrentFile\(\)[\s\S]*executeCommand\(`neplg2 build --emit wat -i \$\{activePath\}`\)/,
    "Web compile path must remain independent from runtime font resource mounting",
);
assertMatch(
    webMain,
    /function formatGuiFontResourceMountError\(error: GuiFontResourceMountError\): string[\s\S]*case 'FetchUnavailable'[\s\S]*case 'InvalidResourcePath'[\s\S]*case 'NetworkError'[\s\S]*case 'HttpError'[\s\S]*case 'InvalidBytes'[\s\S]*case 'InvalidText'[\s\S]*case 'VfsWriteFailed'/,
    "Web main must display all GUI font resource mount errors through typed branches",
);
assertMatch(
    webPanelManager,
    /beforeWasmExecution\?: \(\) => Promise<string \| null>[\s\S]*this\.beforeWasmExecution = options\.beforeWasmExecution \|\| \(async \(\) => null\)[\s\S]*beforeWasmExecution: this\.beforeWasmExecution/,
    "Panel manager must route wasm execution preflight into CanvasTerminal",
);
assertMatch(
    webTerminal,
    /new Shell\(this, \(options as any\)\.vfs \|\| null, \{[\s\S]*getCompilerMode: \(options as any\)\.getCompilerMode,[\s\S]*beforeWasmExecution: \(options as any\)\.beforeWasmExecution/,
    "Canvas terminal must forward wasm execution preflight into Shell",
);
assertMatch(
    webShell,
    /beforeWasmExecution\?: \(\) => Promise<string \| null>[\s\S]*beforeWasmExecutionOption[\s\S]*if \(request\.type === 'run-wasm'\)[\s\S]*await this\.beforeWasmExecutionOption\(\)[\s\S]*throw new Error\(blockedMessage\)/,
    "Shell must guard every run-wasm worker request, including typed terminal commands",
);
assertMatch(
    webVfs,
    /serializeForCompile\(\): Record<string, string>[\s\S]*this\.readOnlyFiles\.has\(path\)[\s\S]*!path\.endsWith\('\.nepl'\)[\s\S]*typeof content !== 'string'/,
    "Web VFS compile serialization must exclude read-only resources, non-NEPL files, and binary payloads",
);
assertMatch(
    webIndex,
    /<link data-trunk rel="copy-dir" href="src">/,
    "Web index must copy bundled font files under src/fonts for Trunk output",
);
assertMatch(
    webFontResourceBehaviorTest,
    /mountBundledGuiFontResources[\s\S]*serializeForCompile/,
    "Web font resource behavior test must cover mount success, typed failures, rollback, and compile overlay exclusion",
);
for (const behaviorKind of ["HttpError", "InvalidBytes", "InvalidText", "InvalidResourcePath", "VfsWriteFailed"]) {
    assertMatch(
        webFontResourceBehaviorTest,
        new RegExp(`"${behaviorKind}"`),
        `Web font resource behavior test must cover ${behaviorKind}`,
    );
}

console.log("web GUI font rendering contract passed");

