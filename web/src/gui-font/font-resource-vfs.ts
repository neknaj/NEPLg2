import type { VFS } from '../runtime/vfs.js';

export type GuiFontResourcePayloadKind = 'binary' | 'text';
export type GuiFontResourcePath = string & { readonly __guiFontResourcePath: unique symbol };
export type GuiFontResourcePathErrorReason =
    | 'Empty'
    | 'Absolute'
    | 'Backslash'
    | 'EmptySegment'
    | 'DotSegment'
    | 'ParentSegment'
    | 'VfsPathMismatch';

export type GuiBundledFontResource = {
    resourcePath: string;
    vfsPath: string;
    sourceUrl: string;
    payloadKind: GuiFontResourcePayloadKind;
};

export type GuiFontResourceMountError =
    | {
        kind: 'FetchUnavailable';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
    }
    | {
        kind: 'InvalidResourcePath';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
        reason: GuiFontResourcePathErrorReason;
    }
    | {
        kind: 'NetworkError';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
        message: string;
    }
    | {
        kind: 'HttpError';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
        status: number;
    }
    | {
        kind: 'InvalidBytes';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
        message: string;
    }
    | {
        kind: 'InvalidText';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
        message: string;
    }
    | {
        kind: 'VfsWriteFailed';
        resourcePath: string;
        vfsPath: string;
        sourceUrl: string;
        message: string;
    };

export type GuiFontResourceMountResult =
    | {
        ok: true;
        mountedPaths: string[];
    }
    | {
        ok: false;
        error: GuiFontResourceMountError;
    };

export type GuiFontResourceFetch = (url: string) => Promise<Response>;

export type GuiFontResourcePathResult =
    | {
        ok: true;
        path: GuiFontResourcePath;
    }
    | {
        ok: false;
        reason: GuiFontResourcePathErrorReason;
    };

type MountedPayload = {
    resourcePath: GuiFontResourcePath;
    vfsPath: string;
    sourceUrl: string;
    content: string | Uint8Array;
};

type GuiFontResourceLoadResult =
    | {
        ok: true;
        payload: MountedPayload;
    }
    | {
        ok: false;
        error: GuiFontResourceMountError;
    };

export const GUI_FONT_RESOURCE_ROOT = 'fonts';
export const HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH = `${GUI_FONT_RESOURCE_ROOT}/HackGenConsoleNF-Regular.ttf`;
const HACKGEN_LICENSE_RESOURCE_PATH = `${GUI_FONT_RESOURCE_ROOT}/HackGen-LICENSE.txt`;

export const BUNDLED_GUI_FONT_RESOURCES: readonly GuiBundledFontResource[] = Object.freeze([
    Object.freeze({
        resourcePath: HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH,
        vfsPath: `/${HACKGEN_CONSOLE_REGULAR_RESOURCE_PATH}`,
        sourceUrl: './src/fonts/HackGenConsoleNF-Regular.ttf',
        payloadKind: 'binary' as const,
    }),
    Object.freeze({
        resourcePath: HACKGEN_LICENSE_RESOURCE_PATH,
        vfsPath: `/${HACKGEN_LICENSE_RESOURCE_PATH}`,
        sourceUrl: './src/fonts/HackGen-LICENSE.txt',
        payloadKind: 'text' as const,
    }),
]);

export function bundledGuiFontResourcePaths(): readonly string[] {
    return BUNDLED_GUI_FONT_RESOURCES.map((resource) => resource.resourcePath);
}

export function normalizeGuiFontResourcePath(rawPath: string): GuiFontResourcePathResult {
    if (rawPath.length === 0) {
        return { ok: false, reason: 'Empty' };
    }
    if (rawPath.startsWith('/')) {
        return { ok: false, reason: 'Absolute' };
    }
    if (rawPath.includes('\\')) {
        return { ok: false, reason: 'Backslash' };
    }
    const segments = rawPath.split('/');
    if (segments.some((segment) => segment.length === 0)) {
        return { ok: false, reason: 'EmptySegment' };
    }
    if (segments.some((segment) => segment === '.')) {
        return { ok: false, reason: 'DotSegment' };
    }
    if (segments.some((segment) => segment === '..')) {
        return { ok: false, reason: 'ParentSegment' };
    }
    return { ok: true, path: rawPath as GuiFontResourcePath };
}

export function guiFontResourceVfsPath(path: GuiFontResourcePath): string {
    return `/${path}`;
}

