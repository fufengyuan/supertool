import fs from 'fs'
const f = process.argv[2]
let s = fs.readFileSync(f, 'utf8')
// Remove remaining agent config functions and vars
s = s.replace(/function saveAgentConfigWithKB\(\)[\s\S]*?^}\n\n/m, '')
s = s.replace(/function saveAgentConfigWithOutputKB\(\)[\s\S]*?^}\n\n/m, '')
s = s.replace(/  await loadAgentConfig\(\)\n/, '')
fs.writeFileSync(f, s)
console.log('fixed', f)
