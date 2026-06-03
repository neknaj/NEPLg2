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
    code?: string | null;
    code_message?: string | null;
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
    expression_range?: { start?: number; end?: number } | null;
    arg_index?: number | null;
    arg_span?: { start?: number; end?: number } | null;
    arg_range?: { start?: number; end?: number } | null;
};

export type AnalysisTokenClassification = {
    token_index?: number;
    category?: EditorTokenType | string | null;
    role?: string | null;
    span?: AnalysisSpan | null;
    enclosing_span?: AnalysisSpan | null;
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
        token_classifications?: AnalysisTokenClassification[];
        syntax_ranges?: Array<{
            kind?: string;
            role?: string;
            span?: AnalysisSpan | null;
            inner_span?: AnalysisSpan | null;
        }>;
    } | null;
};

export type EditorTokenType =
    | 'keyword'
    | 'string'
    | 'literal-string'
    | 'literal-char'
    | 'comment'
    | 'function'
    | 'number'
    | 'literal-number'
    | 'boolean'
    | 'literal-bool'
    | 'literal-unit'
    | 'literal-void'
    | 'operator'
    | 'regex'
    | 'property'
    | 'punctuation'
    | 'variable'
    | 'constant'
    | 'namespace'
    | 'type'
    | 'type-constructor'
    | 'heading'
    | 'bold'
    | 'italic'
    | 'list'
    | 'link'
    | 'inline-code'
    | 'code-block'
    | 'default';

export type EditorToken = {
    startIndex: number;
    endIndex: number;
    type: EditorTokenType;
};

