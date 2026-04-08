import { createPlaygroundEditor, PlaygroundEditor } from '../editor-core/browser-adapter.js';
import { FileExplorer } from '../library/explorer.js';
import { TabManager } from '../library/tabs.js';
import { CanvasTerminal } from '../terminal/terminal.js';
import {
    clampSplitRatio,
    closeLeaf,
    collectLeaves,
    countLeavesByKind,
    createDefaultWorkspace,
    createLeaf,
    DropZone,
    findNode,
    hydratePanelCounter,
    moveLeaf,
    normalizeTree,
    PanelKind,
    splitLeaf,
    WorkspaceNode,
    WorkspaceSnapshot,
} from './panel-layout.js';

type EditorRuntime = {
    leafId: string;
    panelKind: 'editor';
    rootEl: HTMLElement;
    headerTitleEl: HTMLElement;
    tabbarEl: HTMLElement;
    contentEl: HTMLElement;
    canvas: HTMLCanvasElement;
    textarea: HTMLTextAreaElement;
    completionList: HTMLElement;
    editor: PlaygroundEditor;
    tabManager: TabManager;
};

type TerminalRuntime = {
    leafId: string;
    panelKind: 'terminal';
    rootEl: HTMLElement;
    headerTitleEl: HTMLElement;
    contentEl: HTMLElement;
    canvas: HTMLCanvasElement;
    textarea: HTMLTextAreaElement;
    terminal: CanvasTerminal;
};

type ExplorerRuntime = {
    leafId: string;
    panelKind: 'explorer';
    rootEl: HTMLElement;
    headerTitleEl: HTMLElement;
    contentEl: HTMLElement;
    explorer: FileExplorer;
};

type LeafRuntime = EditorRuntime | TerminalRuntime | ExplorerRuntime;

type PanelManagerOptions = {
    root: HTMLElement;
    popup: HTMLElement;
    vfs: any;
    neplProvider: any;
    cursorSpan: HTMLElement;
    analysisSpan: HTMLElement;
    terminalStatusSpan: HTMLElement;
};

const WORKSPACE_STORAGE_KEY = 'neplg2-playground-workspace-v1';

export class PlaygroundPanelManager {
    root: HTMLElement;
    popup: HTMLElement;
    vfs: any;
    neplProvider: any;
    cursorSpan: HTMLElement;
    analysisSpan: HTMLElement;
    terminalStatusSpan: HTMLElement;
    snapshot: WorkspaceSnapshot;
    leafRuntimeMap: Map<string, LeafRuntime>;
    splitDomMap: Map<string, { first: HTMLElement; second: HTMLElement }>;
    dragSourceLeafId: string | null;
    resizeState: { splitId: string; dir: 'h' | 'v'; rect: DOMRect } | null;
    currentFontSize: number;

    constructor(options: PanelManagerOptions) {
        this.root = options.root;
        this.popup = options.popup;
        this.vfs = options.vfs;
        this.neplProvider = options.neplProvider;
        this.cursorSpan = options.cursorSpan;
        this.analysisSpan = options.analysisSpan;
        this.terminalStatusSpan = options.terminalStatusSpan;
        this.snapshot = this.loadWorkspaceSnapshot();
        this.leafRuntimeMap = new Map();
        this.splitDomMap = new Map();
        this.dragSourceLeafId = null;
        this.resizeState = null;
        this.currentFontSize = 14;
        this.bindWindowEvents();
    }

    bindWindowEvents() {
        window.addEventListener('mousemove', (event) => this.handleResizeMove(event));
        window.addEventListener('mouseup', () => this.stopResize());
    }

    loadWorkspaceSnapshot(): WorkspaceSnapshot {
        try {
            const raw = localStorage.getItem(WORKSPACE_STORAGE_KEY);
            if (raw) {
                const parsed = JSON.parse(raw) as WorkspaceSnapshot;
                parsed.root = normalizeTree(parsed.root)!;
                hydratePanelCounter(parsed.root);
                return parsed;
            }
        } catch (error) {
            console.warn('[Playground] Failed to restore workspace snapshot', error);
        }
        const snapshot = createDefaultWorkspace();
        hydratePanelCounter(snapshot.root);
        return snapshot;
    }

