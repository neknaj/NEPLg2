import { createPlaygroundEditor, PlaygroundEditor } from '../editor-core/browser-adapter.js';
import {
    mapAnalysisSpanToTextRange,
    type AnalysisTextRange,
    type DefinitionLocation,
} from '../editor-core/language-analysis.js';
import { installGuiWebRuntimeBridge, registerGuiWebRuntimePresenter } from '../gui-preview/runtime-bridge.js';
import { GuiFloatingWindowManager } from '../gui-preview/window-manager.js';
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
    normalizeWorkspaceSnapshot,
    normalizeTree,
    PanelKind,
    splitLeaf,
    WorkspaceNode,
    WorkspaceSnapshot,
} from './panel-layout.js';
import {
    PanelDragPayload,
    TabDragPayload,
    ExplorerFileDragPayload,
    WorkspaceDragPayload,
    resolveTabbarDropAction,
} from './drag-drop.js';

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
    zoomBadgeEl: HTMLElement;
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
    zoomBadgeEl: HTMLElement;
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

type DefinitionNavigationRange = AnalysisTextRange;

type PanelManagerOptions = {
    root: HTMLElement;
    guiWindowLayer: HTMLElement;
    popup: HTMLElement;
    vfs: any;
    createNeplProvider: () => any;
    getCompilerMode?: () => string;
    cursorSpan: HTMLElement;
    analysisSpan: HTMLElement;
    terminalStatusSpan: HTMLElement;
};

const WORKSPACE_STORAGE_KEY = 'neplg2-playground-workspace-v1';

function normalizeDefinitionTargetPath(location: DefinitionLocation | null | undefined): string | null {
    const value = location?.targetPath;
    if (typeof value !== 'string') {
        return null;
    }
    let normalized = value.trim().replace(/\\/g, '/');
    while (normalized.length > 1 && normalized.endsWith('/')) {
        normalized = normalized.slice(0, -1);
    }
    return normalized.length > 0 ? normalized : null;
}

function normalizeDefinitionNavigationRange(range: DefinitionLocation['targetRange']): DefinitionNavigationRange | null {
    const start = Number(range?.startIndex);
    const end = Number(range?.endIndex ?? start);
    if (!Number.isFinite(start) || !Number.isFinite(end)) {
        return null;
    }
    const startIndex = Math.max(0, Math.trunc(start));
    const endIndex = Math.max(startIndex, Math.trunc(end));
    return { startIndex, endIndex };
}

export class PlaygroundPanelManager {
    root: HTMLElement;
    floatingGui: GuiFloatingWindowManager;
    popup: HTMLElement;
    vfs: any;
    createNeplProvider: () => any;
    getCompilerMode: () => string;
    cursorSpan: HTMLElement;
    analysisSpan: HTMLElement;
    terminalStatusSpan: HTMLElement;
    snapshot: WorkspaceSnapshot;
    leafRuntimeMap: Map<string, LeafRuntime>;
    splitDomMap: Map<string, { first: HTMLElement; second: HTMLElement }>;
    dragPayload: WorkspaceDragPayload | null;
    resizeState: { splitId: string; dir: 'h' | 'v'; rect: DOMRect } | null;
    currentFontSize: number;
    zoomBadgeTimerMap: Map<string, number>;
    pinchState: { leafId: string; initialZoom: number; initialDist: number } | null;
    analysisInsightTimer: number | null;
    analysisInsightVersion: number;

