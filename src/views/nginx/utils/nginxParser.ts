/**
 * Nginx Config Parser & Serializer
 *
 * Parses nginx config text into a structured tree of blocks and directives,
 * preserves comments, and serializes back.
 *
 * Supports:
 * - All block types (server, location, upstream, http, events, map, geo, types, if, limit_except, ...)
 * - Comments (# ...) preserved as leading comments on directives/blocks
 * - Location modifiers (=, ~, ~*, ^~)
 * - Variables ($uri, $scheme, etc.)
 * - Quoted strings
 * - Nested blocks
 * - Multiple values per directive
 */

// ============ Types ============

export interface NginxDirective {
  name: string
  params: string[]         // e.g. ['localhost:8080', 'weight=5']
  raw: string              // original text for preservation
  commentBefore?: string   // comment right before this directive, e.g. '# SSL'
}

export interface NginxBlock {
  type: string             // 'server', 'location', 'upstream', 'http', 'events', 'map', 'geo', 'if', 'limit_except', 'types', etc.
  name: string             // full header: 'location /api', 'server', 'upstream backend'
  params: string[]         // ['/api'] for 'location /api', ['backend'] for 'upstream backend'
  directives: NginxDirective[]
  blocks: NginxBlock[]
  commentBefore?: string   // comment right before this block header
  isParsed: boolean
}

export interface ParsedNginxConfig {
  blocks: NginxBlock[]
  errors: string[]         // parse warnings (non-fatal)
}

// ============ Tokenizer ============

interface Token {
  type: 'word' | 'semicolon' | 'brace_open' | 'brace_close' | 'comment' | 'eof'
  value: string
}

function tokenize(input: string): Token[] {
  const tokens: Token[] = []
  let i = 0
  const len = input.length

  while (i < len) {
    // Skip whitespace
    if (/\s/.test(input[i])) {
      i++
      continue
    }

    // Comment
    if (input[i] === '#') {
      let comment = ''
      while (i < len && input[i] !== '\n') {
        comment += input[i]
        i++
      }
      tokens.push({ type: 'comment', value: comment })
      continue
    }

    // Semicolon
    if (input[i] === ';') {
      tokens.push({ type: 'semicolon', value: ';' })
      i++
      continue
    }

    // Brace open
    if (input[i] === '{') {
      tokens.push({ type: 'brace_open', value: '{' })
      i++
      continue
    }

    // Brace close
    if (input[i] === '}') {
      tokens.push({ type: 'brace_close', value: '}' })
      i++
      continue
    }

    // Quoted string (single or double)
    if (input[i] === "'" || input[i] === '"') {
      const quote = input[i]
      let word = quote
      i++
      while (i < len) {
        word += input[i]
        if (input[i] === quote && input[i - 1] !== '\\') {
          i++
          break
        }
        i++
      }
      tokens.push({ type: 'word', value: word })
      continue
    }

    // Regular word: any non-whitespace, non-special chars
    let word = ''
    while (i < len && !/\s/.test(input[i]) && input[i] !== ';' && input[i] !== '{' && input[i] !== '}') {
      word += input[i]
      i++
    }
    if (word) {
      tokens.push({ type: 'word', value: word })
    }
  }

  tokens.push({ type: 'eof', value: '' })
  return tokens
}

// ============ Parser ============

let tokenPos = 0
let allTokens: Token[] = []

function peek(): Token {
  return allTokens[tokenPos] || { type: 'eof', value: '' }
}

function consume(): Token {
  const tok = allTokens[tokenPos] || { type: 'eof', value: '' }
  tokenPos++
  return tok
}

/** Consume and return any pending comment token */
function consumeComment(): string | undefined {
  const tok = peek()
  if (tok.type === 'comment') {
    consume()
    return tok.value
  }
  return undefined
}

/** Collect all consecutive comments and return them joined */
function consumeAllComments(): string | undefined {
  const comments: string[] = []
  while (peek().type === 'comment') {
    comments.push(consume().value)
  }
  return comments.length > 0 ? comments.join('\n') : undefined
}

