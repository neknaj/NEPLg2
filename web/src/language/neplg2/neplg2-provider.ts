// @ts-nocheck

class NEPLg2LanguageProvider {
    constructor(options = {}) {
        this.options = options || {};
        this.vfs = this.options.vfs || null;
        this.path = null;
        this.updateCallback = () => {};
        this.text = '';
        this.lex = null;
        this.parse = null;
        this.resolve = null;
        this.semantics = null;
        this.documentVersion = 0;
        this.analysisVersion = 0;
        this.analysisWorker = null;
        this.analysisWorkerRequests = new Map();
        this.nextAnalysisWorkerRequestId = 1;
        this.currentSemanticWorkerRequestId = null;
        this.currentStructuralWorkerRequestId = null;
        this.pendingTimer = null;
        this.pendingIdleCallback = null;
        this.pendingStructuralTimer = null;
        this.pendingStructuralIdleCallback = null;
        this.analyzeDelayMs = 80;
        this.structuralAnalyzeDelayMs = 220;
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
    }

    onUpdate(callback) {
        this.updateCallback = callback || (() => {});
    }

    setPath(path) {
        const nextPath = typeof path === 'string' && path.length > 0 ? path : null;
        if (nextPath === this.path) {
            return;
        }
        this.documentVersion += 1;
        this.path = nextPath;
        this._cancelPendingAnalysis();
        this._clearAnalysisState();
        this._publishEmptyPayload();
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
        this._cancelPendingStructuralAnalysis();
        this._cancelActiveAnalysisWorkerRequests('analysis input changed');
    }

    _cancelPendingStructuralAnalysis() {
        if (this.pendingStructuralTimer != null) {
            clearTimeout(this.pendingStructuralTimer);
            this.pendingStructuralTimer = null;
        }
        if (this.pendingStructuralIdleCallback != null && typeof window !== 'undefined' && typeof window.cancelIdleCallback === 'function') {
            window.cancelIdleCallback(this.pendingStructuralIdleCallback);
            this.pendingStructuralIdleCallback = null;
        }
        const hadStructuralWorkerRequest = this.currentStructuralWorkerRequestId != null;
        this.currentStructuralWorkerRequestId = null;
        if (hadStructuralWorkerRequest) {
            this._cancelActiveAnalysisWorkerRequests('structural analysis input changed');
            return;
        }
    }

    _cancelActiveAnalysisWorkerRequests(reason) {
        if (!this.analysisWorker || this.analysisWorkerRequests.size === 0) {
            return;
        }
        const error = new Error(reason || 'analysis cancelled');
        for (const request of this.analysisWorkerRequests.values()) {
            request.reject(error);
        }
        this.analysisWorkerRequests.clear();
        this.currentSemanticWorkerRequestId = null;
        this.currentStructuralWorkerRequestId = null;
        this.analysisWorker.terminate();
        this.analysisWorker = null;
    }

