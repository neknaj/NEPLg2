class JavaScriptLanguageProvider extends BaseLanguageProvider {
    constructor() {
        super('src/language/javascript/js-worker.js'); // 対応するWorkerのパスを指定
    }
}