    saveWorkspaceSnapshot() {
        this.syncSnapshotFromRuntimes();
        localStorage.setItem(WORKSPACE_STORAGE_KEY, JSON.stringify(this.snapshot));
    }

    syncSnapshotFromRuntimes() {
        for (const leaf of collectLeaves(this.snapshot.root)) {
            const runtime = this.leafRuntimeMap.get(leaf.id);
            if (!runtime) {
                continue;
            }
            if (runtime.panelKind === 'editor') {
                const tabState = runtime.tabManager.getTabSnapshot();
                leaf.paths = tabState.paths;
                leaf.activePath = tabState.activePath;
            } else {
                leaf.paths = [];
                leaf.activePath = null;
            }
        }
    }

    redraw() {
        this.root.innerHTML = '';
        this.splitDomMap.clear();
        this.disposeRemovedRuntimes();
        this.root.appendChild(this.renderNode(this.snapshot.root));
        this.syncFocusedClasses();
        this.syncStatusBar();
        this.resizeAll();
        this.saveWorkspaceSnapshot();
    }

    disposeRemovedRuntimes() {
        const activeLeafIds = new Set(collectLeaves(this.snapshot.root).map((leaf) => leaf.id));
        for (const [leafId, runtime] of this.leafRuntimeMap.entries()) {
            if (activeLeafIds.has(leafId)) {
                continue;
            }
            if (runtime.panelKind === 'terminal') {
                runtime.terminal.dispose();
            }
            this.leafRuntimeMap.delete(leafId);
        }
    }

    renderNode(node: WorkspaceNode): HTMLElement {
        if (node.kind === 'leaf') {
            return this.ensureLeafRuntime(node).rootEl;
        }
        const splitEl = document.createElement('div');
        splitEl.className = `split-node ${node.dir === 'v' ? 'split-v' : 'split-h'}`;

        const firstSlot = document.createElement('div');
        firstSlot.className = 'pane-slot';
        const secondSlot = document.createElement('div');
        secondSlot.className = 'pane-slot';
        this.applySplitRatio(node.id, node.ratio, firstSlot, secondSlot);

        const handle = document.createElement('div');
        handle.className = `pane-split ${node.dir === 'v' ? 'split-v' : 'split-h'}`;
        handle.addEventListener('mousedown', (event) => this.startResize(event, node.id, node.dir));

        firstSlot.appendChild(this.renderNode(node.first));
        secondSlot.appendChild(this.renderNode(node.second));
        splitEl.appendChild(firstSlot);
        splitEl.appendChild(handle);
        splitEl.appendChild(secondSlot);
        this.splitDomMap.set(node.id, { first: firstSlot, second: secondSlot });
        return splitEl;
    }

    applySplitRatio(splitId: string, ratio: number, firstEl?: HTMLElement, secondEl?: HTMLElement) {
        const dom = firstEl && secondEl ? { first: firstEl, second: secondEl } : this.splitDomMap.get(splitId);
        if (!dom) {
            return;
        }
        const clamped = clampSplitRatio(ratio);
        dom.first.style.flex = `0 0 ${clamped * 100}%`;
        dom.second.style.flex = '1 1 0';
    }

    ensureLeafRuntime(leaf: Extract<WorkspaceNode, { kind: 'leaf' }>): LeafRuntime {
        const existing = this.leafRuntimeMap.get(leaf.id);
        if (existing) {
            return existing;
        }
        let runtime: LeafRuntime;
        if (leaf.panelKind === 'editor') {
            runtime = this.createEditorRuntime(leaf);
        } else if (leaf.panelKind === 'terminal') {
            runtime = this.createTerminalRuntime(leaf);
        } else {
            runtime = this.createExplorerRuntime(leaf);
        }
        this.leafRuntimeMap.set(leaf.id, runtime);
        return runtime;
    }

