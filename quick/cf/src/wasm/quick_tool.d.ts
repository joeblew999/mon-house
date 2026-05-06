/* tslint:disable */
/* eslint-disable */

/**
 * Returns the engine version string. Useful for cache-busting and sanity
 * checks ("am I talking to the build I just deployed?").
 */
export function engine_version(): string;

/**
 * Expand `<!-- include: ... -->` directives in `content`.
 *
 * Reads partial bodies on-demand through the JS-supplied `vfs` handle.
 * `base_dir` is the directory the directives resolve relative to (typically
 * the parent of the file `content` came from).
 */
export function expand_includes(vfs: any, base_dir: string, content: string): Promise<string>;

/**
 * Find which top-level specs include the given partial.
 *
 * Reads files on-demand through the JS-supplied `vfs` handle (no preload).
 * `specs_dir` is the project's specs root (e.g. `"specs"`); `partial_path`
 * is the partial's relative path (e.g. `"specs/_partials/paint-metal.md"`).
 *
 * JS API:
 * ```js
 * import init, { find_dependents } from "./pkg/quick_tool.js";
 * await init();
 * const deps = await find_dependents(vfs, "specs", "specs/_partials/paint-metal.md");
 * // → ["specs/GATE-01.md", "specs/PAINT.md", "specs/ROOF.md"]
 * ```
 */
export function find_dependents(vfs: any, specs_dir: string, partial_path: string): Promise<any>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly engine_version: () => [number, number];
    readonly expand_includes: (a: any, b: number, c: number, d: number, e: number) => any;
    readonly find_dependents: (a: any, b: number, c: number, d: number, e: number) => any;
    readonly wasm_bindgen__convert__closures_____invoke__h131bb1ef87fc9d35: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h3192b81065fcd87a: (a: number, b: number, c: any, d: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
