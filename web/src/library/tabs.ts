export interface Tab {
    path: string;
    content: string;
    isPermanent: boolean;
    isEditable: boolean;
    zoom: number;
}

export interface TabSnapshot {
    path: string;
}

export class TabManager {
    tabs: Tab[] = [];
    activeTabIndex = -1;
    container: HTMLElement;
    editor: any;
    vfs: any;
    onStateChange: (() => void) | null;
    onActiveTabChange: ((tab: Tab | null) => void) | null;

    constructor(
        container: HTMLElement,
        editor: any,
        vfs: any,
        options: {
            onStateChange?: (() => void) | null;
            onActiveTabChange?: ((tab: Tab | null) => void) | null;
        } = {},
    ) {
        this.container = container;
        this.editor = editor;
        this.vfs = vfs;
        this.onStateChange = options.onStateChange || null;
        this.onActiveTabChange = options.onActiveTabChange || null;
    }

    normalizeText(text: string): string {
        return String(text ?? '').replace(/\r\n?/g, '\n');
    }

    notifyStateChange() {
        if (this.onStateChange) {
            this.onStateChange();
        }
    }

    getTabSnapshot(): { paths: string[]; activePath: string | null; pathZooms: Record<string, number> } {
        return {
            paths: this.tabs.map((tab) => tab.path),
            activePath: this.activeTab?.path || null,
            pathZooms: Object.fromEntries(this.tabs.map((tab) => [tab.path, tab.zoom])),
        };
    }

    restoreTabs(paths: string[], activePath: string | null = null, pathZooms: Record<string, number> = {}) {
        this.tabs = [];
        this.activeTabIndex = -1;
        for (const path of paths) {
            if (!this.vfs.exists(path)) {
                continue;
            }
            const newContent = this.vfs.readFile(path);
            const contentStr = typeof newContent === 'string' ? this.normalizeText(newContent) : 'Binary file...';
            this.tabs.push({
                path,
                content: contentStr,
                isPermanent: true,
                isEditable: this.vfs.isEditable(path),
                zoom: Number.isFinite(pathZooms[path]) ? Number(pathZooms[path]) : 1,
            });
        }
        if (this.tabs.length === 0) {
            this.editor.setText('');
            if (typeof this.editor.setEditable === 'function') {
                this.editor.setEditable(false);
            }
            if (typeof this.editor.setPath === 'function') {
                this.editor.setPath(null);
            }
            this.render();
            this.notifyStateChange();
            if (this.onActiveTabChange) {
                this.onActiveTabChange(null);
            }
            return;
        }
        const index = activePath ? this.tabs.findIndex((tab) => tab.path === activePath) : 0;
        this.setActiveTab(index >= 0 ? index : 0, { focusEditor: false, persistCurrent: false });
    }

    openFile(path: string) {
        const index = this.tabs.findIndex((tab) => tab.path === path);
        if (index !== -1) {
            this.saveCurrentTab();
            this.setActiveTab(index);
            return;
        }

        const newContent = this.vfs.readFile(path);
        const contentStr = typeof newContent === 'string' ? this.normalizeText(newContent) : 'Binary file...';
        const isEditable = this.vfs.isEditable(path);

        if (this.activeTabIndex >= 0) {
            const currentTab = this.tabs[this.activeTabIndex];
            if (!currentTab.isPermanent) {
                const currentEditorText = this.normalizeText(typeof this.editor.getText === 'function' ? this.editor.getText() : this.editor.text);
                if (currentEditorText === currentTab.content) {
                    currentTab.path = path;
                    currentTab.content = contentStr;
                    currentTab.isPermanent = false;
                    currentTab.isEditable = isEditable;
                    currentTab.zoom = 1;
                    this.setActiveTab(this.activeTabIndex, { focusEditor: true, persistCurrent: false });
                    return;
                }
            }
        }

        this.saveCurrentTab();
        this.tabs.push({ path, content: contentStr, isPermanent: false, isEditable, zoom: 1 });
        this.setActiveTab(this.tabs.length - 1);
    }

    saveCurrentTab() {
        if (this.activeTabIndex < 0) {
            return;
        }
        const currentTab = this.tabs[this.activeTabIndex];
        if (!currentTab || !currentTab.isEditable) {
            return;
        }
        const text = this.normalizeText(typeof this.editor.getText === 'function' ? this.editor.getText() : this.editor.text);
        if (text !== currentTab.content) {
            currentTab.content = text;
            currentTab.isPermanent = true;
            this.vfs.writeFile(currentTab.path, currentTab.content);
            this.notifyStateChange();
        }
    }

