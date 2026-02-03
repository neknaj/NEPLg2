export class EditorDOMUI {
    /**
     * @param {CanvasEditor} editor
     * @param {object} elements
     */
    constructor(editor, elements) {
        this.editor = editor;
        this.popup = elements.popup; // Can be null if not used
        this.problemsPanel = elements.problemsPanel; // Can be null
        this.completionList = elements.completionList;

        this.isCompletionVisible = false;
        this.completionSuggestions = [];
        this.selectedSuggestionIndex = 0;
    }

    showPopup(content, x, y) {
        if (!this.popup) return;
        this.popup.classList.remove('hidden');
        this.popup.textContent = content;
        this.popup.style.left = `${x + 10}px`;
        this.popup.style.top = `${y + 10}px`;
    }

    hidePopup() {
        if (!this.popup) return;
        this.popup.classList.add('hidden');
    }

    updateProblemsPanel() {
        if (!this.problemsPanel) return;
        this.problemsPanel.innerHTML = '';
        this.editor.diagnostics.forEach(diag => {
            const { row, col } = this.editor.utils.getPosFromIndex(diag.startIndex, this.editor.lines);
            const li = document.createElement('li');
            li.className = `severity-${diag.severity}`;
            li.textContent = diag.message;
            // ... (rest of logic similar to original if needed)
            this.problemsPanel.appendChild(li);
        });
    }

    showCompletion(suggestions) {
        if (!this.completionList) return;
        this.completionSuggestions = suggestions;
        this.selectedSuggestionIndex = 0;
        this.isCompletionVisible = true;
        this.completionList.innerHTML = '';

        suggestions.forEach((item, index) => {
            const li = document.createElement('li');
            li.dataset.index = String(index);

            // Custom styling structure
            const leftDiv = document.createElement('div');
            leftDiv.className = 'completion-item-left';

            const typeSpan = document.createElement('span');
            typeSpan.className = 'completion-type';
            typeSpan.textContent = this.getCompletionTypeAbbreviation(item.type);
            leftDiv.appendChild(typeSpan);

            const labelSpan = document.createElement('span');
            labelSpan.textContent = item.label;
            leftDiv.appendChild(labelSpan);

            li.appendChild(leftDiv);

            if (item.detail) {
                const detailSpan = document.createElement('span');
                detailSpan.className = 'completion-detail';
                detailSpan.textContent = item.detail;
                li.appendChild(detailSpan);
            }

            if (index === this.selectedSuggestionIndex) {
                li.classList.add('selected');
            }

            li.addEventListener('mousedown', (e) => {
                e.preventDefault();
                this.selectedSuggestionIndex = index;
                this.editor.acceptCompletion();
            });
            this.completionList.appendChild(li);
        });

        this.completionList.classList.remove('hidden');
        this.editor.updateTextareaPosition();
    }

    hideCompletion() {
        if (!this.isCompletionVisible) return;
        this.isCompletionVisible = false;
        this.completionSuggestions = [];
        if (this.completionList) this.completionList.classList.add('hidden');
    }

    updateCompletionSelection(direction) {
        if (!this.isCompletionVisible || !this.completionList) return;
        const newIndex = this.selectedSuggestionIndex + direction;
        const total = this.completionSuggestions.length;
        if (newIndex < 0 || newIndex >= total) return;

        const items = this.completionList.children;
        items[this.selectedSuggestionIndex].classList.remove('selected');
        this.selectedSuggestionIndex = newIndex;
        items[this.selectedSuggestionIndex].classList.add('selected');
        items[this.selectedSuggestionIndex].scrollIntoView({ block: 'nearest' });
    }

    getCompletionTypeAbbreviation(type) {
        switch (type) {
            case 'keyword': return 'K';
            case 'snippet': return 'S';
            case 'function': return 'f';
            case 'struct': return 'T';
            case 'variable': return 'v';
            default: return '·';
        }
    }
}
