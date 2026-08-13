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

// HTML 转义（language 等拼入模板的值必须转义，否则可注入属性/事件）
function escapeHtml(str: string): string {
  return str.replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' })[c] as string);
}

// 代码块渲染（带复制按钮）
// 安全约定：language 必须转义；code 一律经 hljs 转义后再插入（绝不原样拼 HTML）；
// 复制按钮用 data-copy-target + 事件委托（见 attachCopyHandlers），不用内联 onclick
markdownRenderer.code = function({ text: code, lang }: { text: string; lang?: string }): string {
  const language = escapeHtml(lang || 'plaintext');

  const highlighted = lang && hljs.getLanguage(lang)
    ? hljs.highlight(code, { language: lang }).value
    : hljs.highlightAuto(code).value;

  const codeId = `code-${Math.random().toString(36).substr(2, 9)}`;

  return `<div class="code-block-wrapper">
    <div class="code-header">
      <span class="code-lang">${language}</span>
      <button class="copy-btn" data-copy-target="${codeId}" title="复制代码">
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
      ADD_ATTR: ['target', 'id', 'title', 'align', 'colspan', 'rowspan', 'width', 'height', 'href', 'src', 'alt', 'class', 'style', 'data-copy-target'],
      ADD_TAGS: [
        // 基础结构
        'div', 'span', 'p', 'br', 'hr',
        // 标题
        'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
        // 列表
        'ul', 'ol', 'li', 'dl', 'dt', 'dd',
        // 表格
        'table', 'thead', 'tbody', 'tfoot', 'tr', 'th', 'td', 'colgroup', 'col', 'caption',
        // 链接和图片
        'a', 'img',
        // 引用和代码
        'blockquote', 'pre', 'code', 'kbd', 'samp', 'var',
        // 强调
        'strong', 'em', 'b', 'i', 'u', 's', 'del', 'ins', 'mark', 'sub', 'sup',
        // 按钮
        'button', 'svg', 'rect', 'path', 'circle', 'ellipse', 'line', 'polyline', 'polygon', 'text',
        // 任务列表
        'input',
        // 定义
        'abbr', 'cite', 'dfn', 'time',
      ],
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
 * 复制代码功能（事件委托版）
 * renderMarkdown 生成的复制按钮带 data-copy-target="code-xxx"，调用方把
 * 该函数绑到渲染容器的 click 事件上即可（无需内联 onclick，CSP 安全）。
 */
export function setupCopyCode() {
  return (e: MouseEvent) => {
    const target = e.target as HTMLElement | null
    const btn = target?.closest?.('.copy-btn') as HTMLElement | null
    if (!btn) {return}
    const targetId = btn.getAttribute('data-copy-target')
    if (!targetId) {return}
    const codeElement = document.getElementById(targetId)
    if (!codeElement) {return}
    const text = codeElement.textContent || ''
    navigator.clipboard.writeText(text).then(() => {
      btn.classList.add('copied')
      setTimeout(() => btn.classList.remove('copied'), 1000)
    }).catch((err) => {
      console.error('复制失败:', err)
    })
  }
}

/**
 * 清除缓存（用于测试或内存优化）
 */
export function clearMarkdownCache() {
  markdownCache.clear();
}