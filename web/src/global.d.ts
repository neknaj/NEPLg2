declare global {
    interface Window {
        wasmBindings: any;
        editor: any;
        terminal: any;
        executeCommand: (cmd: string) => void;
        PlaygroundEditorFactory?: any;
        NEPLPlaygroundLanguageAnalysis?: any;
    }
    const NEPLg2LanguageProvider: any;
    const CanvasEditorLibrary: any;
}

export { };
