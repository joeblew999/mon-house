// JS-side filesystem adapters — both bridge the Rust BrowserVfs trait
// (cli/src/vfs.rs) which expects four async methods: readToString, write,
// exists, readDir.
//
// Two implementations because browser support diverges:
//
//   WasmVfs        — File System Access API (Chromium browsers).
//                    Read + write. User picks dir via showDirectoryPicker().
//
//   WasmVfsRO      — webkitdirectory <input>. Works in Safari + Firefox.
//                    Read-only. write() throws. User picks dir via the
//                    file input element.
//
// All paths are project-relative (e.g. "specs/PAINT.md").

export class WasmVfs {
  constructor(private root: FileSystemDirectoryHandle) {}

  async readToString(path: string): Promise<string> {
    const file = await this.resolveFile(path, /* createIfMissing */ false);
    if (!file) throw new Error(`readToString: ${path} not found`);
    const blob = await file.getFile();
    return blob.text();
  }

  async write(path: string, data: Uint8Array): Promise<void> {
    const file = await this.resolveFile(path, /* createIfMissing */ true);
    if (!file) throw new Error(`write: cannot create ${path}`);
    const writable = await file.createWritable();
    await writable.write(data);
    await writable.close();
  }

  async exists(path: string): Promise<boolean> {
    return (await this.resolveFile(path, /* createIfMissing */ false)) !== null;
  }

  /**
   * Returns relative paths (project-rooted, like "specs/PAINT.md") of every
   * file directly inside `path`. Does NOT recurse — matches the Rust
   * vfs::read_dir contract.
   */
  async readDir(path: string): Promise<string[]> {
    const dir = await this.resolveDir(path);
    if (!dir) return [];
    const out: string[] = [];
    const prefix = path && !path.endsWith("/") ? path + "/" : path;
    // @ts-expect-error — TS lib.dom doesn't yet have entries() on FSDirectoryHandle
    for await (const [name, handle] of dir.entries()) {
      if (handle.kind === "file") {
        out.push(prefix + name);
      }
    }
    return out;
  }

  // ── helpers ──────────────────────────────────────────────────────────────

  /**
   * Walk the directory handle tree along `path` (slash-separated). Returns
   * the directory at the leaf, or null if any segment is missing / not a
   * directory.
   */
  private async resolveDir(
    path: string,
  ): Promise<FileSystemDirectoryHandle | null> {
    if (!path || path === ".") return this.root;
    let cur: FileSystemDirectoryHandle = this.root;
    for (const seg of path.split("/").filter((s) => s && s !== ".")) {
      try {
        cur = await cur.getDirectoryHandle(seg);
      } catch {
        return null;
      }
    }
    return cur;
  }

  /**
   * Resolve `path` to a file handle. The parent directory must exist; the
   * file itself may or may not, depending on `createIfMissing`.
   */
  private async resolveFile(
    path: string,
    createIfMissing: boolean,
  ): Promise<FileSystemFileHandle | null> {
    const lastSlash = path.lastIndexOf("/");
    const parentPath = lastSlash >= 0 ? path.slice(0, lastSlash) : "";
    const filename = lastSlash >= 0 ? path.slice(lastSlash + 1) : path;
    const parent = await this.resolveDir(parentPath);
    if (!parent) return null;
    try {
      return await parent.getFileHandle(filename, { create: createIfMissing });
    } catch {
      return null;
    }
  }
}

/**
 * Read-only Vfs backed by a FileList (from `<input type="file" webkitdirectory>`).
 *
 * Works in Safari + Firefox where FS Access API is unavailable. File contents
 * are NOT preloaded — each File reference is lazy; `.text()` reads on demand.
 * Write throws because the input element has no write capability.
 *
 * Path normalisation: webkitRelativePath includes the picked-folder name as
 * the first segment (e.g. "quick/specs/PAINT.md" if the user picked `quick/`).
 * We strip that prefix so paths align with what the Rust engine expects
 * (e.g. "specs/PAINT.md" from project root).
 */
export class WasmVfsRO {
  private files: Map<string, File> = new Map();

  constructor(fileList: FileList) {
    // Determine the prefix to strip — the picked folder's name. All files
    // share the same first segment in webkitRelativePath.
    const first = fileList.length > 0 ? fileList[0].webkitRelativePath : "";
    const prefix = first.includes("/") ? first.split("/")[0] + "/" : "";
    for (let i = 0; i < fileList.length; i++) {
      const f = fileList[i];
      const rel = f.webkitRelativePath.startsWith(prefix)
        ? f.webkitRelativePath.slice(prefix.length)
        : f.webkitRelativePath;
      this.files.set(rel, f);
    }
  }

  async readToString(path: string): Promise<string> {
    const f = this.files.get(path);
    if (!f) throw new Error(`readToString: ${path} not found`);
    return f.text();
  }

  async write(_path: string, _data: Uint8Array): Promise<void> {
    throw new Error(
      "write() not supported on Safari/Firefox — File System Access API needed for writes",
    );
  }

  async exists(path: string): Promise<boolean> {
    return this.files.has(path);
  }

  async readDir(path: string): Promise<string[]> {
    const prefix = path && !path.endsWith("/") ? path + "/" : path;
    const out: string[] = [];
    for (const key of this.files.keys()) {
      if (!key.startsWith(prefix)) continue;
      const rest = key.slice(prefix.length);
      if (!rest.includes("/")) {
        out.push(key);
      }
    }
    return out;
  }
}
