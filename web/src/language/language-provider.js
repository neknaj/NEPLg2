/**
 * 言語機能プロバイダの基底クラス。
 * Web Workerとの通信を抽象化する。
 */
class BaseLanguageProvider {
    /**
     * @param {string} workerPath - Web Workerスクリプトへのパス
     */
    constructor(workerPath) {
        this.worker = new Worker(workerPath);
        this.callbacks = new Map();
        this.updateCallback = () => { };

        this.worker.onmessage = (event) => {
            const { type, payload, requestId } = event.data;
            if (type === 'update') {
                this.updateCallback(payload);
            } else if (this.callbacks.has(requestId)) {
                const callback = this.callbacks.get(requestId);
                callback(payload);
                this.callbacks.delete(requestId);
            }
        };
    }

    _postMessageAndWaitForResult(type, payload) {
        return new Promise((resolve) => {
            const requestId = Date.now() + Math.random();
            this.callbacks.set(requestId, resolve);
            this.worker.postMessage({ type, payload, requestId });
        });
    }

    onUpdate(callback) { this.updateCallback = callback; }
    updateText(text) { this.worker.postMessage({ type: 'updateText', payload: text }); }
    getHoverInfo(index) { return this._postMessageAndWaitForResult('getHoverInfo', { index }); }
    getDefinitionLocation(index) { return this._postMessageAndWaitForResult('getDefinitionLocation', { index }); }
    getOccurrences(index) { return this._postMessageAndWaitForResult('getOccurrences', { index }); }
    getNextWordBoundary(index, direction) { return this._postMessageAndWaitForResult('getNextWordBoundary', { index, direction }); }
    getCompletions(index) { return this._postMessageAndWaitForResult('getCompletions', { index }); }
    getIndentation(index) { return this._postMessageAndWaitForResult('getIndentation', { index }); }
    toggleComment(selectionStart, selectionEnd) { return this._postMessageAndWaitForResult('toggleComment', { selectionStart, selectionEnd }); }
    adjustIndentation(selectionStart, selectionEnd, isOutdent) { return this._postMessageAndWaitForResult('adjustIndentation', { selectionStart, selectionEnd, isOutdent }); }
    getBracketMatch(index) { return this._postMessageAndWaitForResult('getBracketMatch', { index }); }
}

window.BaseLanguageProvider = BaseLanguageProvider;
