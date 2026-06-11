import { buildEditorUpdatePayloadFromAnalysis } from '../../editor-core/language-analysis.js';
import type { CompilerAssetUrls } from '../../runtime/compiler-assets.js';

type AnalysisRequest = {
    type: 'analyze';
    requestId: number;
    compiler: CompilerAssetUrls;
    path: string | null;
    text: string;
    vfsSnapshot: Record<string, string> | null;
};

type StructuralRequest = {
    type: 'parse';
    requestId: number;
    compiler: CompilerAssetUrls;
    text: string;
};

type IncomingRequest = AnalysisRequest | StructuralRequest;

let compilerInitPromise: Promise<any> | null = null;

async function loadCompilerBindings(assets: CompilerAssetUrls): Promise<any> {
    if (!compilerInitPromise) {
        compilerInitPromise = (async () => {
            const compilerModule = await import(/* @vite-ignore */ assets.moduleUrl);
            if (typeof compilerModule.default === 'function') {
                await compilerModule.default({ module_or_path: assets.wasmUrl });
            }
            return compilerModule;
        })();
    }
    return compilerInitPromise;
}

function analyzeSemantics(compilerModule: any, request: AnalysisRequest): any {
    if (request.vfsSnapshot && request.path && typeof compilerModule.analyze_semantics_with_vfs === 'function') {
        return compilerModule.analyze_semantics_with_vfs(request.path, request.text, request.vfsSnapshot);
    }
    if (typeof compilerModule.analyze_semantics !== 'function') {
        return null;
    }
    return compilerModule.analyze_semantics(request.text);
}

async function handleAnalyze(request: AnalysisRequest) {
    const compilerModule = await loadCompilerBindings(request.compiler);
    const fallbackDiagnostics: any[] = [];
    let lex: any = { tokens: [], diagnostics: [] };
    let parse: any = null;
    let resolve: any = null;
    let semantics: any = null;

    try {
        semantics = analyzeSemantics(compilerModule, request);
        if (semantics) {
            lex = {
                tokens: semantics.tokens || [],
                diagnostics: (semantics.diagnostics || []).filter((d: any) => d.stage === 'lex'),
            };
            resolve = semantics.name_resolution || null;
            parse = {
                ok: semantics.ok,
                module: null,
                diagnostics: [],
            };
        }
    } catch (error: any) {
        fallbackDiagnostics.push({
            startIndex: 0,
            endIndex: 0,
            message: `analyze_semantics failed: ${String(error?.message || error)}`,
            severity: 'error',
        });
    }

    const payloadBase = buildEditorUpdatePayloadFromAnalysis(request.text, {
        lex,
        parse,
        resolve,
        semantics,
    });
    const payload = {
        ...payloadBase,
        diagnostics: [...(payloadBase.diagnostics || []), ...fallbackDiagnostics].sort((a, b) => a.startIndex - b.startIndex || a.endIndex - b.endIndex),
    };

    self.postMessage({
        type: 'analysis-result',
        requestId: request.requestId,
        lex,
        parse,
        resolve,
        semantics,
        payload,
    });
}

async function handleParse(request: StructuralRequest) {
    const compilerModule = await loadCompilerBindings(request.compiler);
    let module = null;
    try {
        if (typeof compilerModule.analyze_parse === 'function') {
            const parsePayload = compilerModule.analyze_parse(request.text);
            module = parsePayload?.module || null;
        }
    } catch (error: any) {
        self.postMessage({
            type: 'analysis-error',
            requestId: request.requestId,
            message: `analyze_parse failed: ${String(error?.message || error)}`,
        });
        return;
    }

    self.postMessage({
        type: 'structural-result',
        requestId: request.requestId,
        module,
    });
}

self.onmessage = (event: MessageEvent<IncomingRequest>) => {
    const request = event.data;
    const run = request.type === 'analyze'
        ? handleAnalyze(request)
        : handleParse(request);
    run.catch((error: any) => {
        self.postMessage({
            type: 'analysis-error',
            requestId: request.requestId,
            message: String(error?.message || error),
        });
    });
};
