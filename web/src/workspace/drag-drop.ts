import { PanelKind } from './panel-layout.js';

export type PanelDragPayload = {
    kind: 'panel';
    leafId: string;
};

export type TabDragPayload = {
    kind: 'editor-tab';
    leafId: string;
    path: string;
};

export type ExplorerFileDragPayload = {
    kind: 'explorer-file';
    path: string;
};

export type WorkspaceDragPayload = PanelDragPayload | TabDragPayload | ExplorerFileDragPayload;

export type TabbarDropAction = 'attach-tab' | 'open-file' | 'merge-panel' | null;

export function resolveTabbarDropAction(
    payload: WorkspaceDragPayload | null,
    targetPanelKind: PanelKind,
): TabbarDropAction {
    if (!payload || targetPanelKind !== 'editor') {
        return null;
    }
    if (payload.kind === 'editor-tab') {
        return 'attach-tab';
    }
    if (payload.kind === 'explorer-file') {
        return 'open-file';
    }
    if (payload.kind === 'panel') {
        return 'merge-panel';
    }
    return null;
}
