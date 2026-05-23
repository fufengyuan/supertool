import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js/lib/core';

// 注册语言（由调用者负责注册）
// hljs.registerLanguage('bash', bash);
// hljs.registerLanguage('json', json);
// etc.

// Markdown 渲染缓存
const markdownCache = new Map<string, string>();
const MAX_CACHE = 500;

// 自定义渲染器单例
const markdownRenderer = new marked.Renderer();

// 代码块渲染（带复制按钮）
markdownRenderer.code = function({ text: code, lang }: { text: string; lang?: string }): string {
  const language = lang || 'plaintext';
  
  // 检测是否已高亮
  if (code.includes('class="hljs-') || code.includes('class=\'hljs-')) {
    const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
    return `<div class="code-block-wrapper">
      <div class="code-header">
        <span class="code-lang">${language}</span>
        <button class="copy-btn" onclick="copyCode('${codeId}')">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
        </button>
      </div>
      <pre><code id="${codeId}" class="hljs">${code}</code></pre>
    </div>`;
  }
  
  const highlighted = language && hljs.getLanguage(language) 
    ? hljs.highlight(code, { language }).value 
    : hljs.highlightAuto(code).value;
  
  const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;
  
  return `<div class="code-block-wrapper">
    <div class="code-header">
      <span class="code-lang">${language}</span>
      <button class="copy-btn" onclick="copyCode('${codeId}')" title="复制代码">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
      </button>
    </div>
    <pre><code id="${codeId}" class="hljs">${highlighted}</code></pre>
  </div>`;
};

/**
 * Markdown 渲染函数
 * - 代码块带复制按钮
 * - 特殊警告框格式
 * - LRU 缓存
 * - DOMPurify 消毒
 */
export function renderMarkdown(text: string | null): string {
  if (!text) {return '';}
  
  // 缓存命中
  const cached = markdownCache.get(text);
  if (cached) {return cached;}
  
  try {
    // 预处理：特殊格式警告框
    let processedText = text
      .replace(/^\[IMPORTANT:\s*([^\]]+)\]/gm, '<div class="alert-box alert-important">⚠️ <strong>重要:</strong> $1</div>')
      .replace(/^\[WARNING:\s*([^\]]+)\]/gm, '<div class="alert-box alert-warning">⚠️ <strong>警告:</strong> $1</div>')
      .replace(/^\[NOTE:\s*([^\]]+)\]/gm, '<div class="alert-box alert-note">📝 <strong>注意:</strong> $1</div>')
      .replace(/^\[SILENT\]/gm, '<div class="alert-box alert-silent">🔇 <strong>静默模式</strong></div>')
      .replace(/^\[CONTEXT:/gm, '<div class="alert-box alert-context">📋 <strong>上下文压缩摘要</strong><br>');

    // 渲染
    const html = marked.parse(processedText, {
      renderer: markdownRenderer,
      breaks: true,
      gfm: true,
      async: false,
    }) as string;
    
    // 消毒
    const result = DOMPurify.sanitize(html, {
      ADD_ATTR: ['target', 'onclick', 'id', 'title'],
      ADD_TAGS: ['button', 'svg', 'rect', 'path', 'div'],
    });

    // LRU 缓存淘汰
    if (markdownCache.size >= MAX_CACHE) {
      const firstKey = markdownCache.keys().next().value;
      if (firstKey) {markdownCache.delete(firstKey);}
    }
    markdownCache.set(text, result);
    
    return result;
  } catch (e) {
    console.error('Markdown render error:', e);
    return `<div class="text-error">渲染错误: ${e}</div>`;
  }
}

/**
 * 复制代码功能（挂载到 window）
 */
export function setupCopyCode() {
  const copyCode = (codeId: string) => {
    const codeElement = document.getElementById(codeId);
    if (codeElement) {
      const text = codeElement.textContent || '';
      navigator.clipboard.writeText(text).then(() => {
        // 显示复制成功提示（简短闪烁）
        const btn = codeElement.closest('.code-block-wrapper')?.querySelector('.copy-btn');
        if (btn) {
          btn.classList.add('copied');
          setTimeout(() => btn.classList.remove('copied'), 1000);
        }
      }).catch((err) => {
        console.error('复制失败:', err);
      });
    }
  };
  
  // 挂载到 window
  if (typeof window !== 'undefined') {
    (window as any).copyCode = copyCode;
  }
  
  return copyCode;
}

/**
 * 清除缓存（用于测试或内存优化）
 */
export function clearMarkdownCache() {
  markdownCache.clear();
}