    createLeafRoot(leafId: string, panelKind: PanelKind, title: string) {
        const rootEl = document.createElement('section');
        rootEl.className = 'panel';
        rootEl.dataset.panelId = leafId;
        rootEl.dataset.panelKind = panelKind;
        rootEl.addEventListener('mousedown', () => this.setFocusedLeaf(leafId));
        rootEl.addEventListener('dragover', (event) => this.handlePanelDragOver(event, leafId));
        rootEl.addEventListener('dragleave', () => this.clearDropHighlight(leafId));
        rootEl.addEventListener('drop', (event) => this.handlePanelDrop(event, leafId));

        const header = document.createElement('div');
        header.className = 'panel-header';
        header.draggable = true;
        header.addEventListener('dragstart', (event) => this.handleDragStart(event, leafId));
        header.addEventListener('dragend', () => this.clearAllDropHighlights());

        const titleEl = document.createElement('span');
        titleEl.className = 'panel-title';
        titleEl.textContent = title;

        const actions = document.createElement('div');
        actions.className = 'panel-actions';

        header.appendChild(titleEl);
        header.appendChild(actions);
        rootEl.appendChild(header);
        return { rootEl, header, titleEl, actions };
    }

    createPanelButton(label: string, title: string, onClick: () => void): HTMLButtonElement {
        const button = document.createElement('button');
        button.className = 'panel-btn';
        button.textContent = label;
        button.title = title;
        button.addEventListener('click', (event) => {
            event.stopPropagation();
            onClick();
        });
        return button;
    }

    createEditorRuntime(leaf: Extract<WorkspaceNode, { kind: 'leaf' }>): EditorRuntime {
        const shell = this.createLeafRoot(leaf.id, 'editor', 'Editor');
        const tabbarEl = document.createElement('div');
        tabbarEl.className = 'tabbar';
        shell.rootEl.appendChild(tabbarEl);

        const contentEl = document.createElement('div');
        contentEl.className = 'panel-content';
        const canvasContainer = document.createElement('div');
        canvasContainer.className = 'canvas-container';
        const canvas = document.createElement('canvas');
        const textarea = document.createElement('textarea');
        textarea.className = 'hidden-input';
        textarea.autocomplete = 'off';
        textarea.setAttribute('autocorrect', 'off');
        textarea.autocapitalize = 'off';
        textarea.spellcheck = false;
        const completionList = document.createElement('ul');
        completionList.className = 'popup-menu hidden';
        canvasContainer.appendChild(canvas);
        canvasContainer.appendChild(textarea);
        canvasContainer.appendChild(completionList);
        contentEl.appendChild(canvasContainer);
        shell.rootEl.appendChild(contentEl);

        const runtime: EditorRuntime = {
            leafId: leaf.id,
            panelKind: 'editor',
            rootEl: shell.rootEl,
            headerTitleEl: shell.titleEl,
            tabbarEl,
            contentEl,
            canvas,
            textarea,
            completionList,
            editor: null as unknown as PlaygroundEditor,
            tabManager: null as unknown as TabManager,
        };

        runtime.editor = createPlaygroundEditor({
            canvas,
            textarea,
            popup: this.popup,
            problemsPanel: null,
            completionList,
            languageProviders: { nepl: this.neplProvider },
            initialLanguage: 'nepl',
            onCursorChange: (index: number) => {
                if (this.snapshot.focusedLeafId !== leaf.id) {
                    return;
                }
                const pos = runtime.editor.getCursorPosition(index);
                this.cursorSpan.textContent = `Ln ${pos.row + 1}, Col ${pos.col + 1}`;
                const insight = runtime.editor.getTokenInsight(index);
                if (!insight) {
                    this.analysisSpan.textContent = '';
                    return;
                }
                const parts: string[] = [];
                if (insight.inferredType) parts.push(`<${insight.inferredType}>`);
                if (insight.resolvedDefinition) {
                    parts.push(`${insight.resolvedDefinition.kind}:${insight.resolvedDefinition.name}`);
                }
                this.analysisSpan.textContent = parts.join(' | ');
            },
        });
        runtime.editor.setFontSize(this.currentFontSize);
        runtime.tabManager = new TabManager(tabbarEl, runtime.editor, this.vfs, {
            onStateChange: () => {
                const targetLeaf = findNode(this.snapshot.root, leaf.id)?.node;
                if (targetLeaf && targetLeaf.kind === 'leaf') {
                    const state = runtime.tabManager.getTabSnapshot();
                    targetLeaf.paths = state.paths;
                    targetLeaf.activePath = state.activePath;
                }
                this.saveWorkspaceSnapshot();
                if (this.snapshot.focusedLeafId === leaf.id) {
                    this.syncStatusBar();
                }
            },
        });

        shell.actions.appendChild(this.createPanelButton('R', 'Split right', () => this.splitPanel(leaf.id, 'h')));
        shell.actions.appendChild(this.createPanelButton('D', 'Split down', () => this.splitPanel(leaf.id, 'v')));
        shell.actions.appendChild(this.createPanelButton('x', 'Close panel', () => this.closePanel(leaf.id)));

        runtime.tabManager.restoreTabs(leaf.paths || [], leaf.activePath || null);
        return runtime;
    }

