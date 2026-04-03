export type AnalysisSpan = {
    start?: number;
    end?: number;
    start_line?: number;
    start_col?: number;
    end_line?: number;
    end_col?: number;
};

export type AnalysisDiagnostic = {
    span?: AnalysisSpan;
    severity?: string;
    message?: string;
    stage?: string;
};

export type AnalysisToken = {
    kind?: string;
    debug?: string;
    value?: string;
    span?: AnalysisSpan;
};

export type AnalysisDefinition = {
    id?: number;
    name?: string;
    kind?: string;
    span?: AnalysisSpan;
};

export type AnalysisReference = {
    name?: string;
    span?: AnalysisSpan;
    resolved_def_id?: number | null;
    candidate_def_ids?: number[];
};

export type AnalysisTokenResolution = {
    token_index?: number;
    name?: string;
    span?: AnalysisSpan;
    resolved_def_id?: number | null;
    candidate_def_ids?: number[];
};

export type AnalysisTokenSemantic = {
    token_index?: number;
    inferred_type?: string | null;
    expr_span?: { start?: number; end?: number } | null;
    arg_index?: number | null;
    arg_span?: { start?: number; end?: number } | null;
};

export type AnalysisTreeNode = {
    kind?: string;
    span?: AnalysisSpan;
    [key: string]: unknown;
};

export type LanguageAnalysisSnapshot = {
    lex?: {
        tokens?: AnalysisToken[];
        diagnostics?: AnalysisDiagnostic[];
    } | null;
    parse?: {
        module?: { root?: AnalysisTreeNode | null } | null;
        diagnostics?: AnalysisDiagnostic[];
        lex_diagnostics?: AnalysisDiagnostic[];
    } | null;
    resolve?: {
        definitions?: AnalysisDefinition[];
        references?: AnalysisReference[];
        diagnostics?: AnalysisDiagnostic[];
        by_name?: Record<string, unknown>;
    } | null;
    semantics?: {
        diagnostics?: AnalysisDiagnostic[];
        token_resolution?: AnalysisTokenResolution[];
        token_semantics?: AnalysisTokenSemantic[];
    } | null;
};

export type EditorToken = {
    startIndex: number;
    endIndex: number;
    type: string;
};

export type EditorDiagnostic = {
    startIndex: number;
    endIndex: number;
    message: string;
    severity: 'error' | 'warning';
};

export type EditorFoldingRange = {
    startLine: number;
    endLine: number;
    placeholder: string;
};

export type EditorSemanticToken = {
    tokenIndex: number;
    inferredType: string | null;
    exprSpan: { start: number; end: number };
    argIndex: number | null;
    argSpan: { start: number; end: number } | null;
};

export type EditorInlayHint = {
    kind: 'type';
    position: number;
    label: string;
    exprSpan: { start: number; end: number };
};

export type EditorUpdatePayload = {
    tokens: EditorToken[];
    diagnostics: EditorDiagnostic[];
    foldingRanges: EditorFoldingRange[];
    semanticTokens: EditorSemanticToken[];
    inlayHints: EditorInlayHint[];
    config: {
        highlightWhitespace: boolean;
        highlightIndent: boolean;
    };
};

export type DefinitionCandidate = {
    id?: number;
    name?: string;
    kind?: string;
    span?: AnalysisSpan | null;
};

export type TokenInsight = {
    tokenIndex: number;
    tokenKind: string;
    tokenSpan: {
        startIndex: number;
        endIndex: number;
        startLine: number;
        startCol: number;
        endLine: number;
        endCol: number;
    };
    inferredType: string | null;
    exprSpan: { start?: number; end?: number } | null;
    argIndex: number | null;
    argSpan: { start?: number; end?: number } | null;
    resolvedDefId: number | null;
    candidateDefIds: number[];
    definitionCandidates: DefinitionCandidate[];
    resolvedDefinition: DefinitionCandidate | null;
};

export type HoverInfo = {
    content: string;
    startIndex: number;
    endIndex: number;
};

export type DefinitionLocation = {
    targetIndex: number;
};

export type Occurrence = {
    startIndex: number;
    endIndex: number;
};

type OffsetMaps = {
    lineStarts: number[];
    byteOffsets: number[];
};

