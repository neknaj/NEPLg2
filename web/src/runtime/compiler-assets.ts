export type CompilerAssetUrls = {
    moduleUrl: string;
    wasmUrl: string;
};

type LinkLike = {
    getAttribute(name: string): string | null;
    href?: string;
};

type DocumentLike = {
    querySelector?(selector: string): LinkLike | null;
};

type WindowLike = {
    NEPLg2CompilerAssets?: CompilerAssetUrls | null;
};

function readLinkUrl(link: LinkLike | null): string | null {
    if (!link) {
        return null;
    }
    const href = link.getAttribute('href') || link.href || null;
    return href ? String(href) : null;
}

export function readCompilerAssetsFromDocument(doc: DocumentLike | null | undefined): CompilerAssetUrls | null {
    if (!doc?.querySelector) {
        return null;
    }

    const moduleUrl = readLinkUrl(doc.querySelector(`link[rel="modulepreload"][href*="nepl-web-"][href$=".js"]`));
    const wasmUrl = readLinkUrl(doc.querySelector(`link[rel="preload"][href*="nepl-web-"][href$="_bg.wasm"]`));

    if (!moduleUrl || !wasmUrl) {
        return null;
    }

    return {
        moduleUrl,
        wasmUrl,
    };
}

export function resolveCompilerAssets(win?: WindowLike | null, doc?: DocumentLike | null): CompilerAssetUrls | null {
    const fromWindow = win?.NEPLg2CompilerAssets;
    if (fromWindow?.moduleUrl && fromWindow?.wasmUrl) {
        return {
            moduleUrl: String(fromWindow.moduleUrl),
            wasmUrl: String(fromWindow.wasmUrl),
        };
    }
    return readCompilerAssetsFromDocument(doc);
}