    createTerminalRuntime(leaf: Extract<WorkspaceNode, { kind: 'leaf' }>): TerminalRuntime {
        const shell = this.createLeafRoot(leaf.id, 'terminal', 'Terminal');
        const contentEl = document.createElement('div');
        contentEl.className = 'panel-content';
        const canvasContainer = document.createElement('div');
        canvasContainer.className = 'canvas-container';
        const canvas = document.createElement('canvas');
        const textarea = document.createElement('textarea');
        textarea.className = 'hidden-input';
        textarea.autocomplete = 'off';
        textarea.setAttribute('autocorrect', 'off');
        textarea.autocapitalize = 'off';
        textarea.spellcheck = false;
        canvasContainer.appendChild(canvas);
        canvasContainer.appendChild(textarea);
        contentEl.appendChild(canvasContainer);
        shell.rootEl.appendChild(contentEl);

        const terminal = new CanvasTerminal(canvas, textarea, null, { vfs: this.vfs });
        terminal.setFontSize(this.currentFontSize);

        const runtime: TerminalRuntime = {
            leafId: leaf.id,
            panelKind: 'terminal',
            rootEl: shell.rootEl,
            headerTitleEl: shell.titleEl,
            contentEl,
            canvas,
            textarea,
            terminal,
        };

        shell.actions.appendChild(this.createPanelButton('R', 'Split right', () => this.splitPanel(leaf.id, 'h')));
        shell.actions.appendChild(this.createPanelButton('D', 'Split down', () => this.splitPanel(leaf.id, 'v')));
        shell.actions.appendChild(this.createPanelButton('x', 'Close panel', () => this.closePanel(leaf.id)));
        return runtime;
    }

    createExplorerRuntime(leaf: Extract<WorkspaceNode, { kind: 'leaf' }>): ExplorerRuntime {
        const shell = this.createLeafRoot(leaf.id, 'explorer', 'Explorer');
        const contentEl = document.createElement('div');
        contentEl.className = 'panel-content explorer-content';
        shell.rootEl.appendChild(contentEl);

        const runtime: ExplorerRuntime = {
            leafId: leaf.id,
            panelKind: 'explorer',
            rootEl: shell.rootEl,
            headerTitleEl: shell.titleEl,
            contentEl,
            explorer: new FileExplorer(contentEl, this.vfs, (path) => this.openFileInFocusedEditor(path)),
        };

        shell.actions.appendChild(this.createPanelButton('Rf', 'Refresh explorer', () => runtime.explorer.refresh()));
        shell.actions.appendChild(this.createPanelButton('x', 'Close panel', () => this.closePanel(leaf.id)));
        runtime.explorer.render();
        return runtime;
    }

    setFocusedLeaf(leafId: string) {
        if (!findNode(this.snapshot.root, leafId)) {
            return;
        }
        this.snapshot.focusedLeafId = leafId;
        for (const [runtimeLeafId, runtime] of this.leafRuntimeMap.entries()) {
            if (runtimeLeafId === leafId) {
                continue;
            }
            if (runtime.panelKind === 'editor') {
                runtime.editor.blur();
            } else if (runtime.panelKind === 'terminal') {
                runtime.terminal.blur();
            }
        }
        this.syncFocusedClasses();
        this.syncStatusBar();
        this.saveWorkspaceSnapshot();
    }