    constructor(options: PanelManagerOptions) {
        this.root = options.root;
        this.floatingGui = new GuiFloatingWindowManager(options.guiWindowLayer);
        registerGuiWebRuntimePresenter(this.floatingGui);
        const guiRuntimeInstall = installGuiWebRuntimeBridge(globalThis);
        if (guiRuntimeInstall.kind === 'err') {
            console.warn('[Playground] Failed to install GUI runtime bridge', guiRuntimeInstall.error);
        }
        this.popup = options.popup;
        this.vfs = options.vfs;
        this.createNeplProvider = options.createNeplProvider;
        this.getCompilerMode = options.getCompilerMode || (() => 'rust');
        this.cursorSpan = options.cursorSpan;
        this.analysisSpan = options.analysisSpan;
        this.terminalStatusSpan = options.terminalStatusSpan;
        this.snapshot = this.loadWorkspaceSnapshot();
        this.leafRuntimeMap = new Map();
        this.splitDomMap = new Map();
        this.dragPayload = null;
        this.resizeState = null;
        this.currentFontSize = 14;
        this.zoomBadgeTimerMap = new Map();
        this.pinchState = null;
        this.analysisInsightTimer = null;
        this.analysisInsightVersion = 0;
        this.bindWindowEvents();
    }

    bindWindowEvents() {
        window.addEventListener('mousemove', (event) => this.handleResizeMove(event));
        window.addEventListener('mouseup', () => this.stopResize());
        document.addEventListener('dragend', () => {
            this.dragPayload = null;
            this.clearAllDropHighlights();
        });
        document.addEventListener('wheel', (event) => this.handleZoomWheel(event), { passive: false });
        document.addEventListener('touchstart', (event) => this.handlePinchStart(event), { passive: false });
        document.addEventListener('touchmove', (event) => this.handlePinchMove(event), { passive: false });
        document.addEventListener('touchend', () => this.endPinch());
        document.addEventListener('touchcancel', () => this.endPinch());
        document.addEventListener('keydown', (event) => this.handleZoomShortcut(event));
    }

    loadWorkspaceSnapshot(): WorkspaceSnapshot {
        try {
            const raw = localStorage.getItem(WORKSPACE_STORAGE_KEY);
            if (raw) {
                const parsed = JSON.parse(raw) as WorkspaceSnapshot;
                return normalizeWorkspaceSnapshot(parsed);
            }
        } catch (error) {
            console.warn('[Playground] Failed to restore workspace snapshot', error);
        }
        const snapshot = createDefaultWorkspace();
        hydratePanelCounter(snapshot.root);
        return snapshot;
    }