    _scheduleAnalysis(immediate = false) {
        this.analysisVersion += 1;
        this._cancelPendingAnalysis();
        const version = this.analysisVersion;
        if (immediate) {
            this._analyzeAndPublish(version);
            return;
        }
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

    updateText(text) {
        const nextText = text || '';
        const previousText = this.text;
        if (nextText === previousText) {
            return;
        }
        const previousAnalysis = this.lastUpdatePayload?.analysis || null;
        const sourceDocumentVersion = Number.isFinite(previousAnalysis?.sourceDocumentVersion)
            ? Number(previousAnalysis.sourceDocumentVersion)
            : this.documentVersion;
        const sourcePath = Object.prototype.hasOwnProperty.call(previousAnalysis || {}, 'sourcePath')
            ? previousAnalysis.sourcePath
            : this.path;
        this.documentVersion += 1;
        this.text = nextText;
        if (this.lastUpdatePayload) {
            const provisionalPayload = this._buildIncrementalPayload(previousText, this.text, this.lastUpdatePayload);
            if (provisionalPayload) {
                const payload = this._decoratePayload(this._stripSemanticDerivedPayload(provisionalPayload), 'provisional', {
                    sourceDocumentVersion,
                    sourcePath,
                });
                this.lastUpdatePayload = payload;
                this.updateCallback(payload);
            }
        }
        this._scheduleAnalysis();
    }

    replaceDocument(document) {
        const nextPath = typeof document?.path === 'string' && document.path.length > 0 ? document.path : null;
        const nextText = document?.text || '';
        this._replaceDocument(nextPath, nextText);
    }

    replaceDocumentText(text) {
        this._replaceDocument(this.path, text || '');
    }

    _replaceDocument(path, text) {
        const nextPath = typeof path === 'string' && path.length > 0 ? path : null;
        const nextText = text || '';
        if (nextPath === this.path && nextText === this.text && this.lastAnalyzedText === nextText && this.lastUpdatePayload) {
            return;
        }
        this._cancelPendingAnalysis();
        this.documentVersion += 1;
        this.path = nextPath;
        this.text = nextText;
        this._clearAnalysisState();
        this._publishEmptyPayload();
        this._scheduleAnalysis(false);
    }

    _clearAnalysisState() {
        this.lex = { tokens: [], diagnostics: [] };
        this.parse = null;
        this.resolve = null;
        this.semantics = null;
        this.definitionById.clear();
        this.lastUpdatePayload = null;
        this.lastAnalyzedText = '';
    }

    _analysisMetadata(freshness, options = {}) {
        const sourceDocumentVersion = Number.isFinite(options.sourceDocumentVersion)
            ? Number(options.sourceDocumentVersion)
            : this.documentVersion;
        const sourcePath = Object.prototype.hasOwnProperty.call(options, 'sourcePath')
            ? options.sourcePath
            : this.path;
        return {
            path: this.path,
            documentVersion: this.documentVersion,
            sourcePath,
            sourceDocumentVersion,
            analysisVersion: this.analysisVersion,
            freshness,
            isFresh: freshness === 'fresh',
        };
    }

    _decoratePayload(payload, freshness, options = {}) {
        if (!payload) {
            return null;
        }
        return {
            ...payload,
            analysis: this._analysisMetadata(freshness, options),
        };
    }

    _stripSemanticDerivedPayload(payload) {
        return {
            ...payload,
            semanticHighlightTokens: [],
            diagnostics: [],
            foldingRanges: [],
            semanticTokens: [],
            inlayHints: [],
        };
    }

    _isCurrentAnalysisInput(version, documentVersion, path, text) {
        return version === this.analysisVersion
            && documentVersion === this.documentVersion
            && path === this.path
            && text === this.text;
    }

    _hasFreshAnalysis() {
        const metadata = this.lastUpdatePayload?.analysis;
        return metadata?.isFresh === true
            && metadata.documentVersion === this.documentVersion
            && metadata.path === this.path
            && this.lastAnalyzedText === this.text;
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

    _compilerAssets() {
        const assets = this.options.compilerAssets || window.NEPLg2CompilerAssets || null;
        if (!assets?.moduleUrl || !assets?.wasmUrl) {
            return null;
        }
        return {
            moduleUrl: String(assets.moduleUrl),
            wasmUrl: String(assets.wasmUrl),
        };
    }

    _canUseAnalysisWorker() {
        return typeof Worker !== 'undefined'
            && this._compilerAssets() !== null;
    }

    _analysisWorkerInstance() {
        if (!this._canUseAnalysisWorker()) {
            return null;
        }
        if (this.analysisWorker) {
            return this.analysisWorker;
        }
        const worker = new Worker('dist_ts/language/neplg2/neplg2-analysis-worker.js', { type: 'module' });
        worker.onmessage = (event) => this._handleAnalysisWorkerMessage(event.data || {});
        worker.onerror = (event) => {
            const message = event?.message || 'analysis worker failed';
            this._rejectAnalysisWorkerRequests(new Error(message));
            this.analysisWorker = null;
        };
        this.analysisWorker = worker;
        return this.analysisWorker;
    }

    _rejectAnalysisWorkerRequests(error) {
        for (const request of this.analysisWorkerRequests.values()) {
            request.reject(error);
        }
        this.analysisWorkerRequests.clear();
        this.currentSemanticWorkerRequestId = null;
        this.currentStructuralWorkerRequestId = null;
    }

    _handleAnalysisWorkerMessage(message) {
        const requestId = Number(message?.requestId);
        const request = this.analysisWorkerRequests.get(requestId);
        if (!request) {
            return;
        }
        this.analysisWorkerRequests.delete(requestId);
        if (message?.type === 'analysis-error') {
            request.reject(new Error(String(message.message || 'analysis worker error')));
            return;
        }
        request.resolve(message);
    }

    _postAnalysisWorkerRequest(message) {
        const worker = this._analysisWorkerInstance();
        const compiler = this._compilerAssets();
        if (!worker || !compiler) {
            return null;
        }
        const requestId = this.nextAnalysisWorkerRequestId++;
        const request = {
            ...message,
            requestId,
            compiler,
        };
        const promise = new Promise((resolve, reject) => {
            this.analysisWorkerRequests.set(requestId, { resolve, reject });
        });
        worker.postMessage(request);
        return { requestId, promise };
    }

    _analysisBridge() {
        if (typeof window === 'undefined' || !window.NEPLPlaygroundLanguageAnalysis) {
            throw new Error('NEPLPlaygroundLanguageAnalysis is required');
        }
        return window.NEPLPlaygroundLanguageAnalysis;
    }

    _analysisSnapshot(path = this.path) {
        return {
            path,
            sourcePath: path,
            activePath: path,
            lex: this.lex,
            parse: this.parse,
            resolve: this.resolve,
            semantics: this.semantics,
        };
    }

    _vfsSnapshotForAnalysis(path = this.path, text = this.text) {
        if (!path || !String(path).endsWith('.nepl')) {
            return null;
        }
        if (!this.vfs || typeof this.vfs.serializeForCompile !== 'function') {
            return null;
        }
        try {
            const snapshot = { ...this.vfs.serializeForCompile() };
            snapshot[path] = text;
            return snapshot;
        } catch (error) {
            console.warn('[NEPLg2LanguageProvider] VFS snapshot failed, falling back to inline semantics:', error);
            return null;
        }
    }

    _analyzeSemantics(wasm, path = this.path, text = this.text) {
        const vfsSnapshot = this._vfsSnapshotForAnalysis(path, text);
        if (vfsSnapshot && typeof wasm.analyze_semantics_with_vfs === 'function') {
            return wasm.analyze_semantics_with_vfs(path, text, vfsSnapshot);
        }
        return wasm.analyze_semantics(text);
    }

    _publishEmptyPayload() {
        const bridge = this._analysisBridge();
        const payload = this._decoratePayload(bridge.buildEditorUpdatePayloadFromAnalysis(this.text, this._analysisSnapshot()), 'empty');
        this.lastUpdatePayload = payload;
        this.updateCallback(payload);
    }

    _analyzeAndPublish(version) {
        const analysisDocumentVersion = this.documentVersion;
        const analysisPath = this.path;
        const analysisText = this.text;
        if (this._canUseAnalysisWorker()) {
            this._analyzeAndPublishWithWorker(version, analysisDocumentVersion, analysisPath, analysisText);
            return;
        }
        this._analyzeAndPublishSynchronously(version, analysisDocumentVersion, analysisPath, analysisText);
    }

    _analyzeAndPublishWithWorker(version, analysisDocumentVersion, analysisPath, analysisText) {
        const request = this._postAnalysisWorkerRequest({
            type: 'analyze',
            path: analysisPath,
            text: analysisText,
            vfsSnapshot: this._vfsSnapshotForAnalysis(analysisPath, analysisText),
        });
        if (!request) {
            this._analyzeAndPublishSynchronously(version, analysisDocumentVersion, analysisPath, analysisText);
            return;
        }
        this.currentSemanticWorkerRequestId = request.requestId;
        request.promise.then((message) => {
            if (this.currentSemanticWorkerRequestId !== request.requestId) {
                return;
            }
            this.currentSemanticWorkerRequestId = null;
            if (!this._isCurrentAnalysisInput(version, analysisDocumentVersion, analysisPath, analysisText)) {
                return;
            }
            this.lex = message.lex || { tokens: [], diagnostics: [] };
            this.parse = message.parse || null;
            this.resolve = message.resolve || null;
            this.semantics = message.semantics || null;
            const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
            this.definitionById = new Map(defs.map((d) => [d.id, d]));
            const payload = this._decoratePayload(message.payload || this._analysisBridge().buildEditorUpdatePayloadFromAnalysis(analysisText, this._analysisSnapshot(analysisPath)), 'fresh', {
                sourceDocumentVersion: analysisDocumentVersion,
                sourcePath: analysisPath,
            });
            this.lastUpdatePayload = payload;
            this.lastAnalyzedText = analysisText;
            this.updateCallback(payload);
            this._scheduleStructuralAnalysis(version, analysisText, analysisDocumentVersion, analysisPath);
        }).catch((error) => {
            if (!this._isCurrentAnalysisInput(version, analysisDocumentVersion, analysisPath, analysisText)) {
                return;
            }
            console.warn('[NEPLg2LanguageProvider] analysis worker failed:', error);
            this._publishAnalysisFailurePayload(analysisText, analysisDocumentVersion, analysisPath, error);
        });
    }

    _publishAnalysisFailurePayload(text, documentVersion, path, error) {
        this.lex = { tokens: [], diagnostics: [] };
        this.parse = null;
        this.resolve = null;
        this.semantics = null;
        this.definitionById.clear();
        const bridge = this._analysisBridge();
        const payloadBase = bridge.buildEditorUpdatePayloadFromAnalysis(text, this._analysisSnapshot(path));
        const payload = this._decoratePayload({
            ...payloadBase,
            diagnostics: [{
                startIndex: 0,
                endIndex: 0,
                message: `analysis worker failed: ${String(error?.message || error)}`,
                severity: 'error',
            }],
        }, 'fresh', {
            sourceDocumentVersion: documentVersion,
            sourcePath: path,
        });
        this.lastUpdatePayload = payload;
        this.lastAnalyzedText = text;
        this.updateCallback(payload);
    }

    _analyzeAndPublishSynchronously(version, analysisDocumentVersion, analysisPath, analysisText) {
        const wasm = this._wasm();
        if (!wasm || typeof wasm.analyze_lex !== 'function') {
            this.lex = { tokens: [], diagnostics: [] };
            this.parse = null;
            this.resolve = null;
            this.semantics = null;
            this.definitionById.clear();
            const bridge = this._analysisBridge();
            if (!this._isCurrentAnalysisInput(version, analysisDocumentVersion, analysisPath, analysisText)) {
                return;
            }
            const payload = this._decoratePayload(bridge.buildEditorUpdatePayloadFromAnalysis(analysisText, this._analysisSnapshot(analysisPath)), 'fresh', {
                sourceDocumentVersion: analysisDocumentVersion,
                sourcePath: analysisPath,
            });
            this.lastUpdatePayload = payload;
            this.lastAnalyzedText = analysisText;
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
                this.semantics = this._analyzeSemantics(wasm, analysisPath, analysisText);
                // analyze_semantics now includes tokens and name_resolution payloads
                this.lex = {
                    tokens: this.semantics.tokens || [],
                    diagnostics: (this.semantics.diagnostics || []).filter((d: any) => d.stage === 'lex')
                };
                this.resolve = this.semantics.name_resolution || null;
                this.parse = {
                    ok: this.semantics.ok,
                    module: null,
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

        if (!this._isCurrentAnalysisInput(version, analysisDocumentVersion, analysisPath, analysisText)) {
            return;
        }

        const defs = Array.isArray(this.resolve?.definitions) ? this.resolve.definitions : [];
        this.definitionById = new Map(defs.map((d) => [d.id, d]));
        const bridge = this._analysisBridge();
        const payloadBase = bridge.buildEditorUpdatePayloadFromAnalysis(analysisText, this._analysisSnapshot(analysisPath));
        const payload = this._decoratePayload({
            ...payloadBase,
            diagnostics: [...(payloadBase.diagnostics || []), ...fallbackDiagnostics].sort((a, b) => a.startIndex - b.startIndex || a.endIndex - b.endIndex),
        }, 'fresh', {
            sourceDocumentVersion: analysisDocumentVersion,
            sourcePath: analysisPath,
        });
        this.lastUpdatePayload = payload;
        this.lastAnalyzedText = analysisText;
        this.updateCallback(payload);
        this._scheduleStructuralAnalysis(version, analysisText, analysisDocumentVersion, analysisPath);
    }

    _scheduleStructuralAnalysis(version, text, documentVersion, path) {
        this._cancelPendingStructuralAnalysis();
        const run = () => {
            this.pendingStructuralIdleCallback = null;
            if (!this._isCurrentAnalysisInput(version, documentVersion, path, text)) {
                return;
            }
            if (this._canUseAnalysisWorker()) {
                this._requestStructuralParseWithWorker(version, text, documentVersion, path);
                return;
            }
            if (this._ensureStructuralParse()) {
                const bridge = this._analysisBridge();
                const payload = this._decoratePayload(bridge.buildEditorUpdatePayloadFromAnalysis(text, this._analysisSnapshot(path)), 'fresh', {
                    sourceDocumentVersion: documentVersion,
                    sourcePath: path,
                });
                this.lastUpdatePayload = payload;
                this.updateCallback(payload);
            }
        };
        this.pendingStructuralTimer = setTimeout(() => {
            this.pendingStructuralTimer = null;
            if (typeof window !== 'undefined' && typeof window.requestIdleCallback === 'function') {
                this.pendingStructuralIdleCallback = window.requestIdleCallback(run, { timeout: 900 });
            } else {
                run();
            }
        }, this.structuralAnalyzeDelayMs);
    }

    _requestStructuralParseWithWorker(version, text, documentVersion, path) {
        const request = this._postAnalysisWorkerRequest({
            type: 'parse',
            text,
        });
        if (!request) {
            if (this._ensureStructuralParse()) {
                this._publishStructuralPayload(text, documentVersion, path);
            }
            return;
        }
        this.currentStructuralWorkerRequestId = request.requestId;
        request.promise.then((message) => {
            if (this.currentStructuralWorkerRequestId !== request.requestId) {
                return;
            }
            this.currentStructuralWorkerRequestId = null;
            if (!this._isCurrentAnalysisInput(version, documentVersion, path, text)) {
                return;
            }
            this.parse = {
                ...(this.parse || {}),
                module: message.module || null,
                diagnostics: [],
            };
            if (this.parse?.module) {
                this._publishStructuralPayload(text, documentVersion, path);
            }
        }).catch((error) => {
            if (this._isCurrentAnalysisInput(version, documentVersion, path, text)) {
                console.warn('[NEPLg2LanguageProvider] structural analysis worker failed:', error);
            }
        });
    }

    _publishStructuralPayload(text, documentVersion, path) {
        const bridge = this._analysisBridge();
        const payload = this._decoratePayload(bridge.buildEditorUpdatePayloadFromAnalysis(text, this._analysisSnapshot(path)), 'fresh', {
            sourceDocumentVersion: documentVersion,
            sourcePath: path,
        });
        this.lastUpdatePayload = payload;
        this.updateCallback(payload);
    }

    _ensureStructuralParse() {
        if (this.parse?.module && this.lastAnalyzedText === this.text) {
            return true;
        }
        const wasm = this._wasm();
        if (!wasm || typeof wasm.analyze_parse !== 'function') {
            return false;
        }
        try {
            const parsePayload = wasm.analyze_parse(this.text);
            this.parse = {
                ...(this.parse || {}),
                module: parsePayload?.module || null,
                diagnostics: [],
            };
            return Boolean(this.parse?.module);
        } catch (error) {
            console.warn('[NEPLg2LanguageProvider] structural parse failed:', error);
            return false;
        }
    }

    getTokenInsight(index) {
        if (!this._hasFreshAnalysis()) {
            return null;
        }
        const bridge = this._analysisBridge();
        return bridge.getTokenInsightFromAnalysis(this.text, this._analysisSnapshot(), index);
    }

    async getHoverInfo(index) {
        if (!this._hasFreshAnalysis()) {
            return null;
        }
        const bridge = this._analysisBridge();
        return bridge.getHoverInfoFromAnalysis(this.text, this._analysisSnapshot(), index);
    }

    async getDefinitionLocation(index) {
        if (!this._hasFreshAnalysis()) {
            return null;
        }
        const bridge = this._analysisBridge();
        return bridge.getDefinitionLocationFromAnalysis(this.text, this._analysisSnapshot(), index);
    }

    async getDefinitionCandidates(index) {
        const insight = this.getTokenInsight(index);
        return insight ? insight.definitionCandidates : [];
    }

    async getOccurrences(index) {
        if (!this._hasFreshAnalysis()) {
            return [];
        }
        const bridge = this._analysisBridge();
        return bridge.getOccurrencesFromAnalysis(this.text, this._analysisSnapshot(), index);
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
        if (!this._hasFreshAnalysis()) {
            return [];
        }
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
        if (!this.parse?.module && this._canUseAnalysisWorker() && this._hasFreshAnalysis()) {
            this._scheduleStructuralAnalysis(this.analysisVersion, this.text, this.documentVersion, this.path);
            return null;
        }
        this._ensureStructuralParse();
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