    syncFocusedClasses() {
        for (const [leafId, runtime] of this.leafRuntimeMap.entries()) {
            runtime.rootEl.classList.toggle('focused', this.snapshot.focusedLeafId === leafId);
        }
    }

    syncStatusBar() {
        const runtime = this.getFocusedEditorRuntime();
        if (!runtime) {
            this.cursorSpan.textContent = 'No editor';
            this.analysisSpan.textContent = '';
            return;
        }
        const rawEditor = runtime.editor.getRawEditor();
        const index = rawEditor.cursor || 0;
        const pos = runtime.editor.getCursorPosition(index);
        this.cursorSpan.textContent = `Ln ${pos.row + 1}, Col ${pos.col + 1}`;
        const insight = runtime.editor.getTokenInsight(index);
        if (!insight) {
            this.analysisSpan.textContent = '';
            return;
        }
        const parts: string[] = [];
        if (insight.inferredType) parts.push(`<${insight.inferredType}>`);
        if (insight.resolvedDefinition) {
            parts.push(`${insight.resolvedDefinition.kind}:${insight.resolvedDefinition.name}`);
        }
        this.analysisSpan.textContent = parts.join(' | ');
    }

    resizeAll() {
        for (const runtime of this.leafRuntimeMap.values()) {
            if (runtime.panelKind === 'editor') {
                runtime.editor.resizeEditor();
            } else if (runtime.panelKind === 'terminal') {
                runtime.terminal.resizeEditor();
            }
        }
    }

    setFontSize(size: number) {
        this.currentFontSize = size;
        for (const runtime of this.leafRuntimeMap.values()) {
            if (runtime.panelKind === 'editor') {
                runtime.editor.setFontSize(size);
            } else if (runtime.panelKind === 'terminal') {
                runtime.terminal.setFontSize(size);
            }
        }
    }

    focusDefaultEditor() {
        const runtime = this.getFocusedEditorRuntime() || this.getFirstEditorRuntime();
        if (runtime) {
            this.setFocusedLeaf(runtime.leafId);
            runtime.editor.focus();
        }
    }

    getFocusedEditorRuntime(): EditorRuntime | null {
        const focused = this.snapshot.focusedLeafId ? this.leafRuntimeMap.get(this.snapshot.focusedLeafId) : null;
        if (focused && focused.panelKind === 'editor') {
            return focused;
        }
        return this.getFirstEditorRuntime();
    }

    getFirstEditorRuntime(): EditorRuntime | null {
        for (const runtime of this.leafRuntimeMap.values()) {
            if (runtime.panelKind === 'editor') {
                return runtime;
            }
        }
        return null;
    }

    getFocusedTerminalRuntime(): TerminalRuntime | null {
        const focused = this.snapshot.focusedLeafId ? this.leafRuntimeMap.get(this.snapshot.focusedLeafId) : null;
        if (focused && focused.panelKind === 'terminal') {
            return focused;
        }
        for (const runtime of this.leafRuntimeMap.values()) {
            if (runtime.panelKind === 'terminal') {
                return runtime;
            }
        }
        return null;
    }

    ensureEditorLeaf(): string {
        const existing = this.getFirstEditorRuntime();
        if (existing) {
            return existing.leafId;
        }
        const firstLeaf = collectLeaves(this.snapshot.root)[0];
        const newLeaf = createLeaf('editor');
        this.snapshot.root = splitLeaf(this.snapshot.root, firstLeaf.id, 'h', newLeaf, 'after');
        this.snapshot.focusedLeafId = newLeaf.id;
        this.redraw();
        return newLeaf.id;
    }

    ensureTerminalLeaf(): string {
        const existing = this.getFocusedTerminalRuntime();
        if (existing) {
            return existing.leafId;
        }
        const editor = this.getFocusedEditorRuntime() || this.getFirstEditorRuntime();
        const targetId = editor ? editor.leafId : this.ensureEditorLeaf();
        const newLeaf = createLeaf('terminal');
        this.snapshot.root = splitLeaf(this.snapshot.root, targetId, 'v', newLeaf, 'after');
        this.snapshot.focusedLeafId = newLeaf.id;
        this.redraw();
        return newLeaf.id;
    }

