declare global {
    interface Window {
        wasmBindings: any;
        editor: any;
        terminal: any;
        executeCommand: (cmd: string) => void;
    }
    const NEPLg2LanguageProvider: any;
    const CanvasEditorLibrary: any;
}

export { };