    resetWorkspaceLayout() {
        this.snapshot = createDefaultWorkspace();
        hydratePanelCounter(this.snapshot.root);
        this.redraw();
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
                leaf.pathZooms = tabState.pathZooms;
                leaf.zoom = runtime.tabManager.getActiveZoom();
            } else {
                leaf.paths = [];
                leaf.activePath = null;
                if (runtime.panelKind === 'terminal') {
                    leaf.zoom = this.resolveLeafZoom(leaf.id);
                }
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
        const zoomBadgeEl = document.createElement('div');
        zoomBadgeEl.className = 'panel-zoom-badge';
        rootEl.appendChild(zoomBadgeEl);
        return { rootEl, header, titleEl, actions, zoomBadgeEl };
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

    clampPanelZoom(value: number): number {
        const next = Number.isFinite(value) ? value : 1;
        return Math.max(0.6, Math.min(2.4, Math.round(next * 100) / 100));
    }

    resolveLeafZoom(leafId: string): number {
        const leaf = findNode(this.snapshot.root, leafId)?.node;
        if (!leaf || leaf.kind !== 'leaf') {
            return 1;
        }
        if (leaf.panelKind === 'editor') {
            const activePath = leaf.activePath || null;
            if (activePath && leaf.pathZooms && Number.isFinite(leaf.pathZooms[activePath])) {
                return Number(leaf.pathZooms[activePath]);
            }
        }
        return Number.isFinite(leaf.zoom) ? Number(leaf.zoom) : 1;
    }

    setLeafZoom(leafId: string, zoom: number, options: { showBadge?: boolean } = {}) {
        const leaf = findNode(this.snapshot.root, leafId)?.node;
        const runtime = this.leafRuntimeMap.get(leafId);
        if (!leaf || leaf.kind !== 'leaf' || !runtime) {
            return false;
        }
        const clampedZoom = this.clampPanelZoom(zoom);
        if (leaf.panelKind === 'editor') {
            leaf.zoom = clampedZoom;
            const editorRuntime = runtime as EditorRuntime;
            const activePath = editorRuntime.tabManager.activeTab?.path || leaf.activePath || null;
            if (activePath) {
                leaf.pathZooms = leaf.pathZooms || {};
                leaf.pathZooms[activePath] = clampedZoom;
                editorRuntime.tabManager.setActiveZoom(clampedZoom);
            }
            editorRuntime.editor.setFontSize(Math.round(this.currentFontSize * clampedZoom));
            if (options.showBadge !== false) {
                this.showZoomBadge(leafId, clampedZoom);
            }
            this.saveWorkspaceSnapshot();
            return true;
        }
        if (leaf.panelKind === 'terminal') {
            leaf.zoom = clampedZoom;
            const terminalRuntime = runtime as TerminalRuntime;
            terminalRuntime.terminal.setFontSize(Math.round(this.currentFontSize * clampedZoom));
            if (options.showBadge !== false) {
                this.showZoomBadge(leafId, clampedZoom);
            }
            this.saveWorkspaceSnapshot();
            return true;
        }
        return false;
    }

    showZoomBadge(leafId: string, zoom: number) {
        const runtime = this.leafRuntimeMap.get(leafId);
        if (!runtime || runtime.panelKind === 'explorer') {
            return;
        }
        const badgeEl = runtime.zoomBadgeEl;
        badgeEl.textContent = `${Math.round(zoom * 100)}%`;
        badgeEl.classList.add('visible');
        const previousTimer = this.zoomBadgeTimerMap.get(leafId);
        if (previousTimer) {
            window.clearTimeout(previousTimer);
        }
        const nextTimer = window.setTimeout(() => {
            badgeEl.classList.remove('visible');
            this.zoomBadgeTimerMap.delete(leafId);
        }, 900);
        this.zoomBadgeTimerMap.set(leafId, nextTimer);
    }

    applyLeafZoom(leafId: string, options: { showBadge?: boolean } = {}) {
        const leaf = findNode(this.snapshot.root, leafId)?.node;
        if (!leaf || leaf.kind !== 'leaf') {
            return;
        }
        const zoom = this.resolveLeafZoom(leafId);
        if (leaf.panelKind === 'editor') {
            const runtime = this.leafRuntimeMap.get(leafId);
            if (runtime && runtime.panelKind === 'editor') {
                runtime.editor.setFontSize(Math.round(this.currentFontSize * zoom));
                if (options.showBadge) {
                    this.showZoomBadge(leafId, zoom);
                }
            }
            return;
        }
        if (leaf.panelKind === 'terminal') {
            const runtime = this.leafRuntimeMap.get(leafId);
            if (runtime && runtime.panelKind === 'terminal') {
                runtime.terminal.setFontSize(Math.round(this.currentFontSize * zoom));
                if (options.showBadge) {
                    this.showZoomBadge(leafId, zoom);
                }
            }
        }
    }

    adjustZoomForLeaf(leafId: string, delta: number, reset = false): boolean {
        const leaf = findNode(this.snapshot.root, leafId)?.node;
        if (!leaf || leaf.kind !== 'leaf' || leaf.panelKind === 'explorer') {
            return false;
        }
        const nextZoom = reset ? 1 : this.resolveLeafZoom(leafId) + delta;
        return this.setLeafZoom(leafId, nextZoom, { showBadge: true });
    }

    getLeafIdFromEventTarget(target: EventTarget | null): string | null {
        const element = target instanceof HTMLElement ? target : null;
        const panelEl = element ? element.closest('.panel') : null;
        return panelEl?.getAttribute('data-panel-id') || null;
    }

    pinchDistance(touches: TouchList): number {
        const dx = touches[0].clientX - touches[1].clientX;
        const dy = touches[0].clientY - touches[1].clientY;
        return Math.hypot(dx, dy);
    }

    handleZoomWheel(event: WheelEvent) {
        if (!event.ctrlKey) {
            return;
        }
        const leafId = this.getLeafIdFromEventTarget(event.target);
        if (!leafId) {
            return;
        }
        const leaf = findNode(this.snapshot.root, leafId)?.node;
        if (!leaf || leaf.kind !== 'leaf' || leaf.panelKind === 'explorer') {
            return;
        }
        this.setFocusedLeaf(leafId);
        const delta = event.deltaY < 0 ? 0.1 : -0.1;
        if (this.adjustZoomForLeaf(leafId, delta)) {
            event.preventDefault();
        }
    }

    handlePinchStart(event: TouchEvent) {
        if (event.touches.length !== 2) {
            return;
        }
        const leafId = this.getLeafIdFromEventTarget(event.target);
        if (!leafId) {
            return;
        }
        const leaf = findNode(this.snapshot.root, leafId)?.node;
        if (!leaf || leaf.kind !== 'leaf' || leaf.panelKind === 'explorer') {
            return;
        }
        event.preventDefault();
        this.setFocusedLeaf(leafId);
        this.pinchState = {
            leafId,
            initialZoom: this.resolveLeafZoom(leafId),
            initialDist: this.pinchDistance(event.touches),
        };
    }

    handlePinchMove(event: TouchEvent) {
        if (!this.pinchState || event.touches.length !== 2) {
            return;
        }
        event.preventDefault();
        const zoom = this.clampPanelZoom(this.pinchState.initialZoom * (this.pinchDistance(event.touches) / this.pinchState.initialDist));
        this.setLeafZoom(this.pinchState.leafId, zoom, { showBadge: true });
    }

    endPinch() {
        this.pinchState = null;
    }

    handleZoomShortcut(event: KeyboardEvent) {
        if (!event.ctrlKey && !event.metaKey) {
            return;
        }
        const leafId = this.snapshot.focusedLeafId;
        if (!leafId) {
            return;
        }
        if (event.key === '=' || event.key === '+') {
            if (this.adjustZoomForLeaf(leafId, 0.1)) {
                event.preventDefault();
            }
        } else if (event.key === '-') {
            if (this.adjustZoomForLeaf(leafId, -0.1)) {
                event.preventDefault();
            }
        } else if (event.key === '0') {
            if (this.adjustZoomForLeaf(leafId, 0, true)) {
                event.preventDefault();
            }
        }
    }

    createEditorRuntime(leaf: Extract<WorkspaceNode, { kind: 'leaf' }>): EditorRuntime {
        const shell = this.createLeafRoot(leaf.id, 'editor', 'Editor');
        const tabbarEl = document.createElement('div');
        tabbarEl.className = 'tabbar';
        tabbarEl.addEventListener('dragover', (event) => this.handleTabbarDragOver(event, leaf.id));
        tabbarEl.addEventListener('dragleave', () => this.clearTabbarDropHighlight(leaf.id));
        tabbarEl.addEventListener('drop', (event) => this.handleTabbarDrop(event, leaf.id));
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
            zoomBadgeEl: shell.zoomBadgeEl,
            editor: null as unknown as PlaygroundEditor,
            tabManager: null as unknown as TabManager,
        };

        runtime.editor = createPlaygroundEditor({
            canvas,
            textarea,
            popup: this.popup,
            problemsPanel: null,
            completionList,
            languageProviders: { nepl: this.createNeplProvider() },
            initialLanguage: 'nepl',
            onCursorChange: (index: number) => {
                if (this.snapshot.focusedLeafId !== leaf.id) {
                    return;
                }
                const pos = runtime.editor.getCursorPosition(index);
                this.cursorSpan.textContent = `Ln ${pos.row + 1}, Col ${pos.col + 1}`;
                this.scheduleAnalysisInsight(runtime, index);
            },
            onDefinitionNavigation: (location: unknown) => {
                this.openDefinitionTarget(location as DefinitionLocation);
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
                    targetLeaf.pathZooms = state.pathZooms;
                    targetLeaf.zoom = runtime.tabManager.getActiveZoom();
                }
                this.saveWorkspaceSnapshot();
                if (this.snapshot.focusedLeafId === leaf.id) {
                    this.syncStatusBar();
                }
            },
            onActiveTabChange: () => {
                const targetLeaf = findNode(this.snapshot.root, leaf.id)?.node;
                if (targetLeaf && targetLeaf.kind === 'leaf') {
                    targetLeaf.zoom = runtime.tabManager.getActiveZoom();
                }
                this.applyLeafZoom(leaf.id);
            },
            onTabDragStart: ({ path, event }) => {
                this.setDragPayload(event, { kind: 'editor-tab', leafId: leaf.id, path });
            },
        });