export async function mountBundledGuiFontResources(
    vfs: VFS,
    options: { fetch?: GuiFontResourceFetch; resources?: readonly GuiBundledFontResource[] } = {},
): Promise<GuiFontResourceMountResult> {
    const resources = options.resources ?? BUNDLED_GUI_FONT_RESOURCES;
    const fetchResource =
        options.fetch ?? (typeof globalThis.fetch === 'function' ? globalThis.fetch.bind(globalThis) : null);
    if (!fetchResource) {
        const first = resources[0];
        if (!first) {
            return {
                ok: true,
                mountedPaths: [],
            };
        }
        return {
            ok: false,
            error: {
                kind: 'FetchUnavailable',
                resourcePath: first.resourcePath,
                vfsPath: first.vfsPath,
                sourceUrl: first.sourceUrl,
            },
        };
    }

    const staged: MountedPayload[] = [];
    for (const resource of resources) {
        const pathResult = normalizeGuiFontResourcePath(resource.resourcePath);
        if (!pathResult.ok) {
            return invalidResourcePath(resource, pathResult.reason);
        }
        const canonicalVfsPath = guiFontResourceVfsPath(pathResult.path);
        if (resource.vfsPath !== canonicalVfsPath) {
            return invalidResourcePath(resource, 'VfsPathMismatch');
        }
        const result = await loadBundledGuiFontResource(resource, fetchResource);
        if (!result.ok) {
            return result;
        }
        staged.push(result.payload);
    }

    const mountedPaths: string[] = [];
    for (const payload of staged) {
        try {
            vfs.writeFile(payload.vfsPath, payload.content, { force: true });
            vfs.setReadOnly(payload.vfsPath, true);
            mountedPaths.push(payload.vfsPath);
        } catch (error) {
            rollbackMountedFontResource(vfs, payload.vfsPath);
            for (const mountedPath of mountedPaths) {
                rollbackMountedFontResource(vfs, mountedPath);
            }
            return {
                ok: false,
                error: {
                    kind: 'VfsWriteFailed',
                    resourcePath: payload.resourcePath,
                    vfsPath: payload.vfsPath,
                    sourceUrl: payload.sourceUrl,
                    message: error instanceof Error ? error.message : String(error),
                },
            };
        }
    }

    return {
        ok: true,
        mountedPaths,
    };
}

async function loadBundledGuiFontResource(
    resource: GuiBundledFontResource,
    fetchResource: GuiFontResourceFetch,
): Promise<GuiFontResourceLoadResult> {
    let response: Response;
    try {
        response = await fetchResource(resource.sourceUrl);
    } catch (error) {
        return {
            ok: false,
            error: {
                kind: 'NetworkError',
                resourcePath: resource.resourcePath,
                vfsPath: resource.vfsPath,
                sourceUrl: resource.sourceUrl,
                message: error instanceof Error ? error.message : String(error),
            },
        };
    }

    if (!response.ok) {
        return {
            ok: false,
            error: {
                kind: 'HttpError',
                resourcePath: resource.resourcePath,
                vfsPath: resource.vfsPath,
                sourceUrl: resource.sourceUrl,
                status: response.status,
            },
        };
    }

    if (resource.payloadKind === 'binary') {
        let bytes: Uint8Array;
        try {
            bytes = new Uint8Array(await response.arrayBuffer());
        } catch (error) {
            return {
                ok: false,
                error: {
                    kind: 'InvalidBytes',
                    resourcePath: resource.resourcePath,
                    vfsPath: resource.vfsPath,
                    sourceUrl: resource.sourceUrl,
                    message: error instanceof Error ? error.message : String(error),
                },
            };
        }
        if (bytes.byteLength === 0) {
            return {
                ok: false,
                error: {
                    kind: 'InvalidBytes',
                    resourcePath: resource.resourcePath,
                    vfsPath: resource.vfsPath,
                    sourceUrl: resource.sourceUrl,
                    message: 'empty binary resource',
                },
            };
        }
        return {
            ok: true,
            payload: {
                resourcePath: resource.resourcePath as GuiFontResourcePath,
                vfsPath: resource.vfsPath,
                sourceUrl: resource.sourceUrl,
                content: bytes,
            },
        };
    }

    let text: string;
    try {
        text = await response.text();
    } catch (error) {
        return {
            ok: false,
            error: {
                kind: 'InvalidText',
                resourcePath: resource.resourcePath,
                vfsPath: resource.vfsPath,
                sourceUrl: resource.sourceUrl,
                message: error instanceof Error ? error.message : String(error),
            },
        };
    }
    if (text.length === 0) {
        return {
            ok: false,
            error: {
                kind: 'InvalidText',
                resourcePath: resource.resourcePath,
                vfsPath: resource.vfsPath,
                sourceUrl: resource.sourceUrl,
                message: 'empty text resource',
            },
        };
    }
    return {
        ok: true,
        payload: {
            resourcePath: resource.resourcePath as GuiFontResourcePath,
            vfsPath: resource.vfsPath,
            sourceUrl: resource.sourceUrl,
            content: text,
        },
    };
}

function invalidResourcePath(
    resource: GuiBundledFontResource,
    reason: GuiFontResourcePathErrorReason,
): GuiFontResourceMountResult {
    return {
        ok: false,
        error: {
            kind: 'InvalidResourcePath',
            resourcePath: resource.resourcePath,
            vfsPath: resource.vfsPath,
            sourceUrl: resource.sourceUrl,
            reason,
        },
    };
}

function rollbackMountedFontResource(vfs: VFS, path: string) {
    try {
        vfs.setReadOnly(path, false);
    } catch {
    }
    vfs.deleteFile(path);
}
