declare const CanvasEditor: any;

type PlaygroundEditorOptions = {
    canvas: HTMLCanvasElement;
    textarea: HTMLTextAreaElement;
    popup: HTMLElement;
    problemsPanel?: HTMLElement | null;
    completionList: HTMLElement;
    languageProviders?: Record<string, any>;
    initialLanguage?: string;
    initialText?: string;
    editorOptions?: Record<string, unknown>;
    onCursorChange?: (index: number) => void;
};

type PlaygroundDocumentReplacement = {
    path: string | null;
    text: string;
    editable: boolean;
};

export class PlaygroundEditor {
    private readonly inner: any;
    private readonly providers: Record<string, any>;
    private currentLanguage: string | null;
    path: string | null;

    constructor(options: PlaygroundEditorOptions) {
        const {
            canvas,
            textarea,
            popup,
            problemsPanel,
            completionList,
            languageProviders = {},
            initialLanguage,
            initialText = '',
            editorOptions = {},
            onCursorChange,
        } = options;

        this.inner = new CanvasEditor(
            canvas,
            textarea,
            { popup, problemsPanel, completionList },
            { ...editorOptions, onCursorChange },
        );
        this.providers = { ...languageProviders };
        this.currentLanguage = null;
        this.path = null;

        if (initialLanguage) {
            this.setLanguage(initialLanguage);
        }
        if (initialText) {
            this.setText(initialText);
        }
    }

    setLanguage(languageId: string): void {
        const provider = this.providers[languageId];
        if (!provider) {
            return;
        }
        this.currentLanguage = languageId;
        this.inner.registerLanguageProvider(languageId, provider);
        if (typeof provider.setPath === 'function') {
            provider.setPath(this.path);
        }
    }

    registerLanguage(languageId: string, provider: any): void {
        this.providers[languageId] = provider;
    }

    getLanguageProvider(languageId?: string): any {
        const key = languageId || this.currentLanguage || '';
        return this.providers[key];
    }

    setText(text: string): void {
        this.inner.setText(text);
    }

    replaceDocument(document: PlaygroundDocumentReplacement): void {
        const nextPath = typeof document.path === 'string' && document.path.length > 0 ? document.path : null;
        this.path = nextPath;
        if (typeof this.inner.setEditable === 'function') {
            this.inner.setEditable(document.editable);
        }
        if (typeof this.inner.replaceDocument === 'function') {
            this.inner.replaceDocument({
                path: nextPath,
                text: document.text,
                editable: document.editable,
            });
            return;
        }
        const provider = this.getLanguageProvider();
        if (provider && typeof provider.setPath === 'function') {
            provider.setPath(nextPath);
        }
        this.inner.setText(document.text);
    }

    getText(): string {
        return this.inner.text;
    }

    setPath(path: string | null): void {
        this.path = path;
        const provider = this.getLanguageProvider();
        if (provider && typeof provider.setPath === 'function') {
            provider.setPath(path);
        }
    }

    setEditable(editable: boolean): void {
        if (typeof this.inner.setEditable === 'function') {
            this.inner.setEditable(editable);
        }
    }

    getEditable(): boolean {
        if (typeof this.inner.getEditable === 'function') {
            return this.inner.getEditable();
        }
        return true;
    }

    getPath(): string | null {
        return this.path;
    }

    resizeEditor(): void {
        this.inner.resizeEditor();
    }

    setFontSize(size: number): void {
        this.inner.setFontSize(size);
    }

    focus(): void {
        this.inner.focus();
    }

    blur(): void {
        this.inner.blur();
    }

    showPopup(content: string, x: number, y: number): void {
        this.inner.domUI.showPopup(content, x, y);
    }

    getCursorPosition(index: number): { row: number; col: number } {
        return this.inner.utils.getPosFromIndex(index, this.inner.lines);
    }

    getTokenInsight(index: number): any {
        const provider = this.getLanguageProvider();
        if (!provider || typeof provider.getTokenInsight !== 'function') {
            return null;
        }
        return provider.getTokenInsight(index);
    }

    async getHoverInfo(index: number): Promise<any> {
        const provider = this.getLanguageProvider();
        if (!provider || typeof provider.getHoverInfo !== 'function') {
            return null;
        }
        return provider.getHoverInfo(index);
    }

    async getDefinitionLocation(index: number): Promise<any> {
        const provider = this.getLanguageProvider();
        if (!provider || typeof provider.getDefinitionLocation !== 'function') {
            return null;
        }
        return provider.getDefinitionLocation(index);
    }

    async getOccurrences(index: number): Promise<any[]> {
        const provider = this.getLanguageProvider();
        if (!provider || typeof provider.getOccurrences !== 'function') {
            return [];
        }
        return provider.getOccurrences(index);
    }

    getProblems(): any[] {
        const provider = this.getLanguageProvider();
        if (!provider || typeof provider.getAnalysisSnapshot !== 'function') {
            return [];
        }
        const snapshot = provider.getAnalysisSnapshot();
        const payload = snapshot?.update_payload;
        return Array.isArray(payload?.diagnostics) ? payload.diagnostics : [];
    }

    getHighlightSnapshot(): any {
        const provider = this.getLanguageProvider();
        if (!provider || typeof provider.getAnalysisSnapshot !== 'function') {
            return null;
        }
        const snapshot = provider.getAnalysisSnapshot();
        return snapshot?.update_payload ?? null;
    }

    getRawEditor(): any {
        return this.inner;
    }
}

export function createPlaygroundEditor(options: PlaygroundEditorOptions): PlaygroundEditor {
    return new PlaygroundEditor(options);
}

if (typeof window !== 'undefined') {
    window.PlaygroundEditorFactory = {
        PlaygroundEditor,
        createPlaygroundEditor,
    };
}