type PreparedLanguageAnalysis = {
    text: string;
    snapshot: LanguageAnalysisSnapshot;
    offsets: OffsetMaps;
    definitionById: Map<number, AnalysisDefinition>;
};

function buildOffsetMaps(text: string): OffsetMaps {
    const lineStarts = [0];
    const byteOffsets = new Array<number>(text.length + 1);
    byteOffsets[0] = 0;

    let index = 0;
    let bytes = 0;
    while (index < text.length) {
        const codePoint = text.codePointAt(index) ?? 0;
        const charLength = codePoint > 0xffff ? 2 : 1;
        if (codePoint <= 0x7f) bytes += 1;
        else if (codePoint <= 0x7ff) bytes += 2;
        else if (codePoint <= 0xffff) bytes += 3;
        else bytes += 4;

        const nextIndex = index + charLength;
        for (let cursor = index + 1; cursor <= nextIndex && cursor <= text.length; cursor += 1) {
            byteOffsets[cursor] = bytes;
        }
        if (codePoint === 10) {
            lineStarts.push(nextIndex);
        }
        index = nextIndex;
    }
    for (let cursor = 0; cursor <= text.length; cursor += 1) {
        if (!Number.isFinite(byteOffsets[cursor])) {
            byteOffsets[cursor] = bytes;
        }
    }

    return { lineStarts, byteOffsets };
}

function lineColToIndex(text: string, offsets: OffsetMaps, line?: number, col?: number): number | null {
    if (!Number.isFinite(line) || !Number.isFinite(col) || (line ?? 0) < 0 || (col ?? 0) < 0) {
        return null;
    }
    const lineIndex = Math.trunc(line ?? 0);
    const colIndex = Math.trunc(col ?? 0);
    if (lineIndex >= offsets.lineStarts.length) {
        return null;
    }

    const start = offsets.lineStarts[lineIndex];
    const lineEnd = lineIndex + 1 < offsets.lineStarts.length ? offsets.lineStarts[lineIndex + 1] - 1 : text.length;
    let index = start;
    let remaining = colIndex;
    while (index < lineEnd && remaining > 0) {
        const codePoint = text.codePointAt(index) ?? 0;
        index += codePoint > 0xffff ? 2 : 1;
        remaining -= 1;
    }
    return Math.max(0, Math.min(text.length, index));
}

