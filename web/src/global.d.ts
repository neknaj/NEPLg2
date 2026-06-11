declare global {
    interface Window {
        wasmBindings: any;
        editor: any;
        terminal: any;
        executeCommand: (cmd: string) => void;
        PlaygroundEditorFactory?: any;
        NEPLPlaygroundLanguageAnalysis?: any;
        NEPLg2CompilerAssets?: { moduleUrl: string; wasmUrl: string } | null;
    }
    const NEPLg2LanguageProvider: any;
    const CanvasEditorLibrary: any;
}

export { };
