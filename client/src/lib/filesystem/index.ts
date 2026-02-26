export { LocalFileSystem } from "./local";
export type { FileStatExtended } from "./local";
export type { FileEntry, DefaultDirs, FileChangeEvent, FileSystemProvider } from "./types";
export {
  getThumbnail,
  invalidateThumbnail,
  clearThumbnailCache,
  isImageFile,
  IMAGE_EXTENSIONS,
} from "./thumbnail-cache";
export {
  resolvePath,
  getParentDir,
  parentDir,
  joinPath,
  isAbsolute,
  normalizeSeparators,
  getExtension,
  getFileName,
} from "./paths";

import { LocalFileSystem } from "./local";

export const localFs = new LocalFileSystem();
