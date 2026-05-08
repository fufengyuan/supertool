---
name: writing-effective-skills
category: dev
description: How to write skills that AI agents can actually use — focus on triggers and examples, not just command lists.
---

# Writing Effective Skills for AI Agents

## Core Principle

AI agents don't read documentation the way humans do. They need:
1. **Trigger phrases** — exact user phrases that should activate the skill
2. **Concrete examples** — real commands with real arguments, not abstract templates
3. **Scenario context** — "When to use" for each feature

A skill that only lists commands is useless. The user explicitly said: *"给一顿CLI命令的用法，AI哪知道怎么用啊？"*

## Structure

### Every Feature Section Must Have:

```markdown
## Feature Name

**When to use**: Exact user phrases that trigger this feature
  - "add a task", "show my tasks", "what's due today"

```bash
command --flag value          # What it does in plain English
command -j                    # JSON output (for agents)
```

**Examples**:
- Do X: `command --arg "real value" --flag 5`
- Do Y: `command "another real value" -j`
```

### Anti-patterns (DO NOT DO):

❌ Just listing commands without context:
```bash
stool todo add "text" -p high -d date -t tag
stool todo list
stool todo complete <id>
```

❌ Abstract placeholders:
```bash
stool command <id> [--option]   # AI doesn't know what <id> looks like
```

✅ Concrete, real-world examples:
```bash
stool todo add "Fix login bug" -p high -d 2024-12-20 -t 开发
stool todo complete 550e8400-e29b-41d4-a716-446655440000
```

## Full Skill Template

```markdown
---
name: my-skill
category: dev
description: What it does and when to use it
---

# Feature Name

**Golden rule**: One key principle agents must always follow

## Quick Start
```bash
command --help
```

---

## Sub-feature A
**When to use**: User says "exact phrase user would say"

```bash
command -j          # What this does
command --flag val  # What this does
```

**Examples**:
- Real scenario 1: `command "real arg" --flag "real value"`
- Real scenario 2: `command another-arg -j`

---

## Sub-feature B
...

## Pitfalls
1. Gotcha #1
2. Gotcha #2

## Common Agent Workflows
### Workflow Name
```bash
# Step 1
command1 -j
# Step 2
command2 -j | python3 -c "..."
```
```

## Key Takeaways

1. **"When to use"** line is mandatory for every section — maps user intent to commands
2. **Every command gets a real example** — no abstract `<placeholders>` without concrete examples too
3. **Include JSON output patterns** — always show `-j` variant for programmatic use
4. **Show closed-loop workflows** — not just single commands, but multi-step sequences
5. **Version numbers in Quick Start** — so agents know what version they're working with
6. **Pitfalls section** — saves the agent from repeating the same mistakes
