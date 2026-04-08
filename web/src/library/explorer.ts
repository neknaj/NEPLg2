export class FileExplorer {
    container: HTMLElement;
    vfs: any;
    onFileClick: (path: string) => void;
    onFileDragStart: ((path: string, event: DragEvent) => void) | null;
    expandedFolders: Set<string> = new Set(['/', '/examples', '/stdlib']);

    constructor(
        container: HTMLElement,
        vfs: any,
        onFileClick: (path: string) => void,
        options: { onFileDragStart?: ((path: string, event: DragEvent) => void) | null } = {},
    ) {
        this.container = container;
        this.vfs = vfs;
        this.onFileClick = onFileClick;
        this.onFileDragStart = options.onFileDragStart || null;
    }

    refresh() {
        this.render();
    }

    render() {
        this.container.innerHTML = "";
        const rootItems = this.vfs.listDir('/');
        rootItems.forEach((name: string) => {
            this.renderItem('/', name, this.container);
        });
    }

    renderItem(parentPath: string, name: string, parentEl: HTMLElement) {
        const fullPath = (parentPath === '/' ? '/' : parentPath + '/') + name;
        const isDir = this.vfs.isDir(fullPath);
        const isOpen = isDir && this.expandedFolders.has(fullPath);

        const itemEl = document.createElement('div');
        itemEl.className = `explorer-item ${isDir ? 'folder' : 'file'}`;
        if (isOpen) {
            itemEl.classList.add('open');
        }
        if (!isDir) {
            itemEl.draggable = true;
            itemEl.addEventListener('dragstart', (event) => {
                itemEl.classList.add('dragging');
                if (this.onFileDragStart) {
                    this.onFileDragStart(fullPath, event);
                }
            });
            itemEl.addEventListener('dragend', () => {
                itemEl.classList.remove('dragging');
            });
        }

        const disclosureEl = document.createElement('span');
        disclosureEl.className = `explorer-disclosure ${isDir ? '' : 'empty'} ${isOpen ? 'open' : ''}`.trim();

        const iconEl = document.createElement('span');
        iconEl.className = `explorer-icon ${isDir ? 'explorer-icon-folder' : 'explorer-icon-file'} ${isOpen ? 'open' : ''}`.trim();

        const labelEl = document.createElement('span');
        labelEl.className = 'explorer-label';
        labelEl.textContent = name;

        itemEl.appendChild(disclosureEl);
        itemEl.appendChild(iconEl);
        itemEl.appendChild(labelEl);

        itemEl.onclick = (e) => {
            e.stopPropagation();
            if (isDir) {
                if (this.expandedFolders.has(fullPath)) {
                    this.expandedFolders.delete(fullPath);
                } else {
                    this.expandedFolders.add(fullPath);
                }
                this.render();
            } else {
                this.onFileClick(fullPath);
            }
        };

        parentEl.appendChild(itemEl);

        if (isDir && this.expandedFolders.has(fullPath)) {
            const childrenEl = document.createElement('div');
            childrenEl.className = 'explorer-children';
            const children = this.vfs.listDir(fullPath);
            children.forEach((childName: string) => {
                this.renderItem(fullPath, childName, childrenEl);
            });
            parentEl.appendChild(childrenEl);
        }
    }
}
