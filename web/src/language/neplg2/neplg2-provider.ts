// @ts-nocheck

class NEPLg2LanguageProvider {
    constructor() {
        this.updateCallback = () => {};
        this.text = '';
        this.lex = null;
        this.parse = null;
        this.resolve = null;
        this.semantics = null;
        this.analysisVersion = 0;
        this.pendingTimer = null;
        this.analyzeDelayMs = 80;
        this.lastUpdatePayload = null;
        this.definitionById = new Map();
        this.keywordCompletions = [
            'fn', 'let', 'mut', 'set', 'if', 'while', 'cond', 'then', 'else', 'do',
            'block', 'return', 'break', 'match', 'trait', 'impl', 'for', 'enum', 'struct',
            '#entry', '#target', '#indent', '#import', '#use',
        ];
        this.lineStarts = [0];
        this.byteOffsets = [0];
    }

    onUpdate(callback) {
        this.updateCallback = callback || (() => {});
    }

    updateText(text) {
        this.text = text || '';
        this._rebuildOffsetMaps();
        this.analysisVersion += 1;
        if (this.pendingTimer != null) {
            clearTimeout(this.pendingTimer);
            this.pendingTimer = null;
        }
        const version = this.analysisVersion;
        this.pendingTimer = setTimeout(() => {
            this.pendingTimer = null;
            this._analyzeAndPublish(version);
        }, this.analyzeDelayMs);
    }

    _wasm() {
        return window.wasmBindings || null;
    }

    _rebuildOffsetMaps() {
        const s = this.text || '';
        this.lineStarts = [0];
        this.byteOffsets = new Array(s.length + 1);
        this.byteOffsets[0] = 0;

        let i = 0;
        let bytes = 0;
        while (i < s.length) {
            const cp = s.codePointAt(i);
            const chLen = cp > 0xffff ? 2 : 1;
            if (cp <= 0x7f) bytes += 1;
            else if (cp <= 0x7ff) bytes += 2;
            else if (cp <= 0xffff) bytes += 3;
            else bytes += 4;

            const next = i + chLen;
            for (let j = i + 1; j <= next && j <= s.length; j++) {
                this.byteOffsets[j] = bytes;
            }
            if (cp === 10) {
                this.lineStarts.push(next);
            }
            i = next;
        }
        for (let j = 0; j <= s.length; j++) {
            if (!Number.isFinite(this.byteOffsets[j])) this.byteOffsets[j] = bytes;
        }
    }

    _lineColToIndex(line, col) {
        const s = this.text || '';
        const li = Number(line);
        const ci = Number(col);
        if (!Number.isFinite(li) || !Number.isFinite(ci) || li < 0 || ci < 0) return null;
        if (!Array.isArray(this.lineStarts) || li >= this.lineStarts.length) return null;

        const start = this.lineStarts[li];
        const lineEnd = li + 1 < this.lineStarts.length ? this.lineStarts[li + 1] - 1 : s.length;
        let idx = start;
        let remain = ci;
        while (idx < lineEnd && remain > 0) {
            const cp = s.codePointAt(idx);
            idx += cp > 0xffff ? 2 : 1;
            remain -= 1;
        }
        return Math.max(0, Math.min(s.length, idx));
    }

    _byteOffsetToIndex(byteOffset) {
        const b = Number(byteOffset);
        if (!Number.isFinite(b) || b <= 0) return 0;
        const arr = this.byteOffsets || [0];
        let lo = 0;
        let hi = arr.length - 1;
        while (lo < hi) {
            const mid = Math.floor((lo + hi) / 2);
            if (arr[mid] < b) lo = mid + 1;
            else hi = mid;
        }
        if (arr[lo] === b) return lo;
        return Math.max(0, lo - 1);
    }