function parseDirective(): { directive: NginxDirective | null; commentBefore?: string } {
  // Capture any leading comment
  const commentBefore = consumeAllComments()
  const words: string[] = []

  while (tokenPos < allTokens.length) {
    const tok = peek()
    if (tok.type === 'semicolon') {
      consume()
      if (words.length === 0) return { directive: null, commentBefore }
      const name = words[0]
      const params = words.slice(1)
      return {
        directive: {
          name,
          params,
          raw: name + (params.length > 0 ? ' ' + params.join(' ') : '') + ';',
          commentBefore,
        },
        commentBefore,
      }
    }
    if (tok.type === 'brace_open' || tok.type === 'brace_close' || tok.type === 'eof') {
      return { directive: null, commentBefore }
    }
    // Skip inline comments (shouldn't happen here but just in case)
    if (tok.type === 'comment') {
      consume()
      continue
    }
    words.push(consume().value)
  }
  return { directive: null, commentBefore }
}

function parseBlock(): NginxBlock | null {
  // Save position so we can restore if it turns out to be a directive, not a block
  const startPos = tokenPos
  const commentBefore = consumeAllComments()
  const headerWords: string[] = []

  while (tokenPos < allTokens.length) {
    const tok = peek()
    if (tok.type === 'brace_open') {
      consume() // skip {
      if (headerWords.length === 0) return null
      const type = headerWords[0]
      const params = headerWords.slice(1)
      const directives: NginxDirective[] = []
      const blocks: NginxBlock[] = []

      // Parse contents until }
      while (tokenPos < allTokens.length) {
        const next = peek()
        if (next.type === 'brace_close') {
          consume() // skip }
          return {
            type,
            name: headerWords.join(' '),
            params,
            directives,
            blocks,
            commentBefore,
            isParsed: true,
          }
        }
        if (next.type === 'eof') {
          return {
            type,
            name: headerWords.join(' '),
            params,
            directives,
            blocks,
            commentBefore,
            isParsed: false,
          }
        }
        // Try to parse as block first, then directive (both handle leading comments internally)
        const savedPos = tokenPos
        const block = parseBlock()
        if (block) {
          blocks.push(block)
        } else {
          const savedPos2 = tokenPos
          const { directive, commentBefore: dirComment } = parseDirective()
          if (directive) {
            directives.push(directive)
          } else if (tokenPos === savedPos2) {
            // Nothing consumed at all — eat the current token to avoid infinite loop
            consume()
          } else {
            // parseDirective consumed some tokens but failed (e.g. encountered '{')
            // This shouldn't happen if parseBlock is tried first; break to avoid data loss.
            // Restore position and try once more as directive
            tokenPos = savedPos2
            const retryDir = tryParseDirectiveBody()
            if (retryDir) {
              directives.push(retryDir)
            } else {
              break
            }
          }
        }
      }

      return {
        type,
        name: headerWords.join(' '),
        params,
        directives,
        blocks,
        commentBefore,
        isParsed: true,
      }
    }
    if (tok.type === 'semicolon' || tok.type === 'brace_close' || tok.type === 'eof') {
      // Not a block — restore position so caller can parse as directive
      tokenPos = startPos
      return null
    }
    if (tok.type === 'comment') {
      tokenPos = startPos
      return null
    }
    headerWords.push(consume().value)
  }
  tokenPos = startPos
  return null
}

/**
 * Parse a single directive body aggressively — consume tokens until ';' is found.
 * Used as fallback when normal parsing fails within a block body.
 */
function tryParseDirectiveBody(): NginxDirective | null {
  const words: string[] = []
  while (tokenPos < allTokens.length) {
    const tok = peek()
    if (tok.type === 'semicolon') {
      consume()
      if (words.length === 0) return null
      const name = words[0]
      const params = words.slice(1)
      return {
        name,
        params,
        raw: name + (params.length > 0 ? ' ' + params.join(' ') : '') + ';',
      }
    }
    if (tok.type === 'brace_open') {
      // We're inside a block body and found '{' without preceding words
      // This is a sub-block header — let parseBlock handle it (restore position)
      return null
    }
    if (tok.type === 'brace_close' || tok.type === 'eof') {
      return null
    }
    words.push(consume().value)
  }
  return null
}

export function parseNginxConfig(input: string): ParsedNginxConfig {
  allTokens = tokenize(input)
  tokenPos = 0
  const blocks: NginxBlock[] = []
  const errors: string[] = []

  while (tokenPos < allTokens.length) {
    const tok = peek()
    if (tok.type === 'eof') break
    if (tok.type === 'comment') {
      consume()
      continue
    }
    const block = parseBlock()
    if (block) {
      blocks.push(block)
    } else {
      consume()
    }
  }

  return { blocks, errors }
}

// ============ Quoted-string-aware param helpers ============

