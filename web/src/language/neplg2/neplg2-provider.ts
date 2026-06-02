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
        this.pendingIdleCallback = null;
        this.analyzeDelayMs = 80;
        this.lastUpdatePayload = null;
        this.lastAnalyzedText = '';
        this.definitionById = new Map();
        this.keywordCompletions = [
            'fn', 'impure', 'pub', 'let', 'mut', 'set',
            'if', 'cond', 'then', 'else', 'while', 'do', 'match',
            'trait', 'impl', 'for', 'enum', 'struct',
            'unit', 'bool', 'char', 'str', 'i32',
            '#entry', '#target', '#indent', '#import', '#intrinsic', '@merge',
        ];
        this.lineStarts = [0];
        this.byteOffsets = [0];
    }

    onUpdate(callback) {
        this.updateCallback = callback || (() => {});
    }

    _cancelPendingAnalysis() {
        if (this.pendingTimer != null) {
            clearTimeout(this.pendingTimer);
            this.pendingTimer = null;
        }
        if (this.pendingIdleCallback != null && typeof window !== 'undefined' && typeof window.cancelIdleCallback === 'function') {
            window.cancelIdleCallback(this.pendingIdleCallback);
            this.pendingIdleCallback = null;
        }
    }

    updateText(text) {
        const nextText = text || '';
        const previousText = this.text;
        if (nextText === previousText) {
            return;
        }
        this.text = nextText;
        this._rebuildOffsetMaps();
        if (this.lastUpdatePayload) {
            const provisionalPayload = this._buildIncrementalPayload(previousText, this.text, this.lastUpdatePayload);
            if (provisionalPayload) {
                this.lastUpdatePayload = provisionalPayload;
                this.updateCallback(provisionalPayload);
            }
        }
        this.analysisVersion += 1;
        this._cancelPendingAnalysis();
        const version = this.analysisVersion;
        this.pendingTimer = setTimeout(() => {
            this.pendingTimer = null;
            if (typeof window !== 'undefined' && typeof window.requestIdleCallback === 'function') {
                this.pendingIdleCallback = window.requestIdleCallback(() => {
                    this.pendingIdleCallback = null;
                    this._analyzeAndPublish(version);
                }, { timeout: 300 });
            } else {
                this._analyzeAndPublish(version);
            }
        }, this.analyzeDelayMs);
    }

    replaceDocumentText(text) {
        const nextText = text || '';
        if (nextText === this.text && this.lastAnalyzedText === nextText && this.lastUpdatePayload) {
            return;
        }
        this._cancelPendingAnalysis();
        this.text = nextText;
        this._rebuildOffsetMaps();
        this.analysisVersion += 1;
        this._analyzeAndPublish(this.analysisVersion);
    }

    _buildIncrementalPayload(previousText, nextText, previousPayload) {
        if (!previousPayload) {
            return null;
        }
        const bridge = this._analysisBridge();
        if (typeof bridge.remapEditorUpdatePayloadForTextChange !== 'function') {
            return null;
        }
        return bridge.remapEditorUpdatePayloadForTextChange(previousText, nextText, previousPayload);
    }

    _wasm() {
        return window.wasmBindings || null;
    }

    _analysisBridge() {
        if (typeof window === 'undefined' || !window.NEPLPlaygroundLanguageAnalysis) {
            throw new Error('NEPLPlaygroundLanguageAnalysis is required');
        }
        return window.NEPLPlaygroundLanguageAnalysis;
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
            const bridge = this._analysisBridge();
            const payload = bridge.buildEditorUpdatePayloadFromAnalysis(this.text, {
                lex: this.lex,
                parse: this.parse,
                resolve: this.resolve,
                semantics: this.semantics,
            });
            this.lastUpdatePayload = payload;
            this.lastAnalyzedText = this.text;
            this.updateCallback(payload);
            return;
        }

        const fallbackDiagnostics = [];
        this.lex = { tokens: [], diagnostics: [] };
        this.parse = null;
        this.resolve = null;
        this.semantics = null;

        if (typeof wasm.analyze_semantics === 'function') {
            try {
                this.semantics = wasm.analyze_semantics(this.text);
                // analyze_semantics now includes tokens and name_resolution payloads
                this.lex = {
                    tokens: this.semantics.tokens || [],
                    diagnostics: (this.semantics.diagnostics || []).filter((d: any) => d.stage === 'lex')
                };
                this.resolve = this.semantics.name_resolution || null;
                // Currently, parse AST is not directly included, but diagnostics are there
                // We run analyze_parse strictly to get the AST for folding ranges
                let parsePayload = null;
                if (typeof wasm.analyze_parse === 'function') {
                    try { parsePayload = wasm.analyze_parse(this.text); } catch (e) {}
                }
                this.parse = {
                    ok: this.semantics.ok,
                    module: parsePayload?.module || null,
                    diagnostics: [] // We use this.semantics.diagnostics for everything
                };
            } catch (e) {
                console.error('[NEPLg2LanguageProvider] analyze_semantics failed:', e);
                fallbackDiagnostics.push({
                    startIndex: 0,
                    endIndex: 0,
                    message: `analyze_semantics failed: ${String(e?.message || e)}`,
                    severity: 'error',
                });
            }
        }

        if (version !== this.analysisVersion) {
            return;
        }

        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        this.definitionById = new Map(defs.map((d) => [d.id, d]));
        const bridge = this._analysisBridge();
        const payloadBase = bridge.buildEditorUpdatePayloadFromAnalysis(this.text, {
            lex: this.lex,
            parse: this.parse,
            resolve: this.resolve,
            semantics: this.semantics,
        });
        const payload = {
            ...payloadBase,
            diagnostics: [...(payloadBase.diagnostics || []), ...fallbackDiagnostics].sort((a, b) => a.startIndex - b.startIndex || a.endIndex - b.endIndex),
        };
        this.lastUpdatePayload = payload;
        this.lastAnalyzedText = this.text;
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

    _formatHoverExpression(sp) {
        if (!sp) return null;
        const start = Math.max(0, Math.min(this.text.length, Math.trunc(Number(sp.start ?? 0))));
        const end = Math.max(start, Math.min(this.text.length, Math.trunc(Number(sp.end ?? start))));
        const snippet = this.text.slice(start, end).replace(/\s+/g, ' ').trim();
        if (!snippet) return null;
        if (snippet.length <= 160) return snippet;
        return `${snippet.slice(0, 157)}...`;
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
        const bridge = this._analysisBridge();
        return bridge.getTokenInsightFromAnalysis(this.text, {
            lex: this.lex,
            parse: this.parse,
            resolve: this.resolve,
            semantics: this.semantics,
        }, index);
    }

    async getHoverInfo(index) {
        const bridge = this._analysisBridge();
        return bridge.getHoverInfoFromAnalysis(this.text, {
            lex: this.lex,
            parse: this.parse,
            resolve: this.resolve,
            semantics: this.semantics,
        }, index);
    }

    async getDefinitionLocation(index) {
        const bridge = this._analysisBridge();
        return bridge.getDefinitionLocationFromAnalysis(this.text, {
            lex: this.lex,
            parse: this.parse,
            resolve: this.resolve,
            semantics: this.semantics,
        }, index);
    }

    async getDefinitionCandidates(index) {
        const insight = this.getTokenInsight(index);
        return insight ? insight.definitionCandidates : [];
    }

    async getOccurrences(index) {
        const bridge = this._analysisBridge();
        return bridge.getOccurrencesFromAnalysis(this.text, {
            lex: this.lex,
            parse: this.parse,
            resolve: this.resolve,
            semantics: this.semantics,
        }, index);
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
