import fs from 'fs'
const f = process.argv[2]
let s = fs.readFileSync(f, 'utf8')
// Remove agent tab button (4 lines)
s = s.replace(
  /      <button class="tab tab-bordered tab-sm flex items-center gap-1"\n        :class="tab === 'agent' \? 'tab-active' : ''"\n        @click="tab = 'agent'">\n        <SvgIcon name="terminal" size="16" \/>\n        <span>Agent<\/span>\n      <\/button>\n/,
  ''
)
// Remove agent config section (from <!-- Agent Tab --> to next <!-- About Tab -->)
s = s.replace(
  /    <!-- ==================== Agent Tab ==================== -->[\s\S]*?<!-- ==================== About Tab ==================== -->/,
  '    <!-- ==================== About Tab ==================== -->'
)
// Remove agent from tab type union
s = s.replace(/'general' \| 'notifications' \| 'shortcuts' \| 'agent' \| 'network' \| 'about'/,
  "'general' | 'notifications' | 'shortcuts' | 'network' | 'about'")
// Remove agentConfig ref
s = s.replace(/\nconst agentConfig = ref\([\s\S]*?\nconst agentSaved/, '\nconst agentSaved')
// Remove agentSaved assignment
s = s.replace(/\nconst agentSaved = ref\(false\)\n/, '\n')
// Remove loadAgentConfig and saveAgentConfig functions
s = s.replace(/async function loadAgentConfig\(\)[\s\S]*?^}\n\n/m, '')
s = s.replace(/async function saveAgentConfig\(\)[\s\S]*?^}\n\n/m, '')
s = s.replace(/async function saveAgentConfigWithKB\(\)[\s\S]*?^}\n\n/m, '')
// Remove loadAgentConfig / saveAgentConfig calls from onMounted
s = s.replace(/\n  loadAgentConfig\(\)\n/, '\n')
fs.writeFileSync(f, s)
console.log('cleaned', f, `now ${s.split('\n').length} lines`)