/**
 * Split a string into params, respecting single/double quoted groups.
 * e.g. 'Access-Control-Allow-Origin "*"' → ['Access-Control-Allow-Origin', '"*"']
 * e.g. '"GET, POST"' → ['"GET, POST"']
 */
function splitParamsSmart(input: string): string[] {
  const parts: string[] = []
  let i = 0
  const len = input.length
  while (i < len) {
    // Skip whitespace
    if (/\s/.test(input[i])) { i++; continue }
    // Quoted section (single or double)
    if (input[i] === "'" || input[i] === '"') {
      const quote = input[i]
      let chunk = quote
      i++
      while (i < len) {
        chunk += input[i]
        if (input[i] === quote && input[i - 1] !== '\\') {
          i++
          break
        }
        i++
      }
      parts.push(chunk)
      continue
    }
    // Regular non-whitespace chunk
    let chunk = ''
    while (i < len && !/\s/.test(input[i])) {
      chunk += input[i]
      i++
    }
    if (chunk) parts.push(chunk)
  }
  return parts
}

/**
 * Join params into a display string, wrapping values with spaces in quotes.
 * e.g. ['Access-Control-Allow-Origin', '*'] → 'Access-Control-Allow-Origin *'
 * e.g. ['GET, POST'] → '"GET, POST"'  NOT: just joins with space
 */
function joinParamsDisplay(params: string[]): string {
  return params.map(p => {
    // If already quoted, return as-is
    if ((p.startsWith("'") && p.endsWith("'")) || (p.startsWith('"') && p.endsWith('"'))) return p
    // If contains spaces that aren't part of the param value, wrap in quotes
    if (/\s/.test(p)) return `"${p}"`
    return p
  }).join(' ')
}

// ============ Serializer ============

function serializeDirective(d: NginxDirective, indent: number): string {
  const ind = '  '.repeat(indent)
  const comment = d.commentBefore ? d.commentBefore + '\n' : ''
  const params = d.params.length > 0 ? ' ' + joinParamsDisplay(d.params) : ''
  return `${comment}${ind}${d.name}${params};\n`
}

function serializeBlock(b: NginxBlock, indent: number): string {
  const ind = '  '.repeat(indent)
  const comment = b.commentBefore ? b.commentBefore + '\n' : ''
  let result = `${comment}${ind}${b.name} {\n`
  for (const d of b.directives) {
    result += serializeDirective(d, indent + 1)
  }
  for (const child of b.blocks) {
    result += serializeBlock(child, indent + 1)
  }
  result += `${ind}}\n`
  return result
}

export function serializeNginxConfig(parsed: { blocks: NginxBlock[] }): string {
  let result = ''
  for (const block of parsed.blocks) {
    result += serializeBlock(block, 0)
  }
  return result
}

// ============ Block Type Helpers ============

export const NGINX_BLOCK_TYPES = new Set([
  'http', 'server', 'location', 'upstream', 'events',
  'map', 'geo', 'types', 'if', 'limit_except',
  'stream', 'mail', 'imap', 'auth_basic_user_file',
])

export const LOCATION_MODIFIERS = new Set(['=', '~', '~*', '^~'])

/** Get location modifier if present (e.g., '~*' from 'location ~* \.php$') */
export function getLocationModifier(params: string[]): string {
  if (params.length > 0 && LOCATION_MODIFIERS.has(params[0])) return params[0]
  return ''
}

/** Get location path without modifier */
export function getLocationPath(params: string[]): string {
  if (params.length > 0 && LOCATION_MODIFIERS.has(params[0])) return params.slice(1).join(' ')
  return params.join(' ')
}

// ============ Summary / Introspection Types ============

export interface ServerBlockSummary {
  listen: string[]
  serverName: string[]
  root: string
  index: string[]
  sslEnabled: boolean
  ssl: NginxDirective[]
  locations: LocationBlockSummary[]
  subBlocks: NginxBlock[]  // non-location sub-blocks (if, limit_except, etc.)
  other: NginxDirective[]
  block: NginxBlock
}

export interface LocationBlockSummary {
  modifier: string          // '', '=', '~', '~*', '^~'
  path: string
  proxyPass: string
  root: string
  tryFiles: string
  headers: NginxDirective[]
  subBlocks: NginxBlock[]  // nested blocks (if, limit_except, etc.)
  other: NginxDirective[]
  block: NginxBlock
}

export interface UpstreamSummary {
  name: string
  servers: UpstreamServer[]
  other: NginxDirective[]
  block: NginxBlock
}