    openFileInFocusedEditor(path: string) {
        const editorRuntime = this.getFocusedEditorRuntime() || this.leafRuntimeMap.get(this.ensureEditorLeaf());
        if (!editorRuntime || editorRuntime.panelKind !== 'editor') {
            return;
        }
        this.setFocusedLeaf(editorRuntime.leafId);
        editorRuntime.tabManager.openFile(path);
        editorRuntime.editor.focus();
    }

    splitPanel(leafId: string, dir: 'h' | 'v') {
        const location = findNode(this.snapshot.root, leafId);
        if (!location || location.node.kind !== 'leaf' || location.node.panelKind === 'explorer') {
            return;
        }
        const newLeaf = createLeaf(location.node.panelKind);
        const activePath = location.node.activePath || null;
        this.snapshot.root = splitLeaf(this.snapshot.root, leafId, dir, newLeaf, 'after');
        this.snapshot.focusedLeafId = newLeaf.id;
        this.redraw();
        const runtime = this.leafRuntimeMap.get(newLeaf.id);
        if (runtime && runtime.panelKind === 'editor' && activePath) {
            runtime.tabManager.openFile(activePath);
            runtime.editor.focus();
        }
    }

    closePanel(leafId: string) {
        const location = findNode(this.snapshot.root, leafId);
        if (!location || location.node.kind !== 'leaf') {
            return;
        }
        const runtime = this.leafRuntimeMap.get(leafId);
        if (runtime?.panelKind === 'editor') {
            runtime.tabManager.saveCurrentTab();
        }
        if (location.node.panelKind === 'editor' && countLeavesByKind(this.snapshot.root, 'editor') <= 1) {
            return;
        }
        if (location.node.panelKind === 'explorer' && countLeavesByKind(this.snapshot.root, 'explorer') <= 1) {
            return;
        }
        this.snapshot.root = closeLeaf(this.snapshot.root, leafId);
        const nextLeaf = collectLeaves(this.snapshot.root)[0];
        this.snapshot.focusedLeafId = nextLeaf ? nextLeaf.id : null;
        this.redraw();
    }

    handleDragStart(event: DragEvent, leafId: string) {
        this.dragSourceLeafId = leafId;
        if (event.dataTransfer) {
            event.dataTransfer.effectAllowed = 'move';
            event.dataTransfer.setData('text/plain', leafId);
        }
    }

    computeDropZone(event: DragEvent, panelEl: HTMLElement): DropZone {
        const rect = panelEl.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        const horizontal = rect.width * 0.25;
        const vertical = rect.height * 0.25;
        if (x < horizontal) return 'left';
        if (x > rect.width - horizontal) return 'right';
        if (y < vertical) return 'top';
        if (y > rect.height - vertical) return 'bottom';
        return 'center';
    }

    handlePanelDragOver(event: DragEvent, leafId: string) {
        if (!this.dragSourceLeafId || this.dragSourceLeafId === leafId) {
            return;
        }
        event.preventDefault();
        const runtime = this.leafRuntimeMap.get(leafId);
        if (!runtime) {
            return;
        }
        const zone = this.computeDropZone(event, runtime.rootEl);
        this.setDropHighlight(leafId, zone);
    }

    setDropHighlight(leafId: string, zone: DropZone) {
        this.clearAllDropHighlights();
        const runtime = this.leafRuntimeMap.get(leafId);
        if (runtime) {
            runtime.rootEl.classList.add(`panel-drop-${zone}`);
        }
    }

    clearDropHighlight(leafId: string) {
        const runtime = this.leafRuntimeMap.get(leafId);
        if (runtime) {
            runtime.rootEl.classList.remove('panel-drop-left', 'panel-drop-right', 'panel-drop-top', 'panel-drop-bottom', 'panel-drop-center');
        }
    }

