class MarkdownLanguageProvider extends BaseLanguageProvider {
    constructor() {
        super('src/language/markdown/md-worker.js'); // 対応するWorkerのパスを指定
    }
}