    setActiveTab(index: number, options: { focusEditor?: boolean; persistCurrent?: boolean } = {}) {
        if (index < 0 || index >= this.tabs.length) {
            return;
        }
        const shouldPersistCurrent = options.persistCurrent !== false;
        if (shouldPersistCurrent && this.activeTabIndex >= 0 && this.activeTabIndex !== index) {
            this.saveCurrentTab();
        }
        this.activeTabIndex = index;
        const tab = this.tabs[index];
        this.editor.setText(tab.content);
        if (typeof this.editor.setEditable === 'function') {
            this.editor.setEditable(tab.isEditable);
        }
        if (typeof this.editor.setPath === 'function') {
            this.editor.setPath(tab.path);
        } else {
            this.editor.path = tab.path;
        }
        this.render();
        this.notifyStateChange();
        if (this.onActiveTabChange) {
            this.onActiveTabChange(tab);
        }
        if (options.focusEditor !== false && typeof this.editor.focus === 'function') {
            this.editor.focus();
        }
    }

    closeTab(index: number, e?: Event) {
        if (e) {
            e.stopPropagation();
        }
        if (index < 0 || index >= this.tabs.length) {
            return;
        }
        this.tabs.splice(index, 1);
        if (this.activeTabIndex === index) {
            this.activeTabIndex = this.tabs.length > 0 ? Math.max(0, index - 1) : -1;
            if (this.activeTabIndex >= 0) {
                this.setActiveTab(this.activeTabIndex, { focusEditor: false, persistCurrent: false });
            } else {
                this.editor.setText('');
                if (typeof this.editor.setEditable === 'function') {
                    this.editor.setEditable(false);
                }
                if (typeof this.editor.setPath === 'function') {
                    this.editor.setPath(null);
                }
                if (this.onActiveTabChange) {
                    this.onActiveTabChange(null);
                }
            }
        } else if (this.activeTabIndex > index) {
            this.activeTabIndex -= 1;
        }
        this.render();
        this.notifyStateChange();
    }

    mergeFrom(other: TabManager) {
        const sourcePaths = other.tabs.map((tab) => tab.path);
        const targetPaths = new Set(this.tabs.map((tab) => tab.path));
        for (const path of sourcePaths) {
            if (!targetPaths.has(path) && this.vfs.exists(path)) {
                const content = this.vfs.readFile(path);
                this.tabs.push({
                    path,
                    content: typeof content === 'string' ? this.normalizeText(content) : 'Binary file...',
                    isPermanent: true,
                    isEditable: this.vfs.isEditable(path),
                    zoom: 1,
                });
            }
        }
        if (this.activeTabIndex < 0 && this.tabs.length > 0) {
            this.activeTabIndex = 0;
        }
        this.render();
        this.notifyStateChange();
    }

    render() {
        this.container.innerHTML = '';
        this.tabs.forEach((tab, index) => {
            const el = document.createElement('div');
            el.className = `tab ${index === this.activeTabIndex ? 'active' : ''} ${!tab.isPermanent ? 'provisional' : ''} ${!tab.isEditable ? 'readonly' : ''}`;

            const title = document.createElement('span');
            title.className = 'tab-title';
            title.textContent = `${!tab.isEditable ? '[ro] ' : ''}${tab.path.split('/').pop() || tab.path}`;

            const close = document.createElement('span');
            close.className = 'tab-close';
            close.textContent = 'x';
            close.onclick = (event) => this.closeTab(index, event);

            el.appendChild(title);
            el.appendChild(close);
            el.onclick = () => this.setActiveTab(index);
            this.container.appendChild(el);
        });
    }

    get activeTab(): Tab | null {
        return this.activeTabIndex >= 0 ? this.tabs[this.activeTabIndex] : null;
    }

    getActiveZoom(): number {
        return this.activeTab?.zoom ?? 1;
    }

    setActiveZoom(zoom: number) {
        if (!this.activeTab) {
            return;
        }
        this.activeTab.zoom = zoom;
        this.notifyStateChange();
        if (this.onActiveTabChange) {
            this.onActiveTabChange(this.activeTab);
        }
    }
}
