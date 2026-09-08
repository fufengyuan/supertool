import { describe, expect, it } from 'vitest'
import { detectFormat, formatText } from '../toolUtils'

// 解密出来的报文常是压缩成一行的 JSON / XML / 查询串，格式化要能正确识别并缩进
describe('报文格式化', () => {
  it('自动识别 JSON 并美化', () => {
    const raw = '{"code":0,"data":[{"id":1857,"label":"phone","value":"028-64295356"}],"msg":"ok"}'
    expect(detectFormat(raw)).toBe('json')
    const pretty = formatText(raw)
    expect(pretty).toContain('\n')
    expect(pretty).toContain('"code": 0')
    expect(pretty).toContain('"id": 1857')
    // 美化后仍可原样解析回同一对象
    expect(JSON.parse(pretty)).toEqual(JSON.parse(raw))
  })

  it('JSON 压缩回单行', () => {
    const pretty = '{\n  "code": 0,\n  "msg": "ok"\n}'
    expect(formatText(pretty, 'json', false)).toBe('{"code":0,"msg":"ok"}')
  })

  it('自动识别 XML 并缩进', () => {
    const raw = '<root><item id="1">a</item><item id="2">b</item></root>'
    expect(detectFormat(raw)).toBe('xml')
    const pretty = formatText(raw)
    expect(pretty).toContain('\n')
    expect(pretty).toMatch(/^\s*<root>/)
    expect(pretty).toContain('<item id="1">a</item>')
  })

  it('自动识别查询串 / 表单串并逐项展开（值做 URL 解码）', () => {
    const raw = 'method=api.xxx&charset=UTF-8&name=%E5%BC%A0%E4%B8%89'
    expect(detectFormat(raw)).toBe('query')
    const pretty = formatText(raw)
    expect(pretty).toBe('method = api.xxx\ncharset = UTF-8\nname = 张三')
  })

  it('完整 URL 只格式化 ? 之后的查询部分', () => {
    const pretty = formatText('https://a.com/api?x=1&y=2')
    expect(pretty).toBe('x = 1\ny = 2')
  })

  it('密文 hex 这类纯文本保持原样', () => {
    const hex = '35474249A1C65E9C53B46F9236D43636'
    expect(detectFormat(hex)).toBe('text')
    expect(formatText(hex)).toBe(hex)
    expect(formatText(hex, 'text', false)).toBe(hex)
  })

  it('手动指定格式优先于自动识别；解析失败时由调用方捕获', () => {
    expect(() => formatText('not json', 'json')).toThrow()
    expect(detectFormat('not json')).toBe('text')
  })
})
