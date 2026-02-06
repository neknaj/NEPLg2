(function (global) {
    'use strict';

    /**
     * Canvasエディタを外部プロジェクトから簡単に利用するためのヘルパー。
     * @param {object} options
     * @param {HTMLCanvasElement} options.canvas
     * @param {HTMLTextAreaElement} options.textarea
     * @param {HTMLElement} options.popup
     * @param {HTMLElement} options.problemsPanel
     * @param {HTMLElement} options.completionList
     * @param {Object.<string, BaseLanguageProvider>} [options.languageProviders]
     * @param {string} [options.initialLanguage]
     * @param {string} [options.initialText]
     * @param {object} [options.editorOptions]
     * @returns {{ editor: CanvasEditor, setLanguage: Function, registerLanguage: Function, getLanguageProvider: Function }}
     */
    function createCanvasEditor(options) {
        const {
            canvas,
            textarea,
            popup,
            problemsPanel,
            completionList,
            languageProviders = {},
            initialLanguage,
            initialText = '',
            editorOptions = {}
        } = options;

        const domElements = { popup, problemsPanel, completionList };
        const editor = new CanvasEditor(canvas, textarea, domElements, editorOptions);
        const providers = { ...languageProviders };

        function setLanguage(languageId) {
            const provider = providers[languageId];
            if (!provider) {
                return;
            }
            editor.registerLanguageProvider(languageId, provider);
        }

        function registerLanguage(languageId, provider) {
            providers[languageId] = provider;
        }

        function getLanguageProvider(languageId) {
            return providers[languageId];
        }

        if (initialLanguage && providers[initialLanguage]) {
            setLanguage(initialLanguage);
        }

        if (initialText) {
            editor.setText(initialText);
        }

        return { editor, setLanguage, registerLanguage, getLanguageProvider };
    }

    /**
     * Web Workerパスだけを指定して簡易的にカスタム言語プロバイダを作成するファクトリ。
     * @param {string} workerPath
     * @returns {BaseLanguageProvider}
     */
    function createWorkerLanguageProvider(workerPath) {
        return new BaseLanguageProvider(workerPath);
    }

    global.CanvasEditorLibrary = {
        CanvasEditor,
        BaseLanguageProvider,
        JavaScriptLanguageProvider,
        MarkdownLanguageProvider,
        NeplLanguageProvider,
        createCanvasEditor,
        createWorkerLanguageProvider
    };
})(typeof window !== 'undefined' ? window : this);