export type EditorDiagnostic = {
    startIndex: number;
    endIndex: number;
    code: string | null;
    codeMessage: string | null;
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

type TextDiff = {
    start: number;
    previousEnd: number;
    nextEnd: number;
    delta: number;
    insertedText: string;
    removedText: string;
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
    tokenClassificationByIndex: Map<number, AnalysisTokenClassification>;
};

function analysisArray<T>(value: T[] | null | undefined): T[] {
    return Array.isArray(value) ? value : [];
}

function optionalString(value: unknown): string | null {
    return typeof value === 'string' && value.length > 0 ? value : null;
}

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

function diffTexts(previousText: string, nextText: string): TextDiff | null {
    if (previousText === nextText) {
        return null;
    }
    let start = 0;
    while (start < previousText.length && start < nextText.length && previousText[start] === nextText[start]) {
        start += 1;
    }
    let previousEnd = previousText.length;
    let nextEnd = nextText.length;
    while (previousEnd > start && nextEnd > start && previousText[previousEnd - 1] === nextText[nextEnd - 1]) {
        previousEnd -= 1;
        nextEnd -= 1;
    }
    return {
        start,
        previousEnd,
        nextEnd,
        delta: nextText.length - previousText.length,
        insertedText: nextText.slice(start, nextEnd),
        removedText: previousText.slice(start, previousEnd),
    };
}

function countNewlines(text: string): number {
    let count = 0;
    for (let index = 0; index < text.length; index += 1) {
        if (text.charCodeAt(index) === 10) {
            count += 1;
        }
    }
    return count;
}

function lineInfoAt(offsets: OffsetMaps, textLength: number, index: number): { line: number; lineStart: number; lineEnd: number } {
    let line = 0;
    while (line + 1 < offsets.lineStarts.length && offsets.lineStarts[line + 1] <= index) {
        line += 1;
    }
    return {
        line,
        lineStart: offsets.lineStarts[line],
        lineEnd: line + 1 < offsets.lineStarts.length ? offsets.lineStarts[line + 1] - 1 : textLength,
    };
}

function remapIndex(index: number, affectedStart: number, affectedEnd: number, delta: number): number | null {
    if (!Number.isFinite(index)) {
        return null;
    }
    if (index <= affectedStart) {
        return index;
    }
    if (index >= affectedEnd) {
        return index + delta;
    }
    return null;
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

const OPERATOR_TOKEN_KINDS = new Set([
    'Ampersand',
    'Arrow',
    'Backslash',
    'Equals',
    'Minus',
    'PathSep',
    'Percent',
    'Pipe',
    'Plus',
    'Slash',
    'Star',
]);

const PUNCTUATION_TOKEN_KINDS = new Set([
    'Colon',
    'Comma',
    'Dot',
    'LAngle',
    'LBrace',
    'LBracket',
    'LParen',
    'RAngle',
    'RBrace',
    'RBracket',
    'RParen',
    'Semicolon',
    'UnitLiteral',
]);

const IDENT_KEYWORDS = new Set([
    'as',
    'cond',
    'do',
    'else',
    'fn',
    'for',
    'if',
    'impl',
    'impure',
    'let',
    'match',
    'mut',
    'pub',
    'set',
    'struct',
    'then',
    'trait',
    'while',
]);

const PRIMITIVE_TYPE_NAMES = new Set([
    'bool',
    'char',
    'f32',
    'f64',
    'i8',
    'i16',
    'i32',
    'i64',
    'i128',
    'isize',
    'str',
    'u8',
    'u16',
    'u32',
    'u64',
    'u128',
    'unit',
    'usize',
]);

function isUppercaseIdentifier(value?: string): boolean {
    if (!value || value.length === 0) {
        return false;
    }
    const first = value.codePointAt(0);
    return first != null && first >= 65 && first <= 90;
}

function normalizeTokenType(kind?: string, debug?: string, value?: string): EditorTokenType {
    if (!kind) {
        return 'default';
    }
    if (kind.startsWith('Kw') || kind.startsWith('Dir') || kind === 'At') return 'keyword';
    if (kind === 'VoidMarker') return 'literal-void';
    if (kind === 'UnitLiteral') return 'literal-unit';
    if (kind.includes('String') || kind.includes('Mlstr')) return 'literal-string';
    if (kind.includes('CharLiteral')) return 'literal-char';
    if (kind.includes('BoolLiteral')) return 'literal-bool';
    if (kind.includes('IntLiteral') || kind.includes('FloatLiteral')) return 'literal-number';
    if (kind.includes('Comment')) return 'comment';
    if (kind === 'Ident' && IDENT_KEYWORDS.has(value ?? '')) return 'keyword';
    if (kind === 'Ident' && value === 'void') return 'literal-void';
    if (kind === 'Ident' && value === 'unit') return 'literal-unit';
    if (kind === 'Ident' && (PRIMITIVE_TYPE_NAMES.has(value ?? '') || isUppercaseIdentifier(value))) return 'type';
    if (kind === 'Ident') return 'variable';
    if (OPERATOR_TOKEN_KINDS.has(kind)) return 'operator';
    if (PUNCTUATION_TOKEN_KINDS.has(kind)) return 'punctuation';
    if (debug && String(debug).includes('Fn')) return 'function';
    return 'default';
}

function tokenizeDirectiveSpan(prepared: PreparedLanguageAnalysis, span: NonNullable<ReturnType<typeof spanFromPrepared>>): EditorToken[] {
    const lineEnd = prepared.text.indexOf('\n', span.startIndex);
    const expandedEnd = lineEnd === -1 ? prepared.text.length : lineEnd;
    const text = prepared.text.slice(span.startIndex, Math.max(span.endIndex, expandedEnd));
    const tokens: EditorToken[] = [];
    let offset = 0;

    const push = (start: number, end: number, type: EditorTokenType) => {
        if (end > start) {
            tokens.push({
                startIndex: span.startIndex + start,
                endIndex: span.startIndex + end,
                type,
            });
        }
    };

    while (offset < text.length) {
        const ch = text[offset];
        if (/\s/.test(ch)) {
            offset += 1;
            continue;
        }
        if (ch === '#') {
            let cursor = offset + 1;
            while (cursor < text.length && /[A-Za-z0-9_-]/.test(text[cursor])) {
                cursor += 1;
            }
            push(offset, cursor, 'keyword');
            offset = cursor;
            continue;
        }
        if (ch === '"' || ch === '\'') {
            const quote = ch;
            let cursor = offset + 1;
            while (cursor < text.length) {
                const current = text[cursor];
                if (current === '\\') {
                    cursor += 2;
                    continue;
                }
                cursor += 1;
                if (current === quote) {
                    break;
                }
            }
            push(offset, Math.min(cursor, text.length), 'literal-string');
            offset = Math.min(cursor, text.length);
            continue;
        }
        if (ch === '@') {
            let cursor = offset + 1;
            while (cursor < text.length && /[A-Za-z0-9_-]/.test(text[cursor])) {
                cursor += 1;
            }
            push(offset, cursor, 'keyword');
            offset = cursor;
            continue;
        }
        if (/[0-9]/.test(ch)) {
            let cursor = offset + 1;
            while (cursor < text.length && /[0-9_]/.test(text[cursor])) {
                cursor += 1;
            }
            push(offset, cursor, 'literal-number');
            offset = cursor;
            continue;
        }
        if (/[A-Za-z_]/.test(ch)) {
            let cursor = offset + 1;
            while (cursor < text.length && /[A-Za-z0-9_-]/.test(text[cursor])) {
                cursor += 1;
            }
            const word = text.slice(offset, cursor);
            const type = IDENT_KEYWORDS.has(word)
                ? 'keyword'
                : word === 'void'
                    ? 'literal-void'
                    : word === 'unit'
                        ? 'literal-unit'
                        : PRIMITIVE_TYPE_NAMES.has(word) || isUppercaseIdentifier(word)
                            ? 'type'
                            : 'variable';
            push(offset, cursor, type);
            offset = cursor;
            continue;
        }
        if ('*&|+-/=!%\\'.includes(ch)) {
            push(offset, offset + 1, 'operator');
            offset += 1;
            continue;
        }
        if ('()[]{}:;,.<>'.includes(ch)) {
            push(offset, offset + 1, 'punctuation');
            offset += 1;
            continue;
        }
        offset += 1;
    }

    return tokens;
}

function prepareAnalysis(text: string, snapshot?: LanguageAnalysisSnapshot | null): PreparedLanguageAnalysis {
    const safeSnapshot = snapshot ?? {};
    const definitions = analysisArray(safeSnapshot.resolve?.definitions);
    const definitionById = new Map<number, AnalysisDefinition>();
    for (const definition of definitions) {
        if (Number.isFinite(definition?.id)) {
            definitionById.set(Number(definition?.id), definition as AnalysisDefinition);
        }
    }
    const tokenClassificationByIndex = new Map<number, AnalysisTokenClassification>();
    for (const classification of analysisArray(safeSnapshot.semantics?.token_classifications)) {
        if (Number.isFinite(classification?.token_index)) {
            tokenClassificationByIndex.set(Number(classification?.token_index), classification);
        }
    }
    return {
        text,
        snapshot: safeSnapshot,
        offsets: buildOffsetMaps(text),
        definitionById,
        tokenClassificationByIndex,
    };
}

function tokenResolutionAt(prepared: PreparedLanguageAnalysis, tokenIndex: number): AnalysisTokenResolution | null {
    const resolutions = analysisArray(prepared.snapshot.semantics?.token_resolution);
    return resolutions.find((item) => Number(item?.token_index) === tokenIndex) ?? null;
}

function tokenSemanticAt(prepared: PreparedLanguageAnalysis, tokenIndex: number): AnalysisTokenSemantic | null {
    const semantics = analysisArray(prepared.snapshot.semantics?.token_semantics);
    return semantics.find((item) => Number(item?.token_index) === tokenIndex) ?? null;
}

const EDITOR_TOKEN_TYPE_VALUES = new Set<EditorTokenType>([
    'keyword',
    'string',
    'literal-string',
    'literal-char',
    'comment',
    'function',
    'number',
    'literal-number',
    'boolean',
    'literal-bool',
    'literal-unit',
    'literal-void',
    'operator',
    'regex',
    'property',
    'punctuation',
    'variable',
    'constant',
    'namespace',
    'type',
    'type-constructor',
    'heading',
    'bold',
    'italic',
    'list',
    'link',
    'inline-code',
    'code-block',
    'default',
]);

function normalizeEditorTokenType(value: unknown): EditorTokenType | null {
    return typeof value === 'string' && EDITOR_TOKEN_TYPE_VALUES.has(value as EditorTokenType)
        ? value as EditorTokenType
        : null;
}

function tokenClassificationAt(prepared: PreparedLanguageAnalysis, tokenIndex: number): AnalysisTokenClassification | null {
    return prepared.tokenClassificationByIndex.get(tokenIndex) ?? null;
}

function editorTokenTypeForDefinitionKind(kind?: string | null): EditorTokenType | null {
    switch (kind) {
        case 'fn':
        case 'fn_alias':
            return 'function';
        case 'struct':
        case 'enum':
        case 'trait':
            return 'type';
        case 'let_hoisted':
            return 'constant';
        case 'let_mut':
        case 'param':
        case 'match_bind':
            return 'variable';
        default:
            return null;
    }
}

function tokenAt(prepared: PreparedLanguageAnalysis, index: number): { token: AnalysisToken; tokenIndex: number; span: NonNullable<ReturnType<typeof spanFromPrepared>> } | null {
    const tokens = analysisArray(prepared.snapshot.lex?.tokens);
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
    const references = analysisArray(prepared.snapshot.resolve?.references);
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

function walkAnalysisNodes(node: unknown, visit: (value: AnalysisTreeNode) => void): void {
    if (!node || typeof node !== 'object') {
        return;
    }
    const treeNode = node as AnalysisTreeNode;
    visit(treeNode);
    for (const value of Object.values(treeNode)) {
        if (Array.isArray(value)) {
            for (const entry of value) {
                walkAnalysisNodes(entry, visit);
            }
        } else if (value && typeof value === 'object') {
            walkAnalysisNodes(value, visit);
        }
    }
}

function expressionSpanFromAst(prepared: PreparedLanguageAnalysis, tokenSpan: TokenInsight['tokenSpan']): { start: number; end: number } | null {
    const root = prepared.snapshot.parse?.module?.root;
    if (!root) {
        return null;
    }

    const tokenWidth = tokenSpan.endIndex - tokenSpan.startIndex;
    let best: { start: number; end: number } | null = null;
    let bestWidth = Number.MAX_SAFE_INTEGER;

    walkAnalysisNodes(root, (node) => {
        const span = spanFromPrepared(prepared, node);
        if (!span) {
            return;
        }
        if (span.startIndex !== tokenSpan.startIndex) {
            return;
        }
        if (span.endIndex < tokenSpan.endIndex || span.startIndex > tokenSpan.startIndex || span.endIndex <= span.startIndex) {
            return;
        }
        const width = span.endIndex - span.startIndex;
        if (width < tokenWidth) {
            return;
        }
        if (width > tokenWidth && width < bestWidth) {
            best = { start: span.startIndex, end: span.endIndex };
            bestWidth = width;
        }
    });

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
                code: optionalString(item?.code),
                codeMessage: optionalString(item?.code_message),
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
    const tokens = analysisArray(prepared.snapshot.lex?.tokens);
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

        if (kind.startsWith('Dir')) {
            output.push(...tokenizeDirectiveSpan(prepared, span));
            continue;
        }

        let type = normalizeTokenType(kind, token?.debug, typeof token?.value === 'string' ? token.value : undefined);
        const classification = tokenClassificationAt(prepared, tokenIndex);
        const classifiedType = normalizeEditorTokenType(classification?.category);
        if (classifiedType) {
            type = classifiedType;
        } else {
            const resolution = tokenResolutionAt(prepared, tokenIndex);
            if (resolution?.resolved_def_id != null) {
                const definition = prepared.definitionById.get(Number(resolution.resolved_def_id));
                const resolvedType = editorTokenTypeForDefinitionKind(definition?.kind);
                if (resolvedType) {
                    type = resolvedType;
                }
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
    const semantics = analysisArray(prepared.snapshot.semantics?.token_semantics);
    const output: EditorSemanticToken[] = [];
    for (const item of semantics) {
        const exprSpan = item?.expr_span ?? item?.expression_range ?? null;
        const argSpan = item?.arg_span ?? item?.arg_range ?? null;
        if (!exprSpan) {
            continue;
        }
        output.push({
            tokenIndex: Number(item.token_index ?? -1),
            inferredType: item.inferred_type ?? null,
            exprSpan: {
                start: Number(exprSpan.start ?? 0),
                end: Number(exprSpan.end ?? 0),
            },
            argIndex: Number.isInteger(item.arg_index) ? Number(item.arg_index) : null,
            argSpan: argSpan
                ? {
                    start: Number(argSpan.start ?? 0),
                    end: Number(argSpan.end ?? 0),
                }
                : null,
        });
    }
    return output;
}

function buildInlayHints(prepared: PreparedLanguageAnalysis): EditorInlayHint[] {
    const semantics = analysisArray(prepared.snapshot.semantics?.token_semantics);
    const output: EditorInlayHint[] = [];
    for (const item of semantics) {
        const exprSpan = item?.expr_span ?? item?.expression_range ?? null;
        if (!exprSpan || !item?.inferred_type) {
            continue;
        }
        const start = Number(exprSpan.start ?? -1);
        if (start < 0) {
            continue;
        }
        output.push({
            kind: 'type',
            position: start,
            label: `<${item.inferred_type}>`,
            exprSpan: {
                start: Number(exprSpan.start ?? 0),
                end: Number(exprSpan.end ?? 0),
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

export function remapEditorUpdatePayloadForTextChange(previousText: string, nextText: string, previousPayload?: EditorUpdatePayload | null): EditorUpdatePayload | null {
    if (!previousPayload) {
        return null;
    }
    const diff = diffTexts(previousText, nextText);
    if (!diff) {
        return previousPayload;
    }

    const previousOffsets = buildOffsetMaps(previousText);
    const nextOffsets = buildOffsetMaps(nextText);
    const previousStartLine = lineInfoAt(previousOffsets, previousText.length, diff.start);
    const previousEndLine = lineInfoAt(previousOffsets, previousText.length, diff.previousEnd);
    const nextEndLine = lineInfoAt(nextOffsets, nextText.length, diff.nextEnd);
    const affectedPreviousStart = previousStartLine.lineStart;
    const affectedPreviousEnd = previousEndLine.lineEnd;
    const lineDelta = countNewlines(diff.insertedText) - countNewlines(diff.removedText);

    const remapRange = <T extends { startIndex: number; endIndex: number }>(range: T): T | null => {
        if (range.endIndex <= affectedPreviousStart) {
            return { ...range };
        }
        if (range.startIndex >= affectedPreviousEnd) {
            return {
                ...range,
                startIndex: range.startIndex + diff.delta,
                endIndex: range.endIndex + diff.delta,
            };
        }
        return null;
    };

    const tokens = (previousPayload.tokens || [])
        .map((token) => remapRange(token))
        .filter((token): token is EditorToken => Boolean(token))
        .sort((left, right) => left.startIndex - right.startIndex || left.endIndex - right.endIndex);

    const diagnostics = (previousPayload.diagnostics || [])
        .map((diagnostic) => remapRange(diagnostic))
        .filter((diagnostic): diagnostic is EditorDiagnostic => Boolean(diagnostic))
        .sort((left, right) => left.startIndex - right.startIndex || left.endIndex - right.endIndex);

    const foldingRanges = (previousPayload.foldingRanges || [])
        .flatMap((range) => {
            if (range.endLine < previousStartLine.line) {
                return [{ ...range }];
            }
            if (range.startLine > previousEndLine.line) {
                return [{
                    ...range,
                    startLine: range.startLine + lineDelta,
                    endLine: range.endLine + lineDelta,
                }];
            }
            return [];
        })
        .sort((left, right) => left.startLine - right.startLine || left.endLine - right.endLine);

    const semanticTokens = (previousPayload.semanticTokens || [])
        .flatMap((token) => {
            const exprStart = remapIndex(token.exprSpan.start, affectedPreviousStart, affectedPreviousEnd, diff.delta);
            const exprEnd = remapIndex(token.exprSpan.end, affectedPreviousStart, affectedPreviousEnd, diff.delta);
            if (exprStart == null || exprEnd == null) {
                return [];
            }
            const argStart = token.argSpan ? remapIndex(token.argSpan.start, affectedPreviousStart, affectedPreviousEnd, diff.delta) : null;
            const argEnd = token.argSpan ? remapIndex(token.argSpan.end, affectedPreviousStart, affectedPreviousEnd, diff.delta) : null;
            return [{
                ...token,
                exprSpan: { start: exprStart, end: exprEnd },
                argSpan: argStart != null && argEnd != null ? { start: argStart, end: argEnd } : null,
            }];
        });

    const inlayHints = (previousPayload.inlayHints || [])
        .flatMap((hint) => {
            const position = remapIndex(hint.position, affectedPreviousStart, affectedPreviousEnd, diff.delta);
            const exprStart = remapIndex(hint.exprSpan.start, affectedPreviousStart, affectedPreviousEnd, diff.delta);
            const exprEnd = remapIndex(hint.exprSpan.end, affectedPreviousStart, affectedPreviousEnd, diff.delta);
            if (position == null || exprStart == null || exprEnd == null) {
                return [];
            }
            return [{
                ...hint,
                position,
                exprSpan: { start: exprStart, end: exprEnd },
            }];
        });

    return {
        ...previousPayload,
        tokens,
        diagnostics,
        foldingRanges,
        semanticTokens,
        inlayHints,
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
    const exprSpan = semantic?.expr_span ?? semantic?.expression_range ?? null;
    const argSpan = semantic?.arg_span ?? semantic?.arg_range ?? null;

    return {
        tokenIndex: hit.tokenIndex,
        tokenKind: String(hit.token?.kind ?? ''),
        tokenSpan: hit.span,
        inferredType: semantic?.inferred_type ?? null,
        exprSpan,
        argIndex: Number.isInteger(semantic?.arg_index) ? Number(semantic?.arg_index) : null,
        argSpan,
        resolvedDefId: resolution?.resolved_def_id != null ? Number(resolution.resolved_def_id) : null,
        candidateDefIds: analysisArray(resolution?.candidate_def_ids).map((value) => Number(value)),
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

function formatHoverExpression(text: string, span?: { start?: number; end?: number } | null): string | null {
    if (!span) {
        return null;
    }
    const start = Math.max(0, Math.min(text.length, Math.trunc(Number(span.start ?? 0))));
    const end = Math.max(start, Math.min(text.length, Math.trunc(Number(span.end ?? start))));
    const snippet = text.slice(start, end).replace(/\s+/g, ' ').trim();
    if (!snippet) {
        return null;
    }
    if (snippet.length <= 160) {
        return snippet;
    }
    return `${snippet.slice(0, 157)}...`;
}

function resolveHoverExpressionSpan(prepared: PreparedLanguageAnalysis, insight: TokenInsight): { start: number; end: number } | null {
    const semanticStart = Number(insight.exprSpan?.start);
    const semanticEnd = Number(insight.exprSpan?.end);
    const semanticSpan = Number.isFinite(semanticStart) && Number.isFinite(semanticEnd) && semanticEnd > semanticStart
        ? { start: semanticStart, end: semanticEnd }
        : null;
    const astSpan = expressionSpanFromAst(prepared, insight.tokenSpan);
    if (!semanticSpan) {
        return astSpan;
    }
    if (!astSpan) {
        return semanticSpan;
    }
    if (astSpan.start === semanticSpan.start && astSpan.end > semanticSpan.end) {
        return astSpan;
    }
    if (semanticSpan.start === insight.tokenSpan.startIndex && semanticSpan.end <= insight.tokenSpan.endIndex) {
        return astSpan;
    }
    return semanticSpan;
}

export function getHoverInfoFromAnalysis(text: string, snapshot: LanguageAnalysisSnapshot | null | undefined, index: number): HoverInfo | null {
    const prepared = prepareAnalysis(text, snapshot);
    const insight = getTokenInsightFromAnalysis(text, snapshot, index);
    if (!insight) {
        return null;
    }

    const lines: string[] = [];
    const expressionSpan = resolveHoverExpressionSpan(prepared, insight);
    const expression = formatHoverExpression(text, expressionSpan);
    if (expression) {
        lines.push(`expr: ${expression}`);
    }
    if (insight.inferredType) {
        lines.push(`type: ${insight.inferredType}`);
    }
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
    return null;
}

export function getOccurrencesFromAnalysis(text: string, snapshot: LanguageAnalysisSnapshot | null | undefined, index: number): Occurrence[] {
    const prepared = prepareAnalysis(text, snapshot);
    const insight = getTokenInsightFromAnalysis(text, snapshot, index);
    if (!insight) {
        return [];
    }

    const references = analysisArray(prepared.snapshot.resolve?.references);
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
    remapEditorUpdatePayloadForTextChange,
    getTokenInsightFromAnalysis,
    getHoverInfoFromAnalysis,
    getDefinitionLocationFromAnalysis,
    getOccurrencesFromAnalysis,
};

if (typeof window !== 'undefined') {
    window.NEPLPlaygroundLanguageAnalysis = bridge;
}

export { bridge as NEPLPlaygroundLanguageAnalysis };