    _analyzeAndPublish(version) {
        const wasm = this._wasm();
        if (!wasm || typeof wasm.analyze_lex !== 'function') {
            this.lex = { tokens: [], diagnostics: [] };
            this.parse = null;
            this.resolve = null;
            this.semantics = null;
            this.definitionById.clear();
            const payload = {
                tokens: [],
                diagnostics: [],
                foldingRanges: [],
                semanticTokens: [],
                inlayHints: [],
                config: { highlightWhitespace: false, highlightIndent: true },
            };
            this.lastUpdatePayload = payload;
            this.updateCallback(payload);
            return;
        }

        const fallbackDiagnostics = [];
        this.lex = { tokens: [], diagnostics: [] };
        this.parse = null;
        this.resolve = null;
        this.semantics = null;

        try {
            this.lex = wasm.analyze_lex(this.text);
        } catch (e) {
            console.error('[NEPLg2LanguageProvider] analyze_lex failed:', e);
            fallbackDiagnostics.push({
                startIndex: 0,
                endIndex: 0,
                message: `analyze_lex failed: ${String(e?.message || e)}`,
                severity: 'error',
            });
        }

        if (typeof wasm.analyze_parse === 'function') {
            try {
                this.parse = wasm.analyze_parse(this.text);
            } catch (e) {
                const msg = String(e?.message || e);
                console.error('[NEPLg2LanguageProvider] analyze_parse failed:', e);
                fallbackDiagnostics.push({
                    startIndex: 0,
                    endIndex: 0,
                    message: msg.includes('Maximum call stack size exceeded')
                        ? 'parser recursion overflow: parse stage skipped for this source'
                        : `analyze_parse failed: ${msg}`,
                    severity: 'warning',
                });
            }
        }

        if (this.parse?.ok && typeof wasm.analyze_name_resolution === 'function') {
            try {
                this.resolve = wasm.analyze_name_resolution(this.text);
            } catch (e) {
                console.error('[NEPLg2LanguageProvider] analyze_name_resolution failed:', e);
                fallbackDiagnostics.push({
                    startIndex: 0,
                    endIndex: 0,
                    message: `analyze_name_resolution failed: ${String(e?.message || e)}`,
                    severity: 'warning',
                });
            }
        }

        if (this.parse?.ok && typeof wasm.analyze_semantics === 'function') {
            try {
                this.semantics = wasm.analyze_semantics(this.text);
            } catch (e) {
                console.error('[NEPLg2LanguageProvider] analyze_semantics failed:', e);
                fallbackDiagnostics.push({
                    startIndex: 0,
                    endIndex: 0,
                    message: `analyze_semantics failed: ${String(e?.message || e)}`,
                    severity: 'warning',
                });
            }
        }

        if (version !== this.analysisVersion) {
            return;
        }

        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        this.definitionById = new Map(defs.map((d) => [d.id, d]));

        const tokens = this._buildEditorTokens();
        const diagnostics = this._collectDiagnostics();
        diagnostics.push(...fallbackDiagnostics);
        const foldingRanges = this._buildFoldingRanges();
        const semanticTokens = this._buildSemanticTokens();
        const inlayHints = this._buildInlayHints();

        const payload = {
            tokens,
            diagnostics,
            foldingRanges,
            semanticTokens,
            inlayHints,
            config: { highlightWhitespace: false, highlightIndent: true },
        };
        this.lastUpdatePayload = payload;
        this.updateCallback(payload);
    }

    _spanFrom(obj) {
        const s = obj && obj.span;
        if (!s) return null;
        const lcStart = this._lineColToIndex(s.start_line, s.start_col);
        const lcEnd = this._lineColToIndex(s.end_line, s.end_col);
        const start = Number.isFinite(lcStart) ? lcStart : this._byteOffsetToIndex(s.start ?? 0);
        const end = Number.isFinite(lcEnd) ? lcEnd : this._byteOffsetToIndex(s.end ?? 0);
        return {
            startIndex: start,
            endIndex: end,
            startLine: Number(s.start_line ?? 0),
            startCol: Number(s.start_col ?? 0),
            endLine: Number(s.end_line ?? 0),
            endCol: Number(s.end_col ?? 0),
        };
    }

    _severity(diag) {
        const sv = String(diag?.severity || 'error').toLowerCase();
        return sv.includes('warn') ? 'warning' : 'error';
    }