export interface UpstreamServer {
  address: string
  weight?: number
  maxFails?: number
  maxConns?: number
  failTimeout?: string
  backup?: boolean
  down?: boolean
}

// Extract values from directives with a given name
function getDirectiveParams(dirs: NginxDirective[], name: string): string[] {
  return dirs.filter(d => d.name === name).flatMap(d => d.params)
}

function findFirstDirectiveParams(dirs: NginxDirective[], name: string): string[] {
  const d = dirs.find(d => d.name === name)
  return d ? d.params : []
}

/** SSL-related directive names */
const SSL_DIRECTIVES = new Set([
  'ssl_certificate', 'ssl_certificate_key', 'ssl_protocols',
  'ssl_ciphers', 'ssl_prefer_server_ciphers', 'ssl_session_cache',
  'ssl_session_timeout', 'ssl_dhparam', 'ssl_stapling',
  'ssl_stapling_verify', 'ssl_trusted_certificate', 'ssl_session_tickets',
  'ssl_buffer_size', 'ssl_ecdh_curve', 'ssl_password_file',
  'ssl_crl', 'ssl_ocsp', 'ssl_ocsp_cache',
])

// ============ Summarizers ============

export function summarizeServerBlock(block: NginxBlock): ServerBlockSummary {
  const locationBlocks = block.blocks.filter(b => b.type === 'location')
  const nonLocationBlocks = block.blocks.filter(b => b.type !== 'location')

  return {
    listen: getDirectiveParams(block.directives, 'listen'),
    serverName: getDirectiveParams(block.directives, 'server_name'),
    root: findFirstDirectiveParams(block.directives, 'root').join(' '),
    index: getDirectiveParams(block.directives, 'index'),
    sslEnabled: block.directives.some(d => SSL_DIRECTIVES.has(d.name)),
    ssl: block.directives.filter(d => SSL_DIRECTIVES.has(d.name)),
    locations: locationBlocks.map(lb => summarizeLocationBlock(lb)),
    subBlocks: nonLocationBlocks,
    other: block.directives.filter(d => !['listen', 'server_name', 'root', 'index'].includes(d.name) && !SSL_DIRECTIVES.has(d.name)),
    block,
  }
}

export function summarizeLocationBlock(block: NginxBlock): LocationBlockSummary {
  const subBlocks = block.blocks.filter(b => b.type !== 'location')

  return {
    modifier: getLocationModifier(block.params),
    path: getLocationPath(block.params),
    proxyPass: findFirstDirectiveParams(block.directives, 'proxy_pass').join(' '),
    root: findFirstDirectiveParams(block.directives, 'root').join(' '),
    tryFiles: findFirstDirectiveParams(block.directives, 'try_files').join(' '),
    headers: block.directives.filter(d => d.name.startsWith('proxy_set_header') || d.name.startsWith('add_header') || d.name.startsWith('more_set_headers')),
    subBlocks,
    other: block.directives.filter(d =>
      !['proxy_pass', 'root', 'try_files', 'proxy_set_header', 'add_header', 'more_set_headers'].includes(d.name)
    ),
    block,
  }
}

export function summarizeUpstream(block: NginxBlock): UpstreamSummary {
  const serverDirs = block.directives.filter(d => d.name === 'server')
  return {
    name: block.params.join(' '),
    servers: serverDirs.map(d => parseUpstreamServer(d)),
    other: block.directives.filter(d => d.name !== 'server'),
    block,
  }
}

function parseUpstreamServer(d: NginxDirective): UpstreamServer {
  const result: UpstreamServer = { address: d.params[0] || '' }
  for (let i = 1; i < d.params.length; i++) {
    const p = d.params[i]
    if (p === 'backup') result.backup = true
    else if (p === 'down') result.down = true
    else if (p.startsWith('weight=')) result.weight = parseInt(p.split('=')[1]) || undefined
    else if (p.startsWith('max_fails=')) result.maxFails = parseInt(p.split('=')[1]) || undefined
    else if (p.startsWith('max_conns=')) result.maxConns = parseInt(p.split('=')[1]) || undefined
    else if (p.startsWith('fail_timeout=')) result.failTimeout = p.split('=')[1]
  }
  return result
}

// ============ Mutators ============

export function createDirective(name: string, params: string[] = [], commentBefore?: string): NginxDirective {
  return {
    name,
    params,
    raw: name + (params.length > 0 ? ' ' + joinParamsDisplay(params) : '') + ';',
    commentBefore,
  }
}

// Export helpers for use in Vue components
export { splitParamsSmart, joinParamsDisplay }
