export type PanelKind = 'explorer' | 'editor' | 'terminal' | 'gui-preview';
export type GuiPreviewKind = 'mandelbrot' | 'life' | 'counter';
export type SplitDirection = 'h' | 'v';
export type DropZone = 'left' | 'right' | 'top' | 'bottom' | 'center';

export interface LeafPanelSnapshot {
    kind: 'leaf';
    id: string;
    panelKind: PanelKind;
    activePath?: string | null;
    paths?: string[];
    zoom?: number;
    pathZooms?: Record<string, number>;
    previewKind?: GuiPreviewKind | null;
}

export interface SplitNodeSnapshot {
    kind: 'split';
    id: string;
    dir: SplitDirection;
    ratio: number;
    first: WorkspaceNode;
    second: WorkspaceNode;
}

export type WorkspaceNode = LeafPanelSnapshot | SplitNodeSnapshot;

export interface WorkspaceSnapshot {
    root: WorkspaceNode;
    focusedLeafId: string | null;
}

export interface NodeLocation {
    node: WorkspaceNode;
    parent: SplitNodeSnapshot | null;
    key: 'first' | 'second' | null;
}

export const MIN_SPLIT_RATIO = 0.18;
export const MAX_SPLIT_RATIO = 0.82;

let panelCounter = 0;
const PANEL_KINDS = new Set<string>(['explorer', 'editor', 'terminal', 'gui-preview']);
const GUI_PREVIEW_KINDS = new Set<string>(['mandelbrot', 'life', 'counter']);

export function isPanelKind(value: unknown): value is PanelKind {
    return typeof value === 'string' && PANEL_KINDS.has(value);
}

export function isGuiPreviewKind(value: unknown): value is GuiPreviewKind {
    return typeof value === 'string' && GUI_PREVIEW_KINDS.has(value);
}

export function hydratePanelCounter(root: WorkspaceNode | null): void {
    let maxValue = panelCounter;
    walkTree(root, (node) => {
        const match = String(node.id).match(/-(\d+)$/);
        if (match) {
            maxValue = Math.max(maxValue, Number(match[1]));
        }
    });
    panelCounter = maxValue;
}

export function createLeaf(panelKind: PanelKind): LeafPanelSnapshot {
    panelCounter += 1;
    return {
        kind: 'leaf',
        id: `panel-${panelCounter}`,
        panelKind,
        activePath: null,
        paths: [],
    };
}

export function createSplit(
    dir: SplitDirection,
    first: WorkspaceNode,
    second: WorkspaceNode,
    ratio = 0.5,
): SplitNodeSnapshot {
    panelCounter += 1;
    return {
        kind: 'split',
        id: `split-${panelCounter}`,
        dir,
        ratio: clampSplitRatio(ratio),
        first,
        second,
    };
}

export function clampSplitRatio(ratio: number): number {
    const value = Number.isFinite(ratio) ? ratio : 0.5;
    return Math.max(MIN_SPLIT_RATIO, Math.min(MAX_SPLIT_RATIO, value));
}

export function collectLeaves(root: WorkspaceNode | null): LeafPanelSnapshot[] {
    const leaves: LeafPanelSnapshot[] = [];
    walkTree(root, (node) => {
        if (node.kind === 'leaf') {
            leaves.push(node);
        }
    });
    return leaves;
}

export function walkTree(root: WorkspaceNode | null, visit: (node: WorkspaceNode) => void): void {
    if (!root) {
        return;
    }
    visit(root);
    if (root.kind === 'split') {
        walkTree(root.first, visit);
        walkTree(root.second, visit);
    }
}

export function countLeavesByKind(root: WorkspaceNode | null, panelKind: PanelKind): number {
    return collectLeaves(root).filter((leaf) => leaf.panelKind === panelKind).length;
}

export function findNode(root: WorkspaceNode | null, targetId: string): NodeLocation | null {
    return findNodeRecursive(root, targetId, null, null);
}

function findNodeRecursive(
    node: WorkspaceNode | null,
    targetId: string,
    parent: SplitNodeSnapshot | null,
    key: 'first' | 'second' | null,
): NodeLocation | null {
    if (!node) {
        return null;
    }
    if (node.id === targetId) {
        return { node, parent, key };
    }
    if (node.kind !== 'split') {
        return null;
    }
    return findNodeRecursive(node.first, targetId, node, 'first')
        || findNodeRecursive(node.second, targetId, node, 'second');
}

export function normalizeTree(root: WorkspaceNode | null): WorkspaceNode | null {
    if (!root) {
        return null;
    }
    if (root.kind === 'leaf') {
        if (!isPanelKind(root.panelKind)) {
            root.panelKind = 'editor';
        }
        root.paths = Array.isArray(root.paths) ? root.paths.filter(Boolean) : [];
        root.activePath = root.activePath && root.paths.includes(root.activePath) ? root.activePath : (root.paths[0] || null);
        root.zoom = Number.isFinite(root.zoom) ? Number(root.zoom) : 1;
        root.pathZooms = root.pathZooms && typeof root.pathZooms === 'object' ? { ...root.pathZooms } : {};
        root.previewKind = root.panelKind === 'gui-preview' && isGuiPreviewKind(root.previewKind)
            ? root.previewKind
            : null;
        return root;
    }
    root.first = normalizeTree(root.first)!;
    root.second = normalizeTree(root.second)!;
    root.ratio = clampSplitRatio(root.ratio);
    if (!root.first) {
        return root.second;
    }
    if (!root.second) {
        return root.first;
    }
    return root;
}