        shell.actions.appendChild(this.createPanelButton('R', 'Split right', () => this.splitPanel(leaf.id, 'h')));
        shell.actions.appendChild(this.createPanelButton('D', 'Split down', () => this.splitPanel(leaf.id, 'v')));
        shell.actions.appendChild(this.createPanelButton('x', 'Close panel', () => this.closePanel(leaf.id)));

        runtime.tabManager.restoreTabs(leaf.paths || [], leaf.activePath || null, leaf.pathZooms || {});
        runtime.editor.setFontSize(Math.round(this.currentFontSize * this.resolveLeafZoom(leaf.id)));
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

        const terminal = new CanvasTerminal(canvas, textarea, null, {
            vfs: this.vfs,
            getCompilerMode: this.getCompilerMode,
        });
        terminal.setFontSize(this.currentFontSize);

        const runtime: TerminalRuntime = {
            leafId: leaf.id,
            panelKind: 'terminal',
            rootEl: shell.rootEl,
            headerTitleEl: shell.titleEl,
            contentEl,
            canvas,
            textarea,
            zoomBadgeEl: shell.zoomBadgeEl,
            terminal,
        };

        shell.actions.appendChild(this.createPanelButton('R', 'Split right', () => this.splitPanel(leaf.id, 'h')));
        shell.actions.appendChild(this.createPanelButton('D', 'Split down', () => this.splitPanel(leaf.id, 'v')));
        shell.actions.appendChild(this.createPanelButton('x', 'Close panel', () => this.closePanel(leaf.id)));
        terminal.setFontSize(Math.round(this.currentFontSize * this.resolveLeafZoom(leaf.id)));
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
            explorer: new FileExplorer(
                contentEl,
                this.vfs,
                (path) => this.openFileInFocusedEditor(path),
                {
                    onFileDragStart: (path, event) => {
                        this.setDragPayload(event, { kind: 'explorer-file', path });
                    },
                },
            ),
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
            this.cancelAnalysisInsight();
            return;
        }
        const rawEditor = runtime.editor.getRawEditor();
        const index = rawEditor.cursor || 0;
        const pos = runtime.editor.getCursorPosition(index);
        this.cursorSpan.textContent = `Ln ${pos.row + 1}, Col ${pos.col + 1}`;
        this.scheduleAnalysisInsight(runtime, index, 0);
    }

    cancelAnalysisInsight() {
        this.analysisInsightVersion += 1;
        if (this.analysisInsightTimer !== null) {
            window.clearTimeout(this.analysisInsightTimer);
            this.analysisInsightTimer = null;
        }
    }

    scheduleAnalysisInsight(runtime: EditorRuntime, index: number, delayMs = 55) {
        this.cancelAnalysisInsight();
        const version = this.analysisInsightVersion;
        this.analysisInsightTimer = window.setTimeout(() => {
            this.analysisInsightTimer = null;
            if (version !== this.analysisInsightVersion) {
                return;
            }
            if (this.snapshot.focusedLeafId !== runtime.leafId) {
                return;
            }
            this.updateAnalysisInsight(runtime, index);
        }, Math.max(0, delayMs));
    }

    updateAnalysisInsight(runtime: EditorRuntime, index: number) {
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
        this.floatingGui.resizeAll();
    }

    setFontSize(size: number) {
        this.currentFontSize = size;
        for (const runtime of this.leafRuntimeMap.values()) {
            this.applyLeafZoom(runtime.leafId);
        }
        this.floatingGui.setFontSize(size);
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

    stopActiveProcess(): boolean {
        const focused = this.snapshot.focusedLeafId ? this.leafRuntimeMap.get(this.snapshot.focusedLeafId) : null;
        if (focused && focused.panelKind === 'terminal' && focused.terminal.shell.isRunning) {
            focused.terminal.shell.interrupt();
            return true;
        }
        for (const runtime of this.leafRuntimeMap.values()) {
            if (runtime.panelKind === 'terminal' && runtime.terminal.shell.isRunning) {
                runtime.terminal.shell.interrupt();
                return true;
            }
        }
        return false;
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

    openDefinitionTarget(location: DefinitionLocation | null | undefined): boolean {
        if (!location) {
            return false;
        }
        const targetPath = normalizeDefinitionTargetPath(location);
        if (!targetPath || !this.canOpenDefinitionTargetPath(targetPath)) {
            return false;
        }
        const editorRuntime = this.getFocusedEditorRuntime() || this.leafRuntimeMap.get(this.ensureEditorLeaf());
        if (!editorRuntime || editorRuntime.panelKind !== 'editor') {
            return false;
        }
        this.setFocusedLeaf(editorRuntime.leafId);
        editorRuntime.tabManager.openFile(targetPath);
        const range = this.resolveDefinitionNavigationRange(editorRuntime, targetPath, location);
        if (range) {
            editorRuntime.editor.moveCursorToRange(range);
        }
        editorRuntime.editor.focus();
        return true;
    }

    canOpenDefinitionTargetPath(path: string): boolean {
        if (!this.vfs || typeof this.vfs.exists !== 'function') {
            return true;
        }
        return this.vfs.exists(path) === true;
    }

    resolveDefinitionNavigationRange(editorRuntime: EditorRuntime, targetPath: string, location: DefinitionLocation): DefinitionNavigationRange | null {
        const directRange = normalizeDefinitionNavigationRange(location.targetRange);
        if (directRange) {
            return directRange;
        }
        const text = typeof editorRuntime.editor.getText === 'function' ? editorRuntime.editor.getText() : '';
        return mapAnalysisSpanToTextRange(text, location.targetSpan ?? null, targetPath);
    }

    splitPanel(leafId: string, dir: 'h' | 'v') {
        this.syncSnapshotFromRuntimes();
        const location = findNode(this.snapshot.root, leafId);
        if (!location || location.node.kind !== 'leaf' || location.node.panelKind === 'explorer') {
            return;
        }
        const newLeaf = createLeaf(location.node.panelKind);
        newLeaf.zoom = this.resolveLeafZoom(leafId);
        if (location.node.panelKind === 'editor' && location.node.activePath) {
            newLeaf.pathZooms = { [location.node.activePath]: this.resolveLeafZoom(leafId) };
        }
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
        this.setDragPayload(event, { kind: 'panel', leafId });
    }

    setDragPayload(event: DragEvent, payload: WorkspaceDragPayload) {
        this.dragPayload = payload;
        if (!event.dataTransfer) {
            return;
        }
        event.dataTransfer.effectAllowed = 'move';
        event.dataTransfer.setData('application/x-nepl-workspace-drag', JSON.stringify(payload));
        const plainText = payload.kind === 'explorer-file'
            ? payload.path
            : payload.kind === 'panel'
                ? payload.leafId
                : payload.path;
        event.dataTransfer.setData('text/plain', plainText);
    }

    getDragPayload(event?: DragEvent): WorkspaceDragPayload | null {
        if (this.dragPayload) {
            return this.dragPayload;
        }
        if (!event?.dataTransfer) {
            return null;
        }
        const raw = event.dataTransfer.getData('application/x-nepl-workspace-drag');
        if (!raw) {
            return null;
        }
        try {
            return JSON.parse(raw) as WorkspaceDragPayload;
        } catch (error) {
            console.warn('[Playground] Failed to parse drag payload', error);
            return null;
        }
    }

    canDropPayloadOnLeaf(payload: WorkspaceDragPayload | null, targetLeafId: string, zone: DropZone): boolean {
        if (!payload) {
            return false;
        }
        const targetLocation = findNode(this.snapshot.root, targetLeafId);
        if (!targetLocation || targetLocation.node.kind !== 'leaf') {
            return false;
        }
        if (payload.kind === 'panel') {
            if (payload.leafId === targetLeafId) {
                return false;
            }
            if (zone === 'center') {
                const sourceRuntime = this.leafRuntimeMap.get(payload.leafId);
                const targetRuntime = this.leafRuntimeMap.get(targetLeafId);
                return sourceRuntime?.panelKind === 'editor' && targetRuntime?.panelKind === 'editor';
            }
            return true;
        }
        if (payload.kind === 'editor-tab') {
            if (zone === 'center') {
                return targetLocation.node.panelKind === 'editor';
            }
            if (payload.leafId === targetLeafId) {
                return true;
            }
            return true;
        }
        if (payload.kind === 'explorer-file') {
            if (zone === 'center') {
                return targetLocation.node.panelKind === 'editor';
            }
            return true;
        }
        return false;
    }

    ensureEditorLeafForDrop(targetLeafId: string, zone: DropZone): string | null {
        const targetLocation = findNode(this.snapshot.root, targetLeafId);
        if (!targetLocation || targetLocation.node.kind !== 'leaf') {
            return null;
        }
        if (zone === 'center') {
            return targetLocation.node.panelKind === 'editor' ? targetLeafId : null;
        }
        const dir: 'h' | 'v' = zone === 'left' || zone === 'right' ? 'h' : 'v';
        const place = zone === 'left' || zone === 'top' ? 'before' : 'after';
        const newLeaf = createLeaf('editor');
        this.snapshot.root = splitLeaf(this.snapshot.root, targetLeafId, dir, newLeaf, place);
        this.snapshot.focusedLeafId = newLeaf.id;
        this.redraw();
        return newLeaf.id;
    }

    moveDraggedTab(payload: TabDragPayload, targetLeafId: string, zone: DropZone) {
        const sourceRuntime = this.leafRuntimeMap.get(payload.leafId);
        if (!sourceRuntime || sourceRuntime.panelKind !== 'editor') {
            return;
        }
        if (payload.leafId === targetLeafId && zone === 'center') {
            sourceRuntime.tabManager.setActiveTab(sourceRuntime.tabManager.tabs.findIndex((tab) => tab.path === payload.path));
            return;
        }
        const destinationLeafId = this.ensureEditorLeafForDrop(targetLeafId, zone);
        if (!destinationLeafId) {
            return;
        }
        const tab = sourceRuntime.tabManager.detachTabByPath(payload.path);
        const targetRuntime = this.leafRuntimeMap.get(destinationLeafId);
        if (!tab || !targetRuntime || targetRuntime.panelKind !== 'editor') {
            return;
        }
        this.setFocusedLeaf(destinationLeafId);
        targetRuntime.tabManager.attachTab(tab, { activate: true, focusEditor: true });
        targetRuntime.editor.focus();
    }

    openDraggedFile(path: string, targetLeafId: string, zone: DropZone) {
        const destinationLeafId = this.ensureEditorLeafForDrop(targetLeafId, zone);
        if (!destinationLeafId) {
            return;
        }
        const targetRuntime = this.leafRuntimeMap.get(destinationLeafId);
        if (!targetRuntime || targetRuntime.panelKind !== 'editor') {
            return;
        }
        this.setFocusedLeaf(destinationLeafId);
        targetRuntime.tabManager.openFile(path);
        targetRuntime.editor.focus();
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
        const runtime = this.leafRuntimeMap.get(leafId);
        if (!runtime) {
            return;
        }
        const zone = this.computeDropZone(event, runtime.rootEl);
        const payload = this.getDragPayload(event);
        if (!this.canDropPayloadOnLeaf(payload, leafId, zone)) {
            this.clearDropHighlight(leafId);
            return;
        }
        event.preventDefault();
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
            if (runtime.panelKind === 'editor') {
                runtime.tabbarEl.classList.remove('tabbar-drop-merge');
            }
        }
    }

    setTabbarDropHighlight(leafId: string) {
        this.clearAllDropHighlights();
        const runtime = this.leafRuntimeMap.get(leafId);
        if (runtime && runtime.panelKind === 'editor') {
            runtime.tabbarEl.classList.add('tabbar-drop-merge');
        }
    }

    clearTabbarDropHighlight(leafId: string) {
        const runtime = this.leafRuntimeMap.get(leafId);
        if (runtime && runtime.panelKind === 'editor') {
            runtime.tabbarEl.classList.remove('tabbar-drop-merge');
        }
    }

    handleTabbarDragOver(event: DragEvent, targetLeafId: string) {
        const targetRuntime = this.leafRuntimeMap.get(targetLeafId);
        const payload = this.getDragPayload(event);
        const action = resolveTabbarDropAction(payload, targetRuntime?.panelKind || 'explorer');
        if (!action) {
            this.clearTabbarDropHighlight(targetLeafId);
            return;
        }
        event.preventDefault();
        event.stopPropagation();
        this.setTabbarDropHighlight(targetLeafId);
    }

    handleTabbarDrop(event: DragEvent, targetLeafId: string) {
        event.preventDefault();
        event.stopPropagation();
        const payload = this.getDragPayload(event);
        this.dragPayload = null;
        this.clearAllDropHighlights();
        const targetRuntime = this.leafRuntimeMap.get(targetLeafId);
        const action = resolveTabbarDropAction(payload, targetRuntime?.panelKind || 'explorer');
        if (!payload || !targetRuntime || targetRuntime.panelKind !== 'editor' || !action) {
            return;
        }
        if (action === 'attach-tab' && payload.kind === 'editor-tab') {
            this.moveDraggedTab(payload, targetLeafId, 'center');
            return;
        }
        if (action === 'open-file' && payload.kind === 'explorer-file') {
            this.openDraggedFile(payload.path, targetLeafId, 'center');
            return;
        }
        if (action === 'merge-panel' && payload.kind === 'panel') {
            const sourceRuntime = this.leafRuntimeMap.get(payload.leafId);
            if (!sourceRuntime || sourceRuntime.panelKind !== 'editor' || payload.leafId === targetLeafId) {
                return;
            }
            targetRuntime.tabManager.mergeFrom(sourceRuntime.tabManager);
            this.closePanel(payload.leafId);
            this.setFocusedLeaf(targetLeafId);
            targetRuntime.editor.focus();
        }
    }

    handlePanelDrop(event: DragEvent, targetLeafId: string) {
        event.preventDefault();
        const payload = this.getDragPayload(event);
        this.dragPayload = null;
        this.clearAllDropHighlights();
        if (!payload) {
            return;
        }
        const targetRuntime = this.leafRuntimeMap.get(targetLeafId);
        if (!targetRuntime) {
            return;
        }
        const zone = this.computeDropZone(event, targetRuntime.rootEl);
        if (!this.canDropPayloadOnLeaf(payload, targetLeafId, zone)) {
            return;
        }
        if (payload.kind === 'panel') {
            const sourceRuntime = this.leafRuntimeMap.get(payload.leafId);
            if (!sourceRuntime) {
                return;
            }
            if (zone === 'center' && sourceRuntime.panelKind === 'editor' && targetRuntime.panelKind === 'editor') {
                targetRuntime.tabManager.mergeFrom(sourceRuntime.tabManager);
                this.closePanel(payload.leafId);
                this.setFocusedLeaf(targetLeafId);
                return;
            }
            this.snapshot.root = moveLeaf(this.snapshot.root, payload.leafId, targetLeafId, zone);
            this.snapshot.focusedLeafId = payload.leafId;
            this.redraw();
            return;
        }
        if (payload.kind === 'editor-tab') {
            this.moveDraggedTab(payload, targetLeafId, zone);
            return;
        }
        if (payload.kind === 'explorer-file') {
            this.openDraggedFile(payload.path, targetLeafId, zone);
        }
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
        this.terminalStatusSpan.textContent = `wasi-target:${this.getCompilerMode()}`;
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