function byteOffsetToIndex(offsets: OffsetMaps, byteOffset?: number): number {
    const value = Number(byteOffset ?? 0);
    if (!Number.isFinite(value) || value <= 0) {
        return 0;
    }
    let low = 0;
    let high = offsets.byteOffsets.length - 1;
    while (low < high) {
        const mid = Math.floor((low + high) / 2);
        if (offsets.byteOffsets[mid] < value) {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    if (offsets.byteOffsets[low] === value) {
        return low;
    }
    return Math.max(0, low - 1);
}

function unwrapSpan(source?: { span?: AnalysisSpan } | AnalysisSpan | null): AnalysisSpan | null {
    if (!source) {
        return null;
    }
    const maybeWrapped = source as { span?: AnalysisSpan };
    if (Object.prototype.hasOwnProperty.call(maybeWrapped, 'span')) {
        return maybeWrapped.span ?? null;
    }
    return source as AnalysisSpan;
}

function spanFromPrepared(prepared: PreparedLanguageAnalysis, source?: { span?: AnalysisSpan } | AnalysisSpan | null): {
    startIndex: number;
    endIndex: number;
    startLine: number;
    startCol: number;
    endLine: number;
    endCol: number;
} | null {
    const span = unwrapSpan(source);
    if (!span) {
        return null;
    }

    const fromLineColStart = lineColToIndex(prepared.text, prepared.offsets, span.start_line, span.start_col);
    const fromLineColEnd = lineColToIndex(prepared.text, prepared.offsets, span.end_line, span.end_col);
    const startIndex = fromLineColStart ?? byteOffsetToIndex(prepared.offsets, span.start);
    const endIndex = fromLineColEnd ?? byteOffsetToIndex(prepared.offsets, span.end);

    return {
        startIndex,
        endIndex,
        startLine: Number(span.start_line ?? 0),
        startCol: Number(span.start_col ?? 0),
        endLine: Number(span.end_line ?? 0),
        endCol: Number(span.end_col ?? 0),
    };
}

function normalizeSeverity(severity?: string): 'error' | 'warning' {
    return String(severity ?? 'error').toLowerCase().includes('warn') ? 'warning' : 'error';
}

function normalizeTokenType(kind?: string, debug?: string): string {
    if (!kind) {
        return 'default';
    }
    if (kind.startsWith('Kw') || kind === 'At' || kind === 'PathSep') return 'keyword';
    if (kind.includes('String') || kind.includes('Mlstr')) return 'string';
    if (kind.includes('BoolLiteral')) return 'boolean';
    if (kind.includes('IntLiteral') || kind.includes('FloatLiteral')) return 'number';
    if (kind.includes('Comment')) return 'comment';
    if (kind === 'Ident') return 'variable';
    if (kind === 'Pipe' || kind === 'Arrow' || kind === 'Plus' || kind === 'Minus' || kind === 'Star' || kind === 'Slash' || kind === 'Equals') return 'operator';
    if (kind === 'LParen' || kind === 'RParen' || kind === 'LAngle' || kind === 'RAngle' || kind === 'Colon' || kind === 'Semicolon' || kind === 'Comma' || kind === 'Dot') return 'punctuation';
    if (debug && String(debug).includes('Fn')) return 'function';
    return 'default';
}

function prepareAnalysis(text: string, snapshot?: LanguageAnalysisSnapshot | null): PreparedLanguageAnalysis {
    const safeSnapshot = snapshot ?? {};
    const definitions = Array.isArray(safeSnapshot.resolve?.definitions) ? safeSnapshot.resolve?.definitions : [];
    const definitionById = new Map<number, AnalysisDefinition>();
    for (const definition of definitions) {
        if (Number.isFinite(definition?.id)) {
            definitionById.set(Number(definition?.id), definition as AnalysisDefinition);
        }
    }
    return {
        text,
        snapshot: safeSnapshot,
        offsets: buildOffsetMaps(text),
        definitionById,
    };
}

function tokenResolutionAt(prepared: PreparedLanguageAnalysis, tokenIndex: number): AnalysisTokenResolution | null {
    const resolutions = Array.isArray(prepared.snapshot.semantics?.token_resolution)
        ? prepared.snapshot.semantics?.token_resolution
        : [];
    return resolutions.find((item) => Number(item?.token_index) === tokenIndex) ?? null;
}

function tokenSemanticAt(prepared: PreparedLanguageAnalysis, tokenIndex: number): AnalysisTokenSemantic | null {
    const semantics = Array.isArray(prepared.snapshot.semantics?.token_semantics)
        ? prepared.snapshot.semantics?.token_semantics
        : [];
    return semantics.find((item) => Number(item?.token_index) === tokenIndex) ?? null;
}

function tokenAt(prepared: PreparedLanguageAnalysis, index: number): { token: AnalysisToken; tokenIndex: number; span: NonNullable<ReturnType<typeof spanFromPrepared>> } | null {
    const tokens = Array.isArray(prepared.snapshot.lex?.tokens) ? prepared.snapshot.lex?.tokens : [];
    for (let tokenIndex = 0; tokenIndex < tokens.length; tokenIndex += 1) {
        const token = tokens[tokenIndex];
        const span = spanFromPrepared(prepared, token);
        if (span && index >= span.startIndex && index < span.endIndex) {
            return { token, tokenIndex, span };
        }
    }
    return null;
}

function referenceAt(prepared: PreparedLanguageAnalysis, index: number): AnalysisReference | null {
    const references = Array.isArray(prepared.snapshot.resolve?.references) ? prepared.snapshot.resolve?.references : [];
    let best: AnalysisReference | null = null;
    let bestWidth = Number.MAX_SAFE_INTEGER;
    for (const reference of references) {
        const span = spanFromPrepared(prepared, { span: reference?.span });
        if (!span) {
            continue;
        }
        if (index >= span.startIndex && index < span.endIndex) {
            const width = span.endIndex - span.startIndex;
            if (width < bestWidth) {
                best = reference;
                bestWidth = width;
            }
        }
    }
    return best;
}

function definitionCandidates(prepared: PreparedLanguageAnalysis, resolution: AnalysisTokenResolution | null): DefinitionCandidate[] {
    if (!resolution || !Array.isArray(resolution.candidate_def_ids)) {
        return [];
    }
    return resolution.candidate_def_ids
        .map((id) => prepared.definitionById.get(Number(id)))
        .filter((value): value is AnalysisDefinition => Boolean(value))
        .map((definition) => ({
            id: definition.id,
            name: definition.name,
            kind: definition.kind,
            span: definition.span ?? null,
        }));
}

function collectDiagnostics(prepared: PreparedLanguageAnalysis): EditorDiagnostic[] {
    const output: EditorDiagnostic[] = [];
    const pushFrom = (items?: AnalysisDiagnostic[]) => {
        if (!Array.isArray(items)) {
            return;
        }
        for (const item of items) {
            const span = spanFromPrepared(prepared, item);
            output.push({
                startIndex: span ? span.startIndex : 0,
                endIndex: span ? span.endIndex : 0,
                message: String(item?.message ?? 'diagnostic'),
                severity: normalizeSeverity(item?.severity),
            });
        }
    };

    pushFrom(prepared.snapshot.lex?.diagnostics);
    pushFrom(prepared.snapshot.parse?.diagnostics);
    pushFrom(prepared.snapshot.parse?.lex_diagnostics);
    pushFrom(prepared.snapshot.resolve?.diagnostics);
    pushFrom(prepared.snapshot.semantics?.diagnostics);

    output.sort((left, right) => left.startIndex - right.startIndex || left.endIndex - right.endIndex);
    return output;
}

function buildEditorTokens(prepared: PreparedLanguageAnalysis): EditorToken[] {
    const tokens = Array.isArray(prepared.snapshot.lex?.tokens) ? prepared.snapshot.lex?.tokens : [];
    const output: EditorToken[] = [];
    const skipKinds = new Set(['Indent', 'Dedent', 'Eof', 'Newline']);

    for (let tokenIndex = 0; tokenIndex < tokens.length; tokenIndex += 1) {
        const token = tokens[tokenIndex];
        const kind = String(token?.kind ?? '');
        if (skipKinds.has(kind)) {
            continue;
        }
        const span = spanFromPrepared(prepared, token);
        if (!span || span.endIndex <= span.startIndex) {
            continue;
        }

        let type = normalizeTokenType(kind, token?.debug);
        const resolution = tokenResolutionAt(prepared, tokenIndex);
        if (resolution?.resolved_def_id != null) {
            const definition = prepared.definitionById.get(Number(resolution.resolved_def_id));
            if (definition && (definition.kind === 'fn' || definition.kind === 'fn_alias')) {
                type = 'function';
            }
        }

        output.push({
            startIndex: span.startIndex,
            endIndex: span.endIndex,
            type,
        });
    }

    output.sort((left, right) => left.startIndex - right.startIndex || left.endIndex - right.endIndex);
    return output;
}

function buildSemanticTokens(prepared: PreparedLanguageAnalysis): EditorSemanticToken[] {
    const semantics = Array.isArray(prepared.snapshot.semantics?.token_semantics)
        ? prepared.snapshot.semantics?.token_semantics
        : [];
    const output: EditorSemanticToken[] = [];
    for (const item of semantics) {
        if (!item?.expr_span) {
            continue;
        }
        output.push({
            tokenIndex: Number(item.token_index ?? -1),
            inferredType: item.inferred_type ?? null,
            exprSpan: {
                start: Number(item.expr_span.start ?? 0),
                end: Number(item.expr_span.end ?? 0),
            },
            argIndex: Number.isInteger(item.arg_index) ? Number(item.arg_index) : null,
            argSpan: item.arg_span
                ? {
                    start: Number(item.arg_span.start ?? 0),
                    end: Number(item.arg_span.end ?? 0),
                }
                : null,
        });
    }
    return output;
}

function buildInlayHints(prepared: PreparedLanguageAnalysis): EditorInlayHint[] {
    const semantics = Array.isArray(prepared.snapshot.semantics?.token_semantics)
        ? prepared.snapshot.semantics?.token_semantics
        : [];
    const output: EditorInlayHint[] = [];
    for (const item of semantics) {
        if (!item?.expr_span || !item?.inferred_type) {
            continue;
        }
        const start = Number(item.expr_span.start ?? -1);
        if (start < 0) {
            continue;
        }
        output.push({
            kind: 'type',
            position: start,
            label: `<${item.inferred_type}>`,
            exprSpan: {
                start: Number(item.expr_span.start ?? 0),
                end: Number(item.expr_span.end ?? 0),
            },
        });
    }
    return output;
}

function walkAstRanges(node: unknown, output: EditorFoldingRange[]): void {
    if (!node || typeof node !== 'object') {
        return;
    }
    const treeNode = node as AnalysisTreeNode;
    if (treeNode.kind === 'Block' && treeNode.span && Number(treeNode.span.end_line ?? 0) > Number(treeNode.span.start_line ?? 0)) {
        output.push({
            startLine: Number(treeNode.span.start_line ?? 0),
            endLine: Number(treeNode.span.end_line ?? 0),
            placeholder: '...',
        });
    }
    for (const value of Object.values(treeNode)) {
        if (Array.isArray(value)) {
            for (const entry of value) {
                walkAstRanges(entry, output);
            }
        } else if (value && typeof value === 'object') {
            walkAstRanges(value, output);
        }
    }
}

function buildFoldingRanges(prepared: PreparedLanguageAnalysis): EditorFoldingRange[] {
    const root = prepared.snapshot.parse?.module?.root;
    if (!root) {
        return [];
    }
    const output: EditorFoldingRange[] = [];
    walkAstRanges(root, output);
    output.sort((left, right) => left.startLine - right.startLine || left.endLine - right.endLine);
    return output;
}

export function buildEditorUpdatePayloadFromAnalysis(text: string, snapshot?: LanguageAnalysisSnapshot | null): EditorUpdatePayload {
    const prepared = prepareAnalysis(text, snapshot);
    return {
        tokens: buildEditorTokens(prepared),
        diagnostics: collectDiagnostics(prepared),
        foldingRanges: buildFoldingRanges(prepared),
        semanticTokens: buildSemanticTokens(prepared),
        inlayHints: buildInlayHints(prepared),
        config: {
            highlightWhitespace: false,
            highlightIndent: true,
        },
    };
}

export function getTokenInsightFromAnalysis(text: string, snapshot: LanguageAnalysisSnapshot | null | undefined, index: number): TokenInsight | null {
    const prepared = prepareAnalysis(text, snapshot);
    const hit = tokenAt(prepared, index);
    if (!hit) {
        return null;
    }

    const semantic = tokenSemanticAt(prepared, hit.tokenIndex);
    const resolution = tokenResolutionAt(prepared, hit.tokenIndex);
    const definition = resolution?.resolved_def_id != null ? prepared.definitionById.get(Number(resolution.resolved_def_id)) : null;
    const candidates = definitionCandidates(prepared, resolution);

    return {
        tokenIndex: hit.tokenIndex,
        tokenKind: String(hit.token?.kind ?? ''),
        tokenSpan: hit.span,
        inferredType: semantic?.inferred_type ?? null,
        exprSpan: semantic?.expr_span ?? null,
        argIndex: Number.isInteger(semantic?.arg_index) ? Number(semantic?.arg_index) : null,
        argSpan: semantic?.arg_span ?? null,
        resolvedDefId: resolution?.resolved_def_id != null ? Number(resolution.resolved_def_id) : null,
        candidateDefIds: Array.isArray(resolution?.candidate_def_ids) ? resolution.candidate_def_ids.map((value) => Number(value)) : [],
        definitionCandidates: candidates,
        resolvedDefinition: definition
            ? {
                id: definition.id,
                name: definition.name,
                kind: definition.kind,
                span: definition.span ?? null,
            }
            : null,
    };
}

function formatRange(span?: { start?: number; end?: number } | null): string | null {
    if (!span) {
        return null;
    }
    return `[${Number(span.start ?? 0)}, ${Number(span.end ?? 0)})`;
}

export function getHoverInfoFromAnalysis(text: string, snapshot: LanguageAnalysisSnapshot | null | undefined, index: number): HoverInfo | null {
    const prepared = prepareAnalysis(text, snapshot);
    const insight = getTokenInsightFromAnalysis(text, snapshot, index);
    if (!insight) {
        const fallback = referenceAt(prepared, index);
        if (!fallback) {
            return null;
        }
        const definition = fallback.resolved_def_id != null ? prepared.definitionById.get(Number(fallback.resolved_def_id)) : null;
        const lines = [String(fallback.name ?? '')].filter(Boolean);
        if (definition) {
            lines.push(`def: ${definition.kind ?? ''} ${definition.name ?? ''}`.trim());
        }
        const span = spanFromPrepared(prepared, { span: fallback.span });
        return {
            content: lines.join('\n'),
            startIndex: span ? span.startIndex : index,
            endIndex: span ? span.endIndex : index + 1,
        };
    }

    const lines: string[] = [];
    const hit = tokenAt(prepared, index);
    const rawFromToken = String(hit?.token?.value ?? hit?.token?.debug ?? '').trim();
    const rawFromSource = hit?.span ? text.slice(hit.span.startIndex, hit.span.endIndex).trim() : '';
    const raw = rawFromToken || rawFromSource || insight.tokenKind;
    if (raw) lines.push(raw);
    if (insight.inferredType) lines.push(`type: ${insight.inferredType}`);
    const exprRange = formatRange(insight.exprSpan);
    if (exprRange) lines.push(`expr: ${exprRange}`);
    if (Number.isInteger(insight.argIndex)) {
        const argRange = formatRange(insight.argSpan);
        lines.push(`arg#${insight.argIndex}: ${argRange ?? '[0, 0)'}`);
    }
    if (insight.resolvedDefinition) {
        lines.push(`def: ${insight.resolvedDefinition.kind ?? ''} ${insight.resolvedDefinition.name ?? ''}`.trim());
    }
    if (insight.definitionCandidates.length > 1) {
        lines.push(`candidates: ${insight.definitionCandidates.map((item) => `${item.id}:${item.name}`).join(', ')}`);
    }
    if (lines.length === 0) {
        return null;
    }

    return {
        content: lines.join('\n'),
        startIndex: insight.tokenSpan.startIndex,
        endIndex: insight.tokenSpan.endIndex,
    };
}

export function getDefinitionLocationFromAnalysis(text: string, snapshot: LanguageAnalysisSnapshot | null | undefined, index: number): DefinitionLocation | null {
    const prepared = prepareAnalysis(text, snapshot);
    const insight = getTokenInsightFromAnalysis(text, snapshot, index);
    if (insight?.resolvedDefinition?.span) {
        const span = spanFromPrepared(prepared, { span: insight.resolvedDefinition.span });
        if (span) {
            return { targetIndex: span.startIndex };
        }
    }

    const fallback = referenceAt(prepared, index);
    if (fallback?.resolved_def_id != null) {
        const definition = prepared.definitionById.get(Number(fallback.resolved_def_id));
        if (definition?.span) {
            const span = spanFromPrepared(prepared, { span: definition.span });
            if (span) {
                return { targetIndex: span.startIndex };
            }
        }
    }
    return null;
}

export function getOccurrencesFromAnalysis(text: string, snapshot: LanguageAnalysisSnapshot | null | undefined, index: number): Occurrence[] {
    const prepared = prepareAnalysis(text, snapshot);
    const insight = getTokenInsightFromAnalysis(text, snapshot, index);
    if (!insight) {
        return [];
    }

    const references = Array.isArray(prepared.snapshot.resolve?.references) ? prepared.snapshot.resolve?.references : [];
    const output: Occurrence[] = [];
    for (const reference of references) {
        if (insight.resolvedDefId != null && reference?.resolved_def_id === insight.resolvedDefId) {
            const span = spanFromPrepared(prepared, { span: reference.span });
            if (span) {
                output.push({ startIndex: span.startIndex, endIndex: span.endIndex });
            }
        }
    }
    if (output.length > 0) {
        return output;
    }

    const resolution = tokenResolutionAt(prepared, insight.tokenIndex);
    if (!resolution?.name) {
        return [];
    }
    for (const reference of references) {
        if (reference?.name === resolution.name) {
            const span = spanFromPrepared(prepared, { span: reference.span });
            if (span) {
                output.push({ startIndex: span.startIndex, endIndex: span.endIndex });
            }
        }
    }
    return output;
}

const bridge = {
    buildEditorUpdatePayloadFromAnalysis,
    getTokenInsightFromAnalysis,
    getHoverInfoFromAnalysis,
    getDefinitionLocationFromAnalysis,
    getOccurrencesFromAnalysis,
};

if (typeof window !== 'undefined') {
    window.NEPLPlaygroundLanguageAnalysis = bridge;
}

export { bridge as NEPLPlaygroundLanguageAnalysis };
