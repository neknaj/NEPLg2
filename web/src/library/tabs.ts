export interface Tab {
    path: string;
    content: string;
    isPermanent: boolean;
}

export class TabManager {
    tabs: Tab[] = [];
    activeTabIndex: number = -1;
    container: HTMLElement;
    editor: any;
    vfs: any;

    constructor(container: HTMLElement, editor: any, vfs: any) {
        this.container = container;
        this.editor = editor;
        this.vfs = vfs;
    }

    openFile(path: string) {
        // Find if already open
        let index = this.tabs.findIndex(t => t.path === path);
        if (index !== -1) {
            this.saveCurrentTab(); // Save whatever was in the previous tab before switching
            this.setActiveTab(index);
            return;
        }

        const newContent = this.vfs.readFile(path);
        const contentStr = typeof newContent === 'string' ? newContent : "Binary file...";

        // Logic: If current active tab is NOT PERMANENT and UNEDITED, replace it instead of creating new one
        if (this.activeTabIndex >= 0) {
            const currentTab = this.tabs[this.activeTabIndex];
            if (!currentTab.isPermanent) {
                const currentEditorText = typeof this.editor.getText === 'function' ? this.editor.getText() : this.editor.text;

                // If content in editor is exactly what's in the tab record (meaning no edits since load/save)
                if (currentEditorText === currentTab.content) {
                    currentTab.path = path;
                    currentTab.content = contentStr;
                    currentTab.isPermanent = false; // Still provisional
                    this.setActiveTab(this.activeTabIndex);
                    return;
                }
            }
        }

        // Save current tab before opening new one
        this.saveCurrentTab();

        this.tabs.push({ path, content: contentStr, isPermanent: false });
        this.setActiveTab(this.tabs.length - 1);
    }

    saveCurrentTab() {
        if (this.activeTabIndex >= 0) {
            const currentTab = this.tabs[this.activeTabIndex];
            const text = typeof this.editor.getText === 'function' ? this.editor.getText() : this.editor.text;

            if (text !== currentTab.content) {
                currentTab.content = text;
                currentTab.isPermanent = true; // Mark as permanent once edited
                this.vfs.writeFile(currentTab.path, currentTab.content);
            }
        }
    }

    setActiveTab(index: number) {
        this.activeTabIndex = index;
        const tab = this.tabs[index];
        this.editor.setText(tab.content);
        // Explicitly set the path on the editor if possible
        if (this.editor) {
            (this.editor as any).path = tab.path;
        }
        this.render();
    }

    closeTab(index: number, e?: Event) {
        if (e) e.stopPropagation();
        this.tabs.splice(index, 1);
        if (this.activeTabIndex === index) {
            this.activeTabIndex = this.tabs.length > 0 ? 0 : -1;
            if (this.activeTabIndex >= 0) {
                this.setActiveTab(this.activeTabIndex);
            } else {
                this.editor.setText("");
                if (this.editor) (this.editor as any).path = null;
            }
        } else if (this.activeTabIndex > index) {
            this.activeTabIndex--;
        }
        this.render();
    }

    render() {
        this.container.innerHTML = "";
        this.tabs.forEach((tab, i) => {
            const el = document.createElement('div');
            el.className = `tab ${i === this.activeTabIndex ? 'active' : ''} ${!tab.isPermanent ? 'provisional' : ''}`;

            const title = document.createElement('span');
            title.className = 'tab-title';
            title.textContent = tab.path.split('/').pop() || tab.path;

            const close = document.createElement('span');
            close.className = 'tab-close';
            close.textContent = '×';
            close.onclick = (e) => this.closeTab(i, e);

            el.appendChild(title);
            el.appendChild(close);
            el.onclick = () => this.setActiveTab(i);

            this.container.appendChild(el);
        });
    }

    get activeTab(): Tab | null {
        return this.activeTabIndex >= 0 ? this.tabs[this.activeTabIndex] : null;
    }
}