    _collectDiagnostics() {
        const all = [];
        const pushFrom = (arr) => {
            if (!Array.isArray(arr)) return;
            for (const d of arr) {
                const sp = this._spanFrom(d);
                all.push({
                    startIndex: sp ? sp.startIndex : 0,
                    endIndex: sp ? sp.endIndex : 0,
                    message: String(d?.message || 'diagnostic'),
                    severity: this._severity(d),
                });
            }
        };

        pushFrom(this.lex?.diagnostics);
        pushFrom(this.parse?.diagnostics);
        pushFrom(this.parse?.lex_diagnostics);
        pushFrom(this.resolve?.diagnostics);
        pushFrom(this.semantics?.diagnostics);

        all.sort((a, b) => a.startIndex - b.startIndex || a.endIndex - b.endIndex);
        return all;
    }

    _tokenType(kind, debug) {
        if (!kind) return 'default';
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

    _buildEditorTokens() {
        const lexTokens = Array.isArray(this.lex?.tokens) ? this.lex.tokens : [];
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        const skipKinds = new Set(['Indent', 'Dedent', 'Eof', 'Newline']);
        const normalized = [];

        for (let idx = 0; idx < lexTokens.length; idx++) {
            const tok = lexTokens[idx];
            const kind = String(tok?.kind || '');
            if (skipKinds.has(kind)) continue;
            const span = this._spanFrom(tok) || { startIndex: 0, endIndex: 0 };
            if (!Number.isFinite(span.startIndex) || !Number.isFinite(span.endIndex)) continue;
            if (span.endIndex <= span.startIndex) continue;
            let t = this._tokenType(String(tok.kind || ''), tok.debug);

            const tr = tokenRes[idx];
            if (tr && tr.resolved_def_id != null) {
                const def = this.definitionById.get(tr.resolved_def_id);
                if (def && (def.kind === 'fn' || def.kind === 'fn_alias')) {
                    t = 'function';
                }
            }
            normalized.push({
                startIndex: span.startIndex,
                endIndex: span.endIndex,
                type: t,
            });
        }

        normalized.sort((a, b) => a.startIndex - b.startIndex || a.endIndex - b.endIndex);
        return normalized;
    }

    _buildSemanticTokens() {
        const tokenSem = Array.isArray(this.semantics?.token_semantics) ? this.semantics.token_semantics : [];
        const out = [];
        for (const ts of tokenSem) {
            const sp = ts?.expr_span;
            if (!sp) continue;
            out.push({
                tokenIndex: Number(ts.token_index ?? -1),
                inferredType: ts.inferred_type || null,
                exprSpan: {
                    start: Number(sp.start ?? 0),
                    end: Number(sp.end ?? 0),
                },
                argIndex: Number.isInteger(ts?.arg_index) ? Number(ts.arg_index) : null,
                argSpan: ts?.arg_span
                    ? { start: Number(ts.arg_span.start ?? 0), end: Number(ts.arg_span.end ?? 0) }
                    : null,
            });
        }
        return out;
    }

    _buildInlayHints() {
        const tokenSem = Array.isArray(this.semantics?.token_semantics) ? this.semantics.token_semantics : [];
        const out = [];
        for (const ts of tokenSem) {
            if (!ts || !ts.inferred_type || !ts.expr_span) continue;
            const start = Number(ts.expr_span.start ?? -1);
            if (start < 0) continue;
            out.push({
                kind: 'type',
                position: start,
                label: `<${ts.inferred_type}>`,
                exprSpan: {
                    start: Number(ts.expr_span.start ?? 0),
                    end: Number(ts.expr_span.end ?? 0),
                },
            });
        }
        return out;
    }

    _walkAst(node, out) {
        if (!node || typeof node !== 'object') return;
        if (node.kind === 'Block' && node.span && Number(node.span.end_line) > Number(node.span.start_line)) {
            out.push({
                startLine: Number(node.span.start_line),
                endLine: Number(node.span.end_line),
                placeholder: '...',
            });
        }
        for (const v of Object.values(node)) {
            if (Array.isArray(v)) {
                for (const it of v) this._walkAst(it, out);
            } else if (v && typeof v === 'object') {
                this._walkAst(v, out);
            }
        }
    }

    _buildFoldingRanges() {
        const root = this.parse?.module?.root;
        if (!root) return [];
        const ranges = [];
        this._walkAst(root, ranges);
        ranges.sort((a, b) => a.startLine - b.startLine || a.endLine - b.endLine);
        return ranges;
    }

    _tokenAt(index) {
        const tokens = Array.isArray(this.lex?.tokens) ? this.lex.tokens : [];
        for (let i = 0; i < tokens.length; i++) {
            const sp = this._spanFrom(tokens[i]);
            if (sp && index >= sp.startIndex && index < sp.endIndex) {
                return { token: tokens[i], tokenIndex: i, span: sp };
            }
        }
        return null;
    }

    _tokenSemanticByIndex(tokenIndex) {
        const tokenSem = Array.isArray(this.semantics?.token_semantics) ? this.semantics.token_semantics : [];
        return tokenSem.find((x) => Number(x?.token_index) === tokenIndex) || null;
    }

    _tokenResolutionByIndex(tokenIndex) {
        const tokenRes = Array.isArray(this.semantics?.token_resolution) ? this.semantics.token_resolution : [];
        return tokenRes.find((x) => Number(x?.token_index) === tokenIndex) || null;
    }

    _formatSpan(sp) {
        if (!sp) return null;
        return `[${Number(sp.start ?? 0)}, ${Number(sp.end ?? 0)})`;
    }

    _definitionCandidates(tr) {
        if (!tr || !Array.isArray(tr.candidate_def_ids)) return [];
        return tr.candidate_def_ids
            .map((id) => this.definitionById.get(id))
            .filter(Boolean)
            .map((d) => ({
                id: d.id,
                name: d.name,
                kind: d.kind,
                span: d.span || null,
            }));
    }

    getTokenInsight(index) {
        const hit = this._tokenAt(index);
        if (!hit) return null;

        const ts = this._tokenSemanticByIndex(hit.tokenIndex);
        const tr = this._tokenResolutionByIndex(hit.tokenIndex);
        const def = tr && tr.resolved_def_id != null ? this.definitionById.get(tr.resolved_def_id) : null;
        const candidates = this._definitionCandidates(tr);

        return {
            tokenIndex: hit.tokenIndex,
            tokenKind: String(hit.token?.kind || ''),
            tokenSpan: hit.span,
            inferredType: ts?.inferred_type || null,
            exprSpan: ts?.expr_span || null,
            argIndex: Number.isInteger(ts?.arg_index) ? ts.arg_index : null,
            argSpan: ts?.arg_span || null,
            resolvedDefId: tr?.resolved_def_id ?? null,
            candidateDefIds: Array.isArray(tr?.candidate_def_ids) ? tr.candidate_def_ids : [],
            definitionCandidates: candidates,
            resolvedDefinition: def
                ? { id: def.id, name: def.name, kind: def.kind, span: def.span || null }
                : null,
        };
    }

    async getHoverInfo(index) {
        const insight = this.getTokenInsight(index);
        if (!insight) {
            const fallbackRef = this._referenceAt(index);
            if (!fallbackRef) return null;
            const resolvedDef = fallbackRef.resolved_def_id != null ? this.definitionById.get(fallbackRef.resolved_def_id) : null;
            const lines = [fallbackRef.name];
            if (resolvedDef) lines.push(`def: ${resolvedDef.kind} ${resolvedDef.name}`);
            return { content: lines.join('\n'), startIndex: Number(fallbackRef.span.start ?? index), endIndex: Number(fallbackRef.span.end ?? index + 1) };
        }

        const lines = [];
        const hit = this._tokenAt(index);
        const rawFromToken = String(hit?.token?.value || hit?.token?.debug || '').trim();
        const rawFromSource = hit?.span ? this.text.slice(hit.span.startIndex, hit.span.endIndex).trim() : '';
        const raw = rawFromToken || rawFromSource || String(insight.tokenKind || '');
        if (raw) lines.push(raw);
        if (insight.inferredType) lines.push(`type: ${insight.inferredType}`);
        if (insight.exprSpan) lines.push(`expr: ${this._formatSpan(insight.exprSpan)}`);
        if (Number.isInteger(insight.argIndex)) lines.push(`arg#${insight.argIndex}: ${this._formatSpan(insight.argSpan)}`);
        if (insight.resolvedDefinition) lines.push(`def: ${insight.resolvedDefinition.kind} ${insight.resolvedDefinition.name}`);
        if (insight.definitionCandidates.length > 1) {
            lines.push(`candidates: ${insight.definitionCandidates.map((d) => `${d.id}:${d.name}`).join(', ')}`);
        }

        if (lines.length === 0) return null;
        return { content: lines.join('\n'), startIndex: insight.tokenSpan.startIndex, endIndex: insight.tokenSpan.endIndex };
    }

    async getDefinitionLocation(index) {
        const insight = this.getTokenInsight(index);
        if (insight && insight.resolvedDefinition && insight.resolvedDefinition.span) {
            const sp = this._spanFrom({ span: insight.resolvedDefinition.span });
            return { targetIndex: sp ? sp.startIndex : 0 };
        }
        const fallbackRef = this._referenceAt(index);
        if (fallbackRef && fallbackRef.resolved_def_id != null) {
            const def = this.definitionById.get(fallbackRef.resolved_def_id);
            if (def?.span) {
                const sp = this._spanFrom({ span: def.span });
                return { targetIndex: sp ? sp.startIndex : 0 };
            }
        }
        return null;
    }

    async getDefinitionCandidates(index) {
        const insight = this.getTokenInsight(index);
        return insight ? insight.definitionCandidates : [];
    }

    async getOccurrences(index) {
        const insight = this.getTokenInsight(index);
        if (!insight) return [];
        const refs = Array.isArray(this.resolve?.references) ? this.resolve.references : [];
        const out = [];

        for (const r of refs) {
            if (!r?.span) continue;
            if (insight.resolvedDefId != null && r.resolved_def_id === insight.resolvedDefId) {
                const sp = this._spanFrom({ span: r.span });
                if (sp) out.push({ startIndex: sp.startIndex, endIndex: sp.endIndex });
            }
        }
        if (out.length > 0) return out;

        const tr = this._tokenResolutionByIndex(insight.tokenIndex);
        if (tr?.name) {
            for (const r of refs) {
                if (r?.name === tr.name && r?.span) {
                    const sp = this._spanFrom({ span: r.span });
                    if (sp) out.push({ startIndex: sp.startIndex, endIndex: sp.endIndex });
                }
            }
        }
        return out;
    }

    _referenceAt(index) {
        const refs = Array.isArray(this.resolve?.references) ? this.resolve.references : [];
        let best = null;
        let bestWidth = Number.MAX_SAFE_INTEGER;
        for (const r of refs) {
            const sp = this._spanFrom({ span: r?.span });
            if (!sp) continue;
            const s = Number(sp.startIndex ?? -1);
            const e = Number(sp.endIndex ?? -1);
            if (s < 0 || e <= s) continue;
            if (index >= s && index < e) {
                const w = e - s;
                if (w < bestWidth) {
                    best = r;
                    bestWidth = w;
                }
            }
        }
        return best;
    }

    _wordAt(index) {
        const s = this.text || '';
        let l = index;
        let r = index;
        const isWord = (c) => /[A-Za-z0-9_#]/.test(c);
        while (l > 0 && isWord(s[l - 1])) l--;
        while (r < s.length && isWord(s[r])) r++;
        return { start: l, end: r, text: s.slice(l, r) };
    }

    async getNextWordBoundary(index, direction) {
        const s = this.text || '';
        if (direction === 'left') {
            let i = Math.max(0, index - 1);
            while (i > 0 && /\s/.test(s[i])) i--;
            while (i > 0 && /[A-Za-z0-9_]/.test(s[i - 1])) i--;
            return { targetIndex: i };
        }
        let i = Math.min(s.length, index);
        while (i < s.length && /[A-Za-z0-9_]/.test(s[i])) i++;
        while (i < s.length && /\s/.test(s[i])) i++;
        return { targetIndex: i };
    }

    _collectCompletionSymbols() {
        const names = new Map();
        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        for (const d of defs) {
            if (!d?.name) continue;
            names.set(d.name, {
                label: String(d.name),
                type: d.kind === 'fn' || d.kind === 'fn_alias' ? 'function' : 'variable',
                detail: String(d.kind || ''),
                insertText: String(d.name),
            });
        }

        const byName = this.resolve?.by_name;
        if (byName && typeof byName === 'object') {
            for (const k of Object.keys(byName)) {
                if (!names.has(k)) {
                    names.set(k, {
                        label: k,
                        type: 'variable',
                        detail: 'name',
                        insertText: k,
                    });
                }
            }
        }
        return [...names.values()];
    }

    async getCompletions(index) {
        const word = this._wordAt(index);
        const prefix = (word?.text || '').toLowerCase();
        const items = [];
        for (const kw of this.keywordCompletions) {
            items.push({ label: kw, type: 'keyword', insertText: kw });
        }
        items.push(...this._collectCompletionSymbols());
        if (!prefix) return items;
        return items.filter((it) => String(it.label || '').toLowerCase().startsWith(prefix));
    }

    async getIndentation(index) {
        const lineStart = this.text.lastIndexOf('\n', index - 1) + 1;
        const line = this.text.slice(lineStart, index);
        const indent = (line.match(/^\s*/) || [''])[0];
        const trimmed = line.trim();
        if (trimmed.endsWith(':')) {
            return { textToInsert: `\n${indent}    `, cursorOffset: indent.length + 5 };
        }
        return { textToInsert: `\n${indent}`, cursorOffset: indent.length + 1 };
    }

    async toggleComment(selectionStart, selectionEnd) {
        const lineStart = this.text.lastIndexOf('\n', selectionStart - 1) + 1;
        let lineEnd = this.text.indexOf('\n', selectionEnd);
        if (lineEnd === -1) lineEnd = this.text.length;

        const selected = this.text.slice(lineStart, lineEnd);
        const lines = selected.split('\n');
        const allCommented = lines.filter((l) => l.trim() !== '').every((l) => l.trimStart().startsWith('//'));

        const next = lines.map((line) => {
            if (line.trim() === '') return line;
            if (allCommented) return line.replace(/^(\s*)\/\/\s?/, '$1');
            const lead = (line.match(/^\s*/) || [''])[0];
            return `${lead}// ${line.slice(lead.length)}`;
        });

        const newText = this.text.slice(0, lineStart) + next.join('\n') + this.text.slice(lineEnd);
        return { newText, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd };
    }

    async adjustIndentation(selectionStart, selectionEnd, isOutdent) {
        const lines = this.text.split('\n');
        const indentUnit = '    ';
        let cursor = 0;
        let startLine = 0;
        let endLine = lines.length - 1;
        for (let i = 0; i < lines.length; i++) {
            const end = cursor + lines[i].length;
            if (selectionStart >= cursor && selectionStart <= end) startLine = i;
            if (selectionEnd >= cursor && selectionEnd <= end) {
                endLine = i;
                break;
            }
            cursor = end + 1;
        }

        for (let i = startLine; i <= endLine; i++) {
            if (isOutdent) {
                if (lines[i].startsWith(indentUnit)) lines[i] = lines[i].slice(indentUnit.length);
                else lines[i] = lines[i].replace(/^\s{1,4}/, '');
            } else {
                lines[i] = indentUnit + lines[i];
            }
        }

        const newText = lines.join('\n');
        return { newText, newSelectionStart: selectionStart, newSelectionEnd: selectionEnd };
    }

    async getBracketMatch(index) {
        const text = this.text || '';
        const pairs = { '(': ')', '[': ']', '{': '}', ')': '(', ']': '[', '}': '{' };
        const c = text[index];
        if (!pairs[c]) return [];
        const isOpen = c === '(' || c === '[' || c === '{';
        const target = pairs[c];
        let depth = 1;
        for (let i = index + (isOpen ? 1 : -1); i >= 0 && i < text.length; i += isOpen ? 1 : -1) {
            if (text[i] === c) depth++;
            if (text[i] === target) depth--;
            if (depth === 0) {
                return [
                    { startIndex: index, endIndex: index + 1 },
                    { startIndex: i, endIndex: i + 1 },
                ];
            }
        }
        return [];
    }

    getAnalysisSnapshot() {
        return {
            version: this.analysisVersion,
            lex: this.lex,
            parse: this.parse,
            name_resolution: this.resolve,
            semantics: this.semantics,
            update_payload: this.lastUpdatePayload,
        };
    }

    getAst() {
        return this.parse?.module?.root || null;
    }

    getNameResolution() {
        return this.resolve || null;
    }

    getSemantics() {
        return this.semantics || null;
    }
}

window.NEPLg2LanguageProvider = NEPLg2LanguageProvider;