    clearAllDropHighlights() {
        for (const runtime of this.leafRuntimeMap.values()) {
            runtime.rootEl.classList.remove('panel-drop-left', 'panel-drop-right', 'panel-drop-top', 'panel-drop-bottom', 'panel-drop-center');
        }
    }

    handlePanelDrop(event: DragEvent, targetLeafId: string) {
        event.preventDefault();
        const sourceLeafId = this.dragSourceLeafId;
        this.dragSourceLeafId = null;
        this.clearAllDropHighlights();
        if (!sourceLeafId || sourceLeafId === targetLeafId) {
            return;
        }
        const sourceRuntime = this.leafRuntimeMap.get(sourceLeafId);
        const targetRuntime = this.leafRuntimeMap.get(targetLeafId);
        if (!sourceRuntime || !targetRuntime) {
            return;
        }
        const zone = this.computeDropZone(event, targetRuntime.rootEl);
        if (zone === 'center' && sourceRuntime.panelKind === 'editor' && targetRuntime.panelKind === 'editor') {
            targetRuntime.tabManager.mergeFrom(sourceRuntime.tabManager);
            this.closePanel(sourceLeafId);
            this.setFocusedLeaf(targetLeafId);
            return;
        }
        this.snapshot.root = moveLeaf(this.snapshot.root, sourceLeafId, targetLeafId, zone);
        this.snapshot.focusedLeafId = sourceLeafId;
        this.redraw();
    }

    startResize(event: MouseEvent, splitId: string, dir: 'h' | 'v') {
        event.preventDefault();
        const handle = event.currentTarget as HTMLElement;
        const splitNode = handle.parentElement;
        if (!splitNode) {
            return;
        }
        this.resizeState = {
            splitId,
            dir,
            rect: splitNode.getBoundingClientRect(),
        };
        handle.classList.add('active');
        document.body.classList.add('is-resizing-panels');
    }

    handleResizeMove(event: MouseEvent) {
        if (!this.resizeState) {
            return;
        }
        const split = findNode(this.snapshot.root, this.resizeState.splitId)?.node;
        if (!split || split.kind !== 'split') {
            return;
        }
        const ratio = this.resizeState.dir === 'h'
            ? (event.clientX - this.resizeState.rect.left) / this.resizeState.rect.width
            : (event.clientY - this.resizeState.rect.top) / this.resizeState.rect.height;
        split.ratio = clampSplitRatio(ratio);
        this.applySplitRatio(split.id, split.ratio);
        this.resizeAll();
    }

    stopResize() {
        if (!this.resizeState) {
            return;
        }
        document.querySelectorAll('.pane-split.active').forEach((node) => node.classList.remove('active'));
        document.body.classList.remove('is-resizing-panels');
        this.resizeState = null;
        this.saveWorkspaceSnapshot();
    }

    getActiveEditorTabPath(): string | null {
        const runtime = this.getFocusedEditorRuntime();
        return runtime?.tabManager.activeTab?.path || null;
    }

    saveFocusedEditorTab() {
        const runtime = this.getFocusedEditorRuntime();
        if (runtime) {
            runtime.tabManager.saveCurrentTab();
        }
    }

    executeInFocusedTerminal(command: string) {
        const terminalLeafId = this.ensureTerminalLeaf();
        const runtime = this.leafRuntimeMap.get(terminalLeafId);
        if (!runtime || runtime.panelKind !== 'terminal') {
            return;
        }
        this.setFocusedLeaf(terminalLeafId);
        runtime.terminal.currentInput = command;
        runtime.terminal.execute();
        runtime.terminal.focus();
        this.terminalStatusSpan.textContent = 'wasi-target';
    }

    showEditorHelp(anchorRect: DOMRect) {
        const editor = this.getFocusedEditorRuntime();
        if (!editor) {
            return;
        }
        const guide = [
            'Editor guide',
            '',
            'Hover: pause pointer for 1s to inspect expression and type',
            'Definition jump: F12',
            'Completion: Enter or Tab to accept',
            'Toggle comment: Ctrl+/',
            'Panel split: use R and D in the panel header',
        ].join('\n');
        editor.editor.showPopup(guide, anchorRect.left, anchorRect.bottom + 8);
        editor.editor.focus();
    }
}
