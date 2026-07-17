import fs from 'fs'
const f = 'src/types.ts'
let s = fs.readFileSync(f, 'utf8')
// Remove lines 425-517 (1-indexed) which are corrupted agent types
const lines = s.split('\n')
const cleaned = [...lines.slice(0, 424), ...lines.slice(517)]
fs.writeFileSync(f, cleaned.join('\n'))
console.log('fixed', f, `was ${lines.length} lines, now ${cleaned.length} lines`)
