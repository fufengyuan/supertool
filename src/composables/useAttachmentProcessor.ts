/**
 * 附件处理 composable
 * 处理剪贴板粘贴的图片和文本文件
 */

// ─── Types ────────────────────────────────────────────────────────────────────

export interface Attachment {
  id: string;
  kind: 'image' | 'text-file' | 'path-ref';
  name: string;
  mime: string;
  size: number;
  dataUrl?: string;  // images: data:image/png;base64,...
  text?: string;     // text files: raw UTF-8 content
  path?: string;     // binary refs: filesystem path
}

// ─── Constants ────────────────────────────────────────────────────────────────

export const MAX_IMAGE_BYTES = 20 * 1024 * 1024; // 20MB
export const MAX_TEXT_BYTES = 256 * 1024; // 256KB

export const ALLOWED_IMAGE_MIMES = new Set<string>([
  'image/png',
  'image/jpeg',
  'image/webp',
  'image/gif',
]);

export const ALLOWED_TEXT_EXTENSIONS = new Set<string>([
  // 代码文件
  '.js', '.ts', '.tsx', '.jsx', '.mjs', '.cjs',
  '.vue', '.svelte',
  '.py', '.pyi', '.pyw',
  '.rs', '.go', '.java', '.kt', '.kts',
  '.c', '.cpp', '.cc', '.cxx', '.h', '.hpp', '.hh',
  '.cs', '.swift', '.rb', '.php', '.pl', '.lua',
  '.r', '.R',
  // Shell / 配置
  '.sh', '.bash', '.zsh', '.fish', '.ps1',
  '.yml', '.yaml', '.toml', '.ini', '.cfg', '.conf',
  '.json', '.jsonc', '.json5',
  '.xml', '.html', '.htm', '.css', '.scss', '.sass', '.less',
  // 标记 / 文档
  '.md', '.mdx', '.txt', '.csv', '.tsv', '.log',
  '.tex', '.rst',
  // 其他
  '.sql', '.graphql', '.gql',
  '.env', '.gitignore', '.dockerignore', '.editorconfig',
  '.prettierrc', '.eslintrc',
]);

// ─── Helpers ──────────────────────────────────────────────────────────────────

/**
 * 生成唯一 ID
 */
function generateId(): string {
  return `att_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

/**
 * 读取文件为 base64 data URL
 */
export function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error(`读取文件失败: ${file.name}`));
    reader.readAsDataURL(file);
  });
}

/**
 * 读取文件为 UTF-8 文本
 */
export function readAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error(`读取文件失败: ${file.name}`));
    reader.readAsText(file, 'utf-8');
  });
}

/**
 * 检查 MIME 是否为支持的图片类型
 */
export function isImageMime(mime: string): boolean {
  return ALLOWED_IMAGE_MIMES.has(mime);
}

/**
 * 检查文件是否为文本文件（通过 MIME 或扩展名）
 */
export function isTextFile(mime: string, name: string): boolean {
  // MIME 检查
  if (mime.startsWith('text/')) {return true;}
  if (mime === 'application/json') {return true;}
  if (mime === 'application/xml') {return true;}
  if (mime === 'application/javascript') {return true;}
  if (mime === 'application/typescript') {return true;}
  if (mime === 'application/x-yaml') {return true;}
  // 无扩展名/无 MIME 时的模糊扩展名匹配
  const ext = name.includes('.') ? '.' + name.split('.').pop()!.toLowerCase() : '';
  if (ext && ALLOWED_TEXT_EXTENSIONS.has(ext)) {return true;}
  return false;
}

/**
 * 从 ClipboardEvent 中提取文件列表
 */
export function filesFromClipboard(event: ClipboardEvent): File[] {
  const files: File[] = [];
  const items = event.clipboardData?.items;
  if (!items) {return files;}
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.kind === 'file') {
      const file = item.getAsFile();
      if (file) {
        files.push(file);
      }
    }
  }
  return files;
}

// ─── Core ─────────────────────────────────────────────────────────────────────

export interface ProcessFilesOptions {
  /** 远程模式：不支持 path-ref，二进制文件直接报错 */
  remoteMode?: boolean;
}

export interface ProcessFilesResult {
  attachments: Attachment[];
  errors: string[];
}

/**
 * 处理文件列表，生成 Attachment 数组
 *
 * - 图片文件（ALLOWED_IMAGE_MIMES）→ dataUrl，受 MAX_IMAGE_BYTES 限制
 * - 文本文件（isTextFile）→ text，受 MAX_TEXT_BYTES 限制
 * - 二进制文件：
 *   - remoteMode: error
 *   - localMode: path-ref（需要外部调用方提供 path，此处仅标记）
 */
export async function processFiles(
  files: File[],
  options?: ProcessFilesOptions,
): Promise<ProcessFilesResult> {
  const remoteMode = options?.remoteMode ?? false;
  const attachments: Attachment[] = [];
  const errors: string[] = [];

  for (const file of files) {
    const mime = file.type || 'application/octet-stream';

    // 图片
    if (isImageMime(mime)) {
      if (file.size > MAX_IMAGE_BYTES) {
        const sizeMB = (file.size / (1024 * 1024)).toFixed(1);
        errors.push(`图片 ${file.name} 过大（${sizeMB}MB），最大 ${MAX_IMAGE_BYTES / (1024 * 1024)}MB`);
        continue;
      }
      try {
        const dataUrl = await readAsDataUrl(file);
        attachments.push({
          id: generateId(),
          kind: 'image',
          name: file.name,
          mime,
          size: file.size,
          dataUrl,
        });
      } catch (e) {
        errors.push(`读取图片 ${file.name} 失败: ${e instanceof Error ? e.message : String(e)}`);
      }
      continue;
    }

    // 文本
    if (isTextFile(mime, file.name)) {
      if (file.size > MAX_TEXT_BYTES) {
        const sizeKB = (file.size / 1024).toFixed(1);
        errors.push(`文本文件 ${file.name} 过大（${sizeKB}KB），最大 ${MAX_TEXT_BYTES / 1024}KB`);
        continue;
      }
      try {
        const text = await readAsText(file);
        attachments.push({
          id: generateId(),
          kind: 'text-file',
          name: file.name,
          mime,
          size: file.size,
          text,
        });
      } catch (e) {
        errors.push(`读取文件 ${file.name} 失败: ${e instanceof Error ? e.message : String(e)}`);
      }
      continue;
    }

    // 二进制文件
    if (remoteMode) {
      errors.push(`二进制文件 ${file.name} (${mime}) 不支持在远程模式下使用`);
      continue;
    }

    // 本地模式：path-ref（文件路径需由调用方填充）
    attachments.push({
      id: generateId(),
      kind: 'path-ref',
      name: file.name,
      mime,
      size: file.size,
    });
  }

  return { attachments, errors };
}
