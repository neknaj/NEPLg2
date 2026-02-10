// js-worker.js

importScripts('js-analyzer.js');

let analyzer;
self.onmessage = (event) => {
    const { type, payload, requestId } = event.data;
    switch (type) {
        case 'updateText':
            analyzer = new JavaScriptAnalyzer(payload);
            analyzer.analyze();
            self.postMessage({
                type: 'update',
                payload: {
                    tokens: analyzer.tokens,
                    diagnostics: analyzer.diagnostics,
                    foldingRanges: analyzer.foldingRanges,
                    config: { highlightWhitespace: true, highlightIndent: true }
                }
            });
            break;
        case 'getHoverInfo': if (analyzer) self.postMessage({ type, payload: analyzer.getHoverInfoAt(payload.index), requestId }); break;
        case 'getDefinitionLocation': if (analyzer) self.postMessage({ type, payload: analyzer.getDefinitionLocationAt(payload.index), requestId }); break;
        case 'getOccurrences': if (analyzer) self.postMessage({ type, payload: analyzer.getOccurrencesAt(payload.index), requestId }); break;
        case 'getNextWordBoundary': if (analyzer) self.postMessage({ type, payload: { targetIndex: analyzer.getNextWordBoundary(payload.index, payload.direction) }, requestId }); break;
        case 'getCompletions': if(analyzer) self.postMessage({ type, payload: analyzer.getCompletions(payload.index), requestId}); break;
        case 'getIndentation': if (analyzer) self.postMessage({ type, payload: analyzer.getIndentationAt(payload.index), requestId }); break;
        case 'toggleComment': if (analyzer) self.postMessage({ type, payload: analyzer.toggleCommentAt(payload.selectionStart, payload.selectionEnd), requestId }); break;
        case 'adjustIndentation': if (analyzer) self.postMessage({ type, payload: analyzer.adjustIndentationAt(payload.selectionStart, payload.selectionEnd, payload.isOutdent), requestId }); break;
        case 'getBracketMatch': if (analyzer) self.postMessage({ type, payload: analyzer.getBracketMatchAt(payload.index), requestId }); break;
    }
};