export function normalizeWorkspaceSnapshot(snapshot: Partial<WorkspaceSnapshot> | null | undefined): WorkspaceSnapshot {
    if (!snapshot || !snapshot.root) {
        const fallback = createDefaultWorkspace();
        hydratePanelCounter(fallback.root);
        return fallback;
    }
    const root = normalizeTree(snapshot.root) || createDefaultWorkspace().root;
    hydratePanelCounter(root);
    const requestedFocus = typeof snapshot.focusedLeafId === 'string' ? snapshot.focusedLeafId : null;
    const focusedLeafId = requestedFocus && findNode(root, requestedFocus)
        ? requestedFocus
        : resolveDefaultFocusedLeafId(root);
    return { root, focusedLeafId };
}

export function resolveDefaultFocusedLeafId(root: WorkspaceNode | null): string | null {
    const leaves = collectLeaves(root);
    return leaves.find((leaf) => leaf.panelKind === 'editor')?.id || leaves[0]?.id || null;
}

export function splitLeaf(
    root: WorkspaceNode,
    targetLeafId: string,
    dir: SplitDirection,
    newLeaf: LeafPanelSnapshot,
    place: 'before' | 'after',
): WorkspaceNode {
    const location = findNode(root, targetLeafId);
    if (!location || location.node.kind !== 'leaf') {
        return root;
    }
    const branch = place === 'before'
        ? createSplit(dir, newLeaf, location.node, 0.5)
        : createSplit(dir, location.node, newLeaf, 0.5);
    return replaceNode(root, targetLeafId, branch);
}

export function closeLeaf(root: WorkspaceNode, targetLeafId: string): WorkspaceNode {
    const location = findNode(root, targetLeafId);
    if (!location || location.node.kind !== 'leaf') {
        return root;
    }
    if (!location.parent || !location.key) {
        return root;
    }
    const sibling = location.key === 'first' ? location.parent.second : location.parent.first;
    return replaceNode(root, location.parent.id, sibling);
}

export function moveLeaf(root: WorkspaceNode, sourceLeafId: string, targetLeafId: string, zone: DropZone): WorkspaceNode {
    if (sourceLeafId === targetLeafId) {
        return root;
    }
    if (zone === 'center') {
        return root;
    }
    const extraction = detachLeaf(root, sourceLeafId);
    if (!extraction) {
        return root;
    }
    const { root: rootWithoutSource, leaf } = extraction;
    const targetExists = findNode(rootWithoutSource, targetLeafId);
    if (!targetExists || targetExists.node.kind !== 'leaf') {
        return root;
    }
    const dir: SplitDirection = zone === 'left' || zone === 'right' ? 'h' : 'v';
    const place = zone === 'left' || zone === 'top' ? 'before' : 'after';
    return splitLeaf(rootWithoutSource, targetLeafId, dir, leaf, place);
}

export function replaceNode(root: WorkspaceNode, targetId: string, replacement: WorkspaceNode): WorkspaceNode {
    if (root.id === targetId) {
        return replacement;
    }
    if (root.kind !== 'split') {
        return root;
    }
    if (root.first.id === targetId) {
        root.first = replacement;
        return normalizeTree(root)!;
    }
    if (root.second.id === targetId) {
        root.second = replacement;
        return normalizeTree(root)!;
    }
    root.first = replaceNode(root.first, targetId, replacement);
    root.second = replaceNode(root.second, targetId, replacement);
    return normalizeTree(root)!;
}

export function detachLeaf(root: WorkspaceNode, targetLeafId: string): { root: WorkspaceNode; leaf: LeafPanelSnapshot } | null {
    const location = findNode(root, targetLeafId);
    if (!location || location.node.kind !== 'leaf' || !location.parent || !location.key) {
        return null;
    }
    const sibling = location.key === 'first' ? location.parent.second : location.parent.first;
    const nextRoot = replaceNode(root, location.parent.id, sibling);
    return { root: normalizeTree(nextRoot)!, leaf: location.node };
}

export function createDefaultWorkspace(): WorkspaceSnapshot {
    const explorer = createLeaf('explorer');
    const editor = createLeaf('editor');
    const terminal = createLeaf('terminal');
    return {
        root: createSplit('h', explorer, createSplit('v', editor, terminal, 0.68), 0.24),
        focusedLeafId: editor.id,
    };
}

export function cloneWorkspaceSnapshot(snapshot: WorkspaceSnapshot): WorkspaceSnapshot {
    return JSON.parse(JSON.stringify(snapshot));
}
