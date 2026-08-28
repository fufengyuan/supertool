//! AI 助手 —— 项目指南知识库（编译期内嵌基准快照）
//!
//! 把项目根目录的 AGENTS.md 与 docs/*.md **编译期内嵌**成可检索条目，
//! 助手用它查本项目约定、踩坑结论、维护文档——出现问题时先在这里找真实原因。
//!
//! 与 `knowledge.rs`（配置教学条目）的区别：
//! - knowledge.rs 是手工提炼的「怎么配字段 / 报错怎么办」短条目；
//! - 这里是**文档全文**：内容随仓库同步，改文档重新构建即生效，不依赖运行时文件路径。
//!
//! include_str! 路径相对本源文件（tauri/src/assistant/）：`../../../` 即项目根（supertool/）。

use serde_json::{Value, json};

pub struct ProjectGuide {
    pub id: &'static str,
    pub title: &'static str,
    pub module: &'static str,
    /// 文档相对项目根的路径（仅展示用，不用于读文件）
    pub path: &'static str,
    pub keywords: &'static [&'static str],
    pub body: &'static str,
}

pub const PROJECT_GUIDES: &[ProjectGuide] = &[
    ProjectGuide {
        id: "project-agents",
        title: "项目总约定（AGENTS.md）",
        module: "项目",
        path: "AGENTS.md",
        keywords: &["AGENTS", "约定", "规则", "架构", "模块", "CICD", "部署", "助手", "提交规范"],
        body: include_str!("../../../AGENTS.md"),
    },
    ProjectGuide {
        id: "project-doc-ai-config-assistant",
        title: "AI 配置助手实现文档",
        module: "项目",
        path: "docs/ai-config-assistant.md",
        keywords: &["助手", "AI", "工具集", "知识库", "安全红线", "事件", "提示词"],
        body: include_str!("../../../docs/ai-config-assistant.md"),
    },
    ProjectGuide {
        id: "project-doc-cicd-multi-env",
        title: "CICD 多环境部署实现文档",
        module: "项目",
        path: "docs/cicd-multi-env-deploy.md",
        keywords: &["CICD", "多环境", "部署", "environments", "构建目录", "产物目录"],
        body: include_str!("../../../docs/cicd-multi-env-deploy.md"),
    },
    ProjectGuide {
        id: "project-doc-cicd-freeze",
        title: "CICD 部署进度事件防卡死",
        module: "项目",
        path: "docs/cicd-deploy-ui-freeze.md",
        keywords: &["卡死", "冻结", "部署进度", "攒批", "事件风暴", "emit"],
        body: include_str!("../../../docs/cicd-deploy-ui-freeze.md"),
    },
    ProjectGuide {
        id: "project-doc-mcp",
        title: "MCP 集成文档",
        module: "项目",
        path: "docs/mcp-integration.md",
        keywords: &["MCP", "集成", "协议", "工具"],
        body: include_str!("../../../docs/mcp-integration.md"),
    },
    ProjectGuide {
        id: "project-doc-ai-iteration",
        title: "AI 工具链迭代计划",
        module: "项目",
        path: "docs/ai-tooling-iteration-plan.md",
        keywords: &["迭代", "计划", "工具链", "roadmap"],
        body: include_str!("../../../docs/ai-tooling-iteration-plan.md"),
    },
    ProjectGuide {
        id: "project-doc-chat-audit",
        title: "助手对话审计报告",
        module: "项目",
        path: "docs/AGENT_CHAT_AUDIT_REPORT.md",
        keywords: &["审计", "对话", "报告", "安全", "redact", "脱敏"],
        body: include_str!("../../../docs/AGENT_CHAT_AUDIT_REPORT.md"),
    },
];

fn score(guide: &ProjectGuide, query: &str) -> usize {
    let mut s = 0usize;
    if guide.title.to_lowercase().contains(&query) {
        s += 5;
    }
    for k in guide.keywords {
        let k = k.to_lowercase();
        if query.contains(&k) || k.contains(&query) {
            s += 3;
        }
    }
    if guide.body.to_lowercase().contains(&query) {
        s += 2;
    }
    if guide.module.to_lowercase().contains(&query) {
        s += 1;
    }
    s
}

/// 检索项目指南（命中按相关度排序，正文只给开头预览，全文用 get_project_guide）
pub fn search_project_guides(query: &str, limit: usize) -> Vec<Value> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(usize, &ProjectGuide)> = PROJECT_GUIDES
        .iter()
        .map(|g| (score(g, &q), g))
        .filter(|(s, _)| *s > 0)
        .collect();
    hits.sort_by(|a, b| b.0.cmp(&a.0));
    hits.into_iter()
        .take(limit)
        .map(|(s, g)| {
            json!({
                "id": g.id, "title": g.title, "module": g.module, "path": g.path,
                "relevance": s, "body": preview(&g.body),
            })
        })
        .collect()
}

/// 取整篇文档全文
pub fn get_project_guide(id: &str) -> Option<Value> {
    PROJECT_GUIDES.iter().find(|g| g.id == id).map(|g| {
        json!({
            "id": g.id, "title": g.title, "module": g.module, "path": g.path, "body": g.body,
        })
    })
}

pub fn project_guide_index() -> Value {
    Value::Array(
        PROJECT_GUIDES
            .iter()
            .map(|g| json!({ "id": g.id, "title": g.title, "module": g.module, "path": g.path }))
            .collect(),
    )
}

/// 正文预览：前 600 字（搜索结果不整篇进上下文，需要全文时再 get）
fn preview(body: &str) -> String {
    let text = body.trim();
    let chars: Vec<char> = text.chars().take(600).collect();
    let mut out: String = chars.into_iter().collect();
    if text.chars().count() > 600 {
        out.push_str("……（正文过长已截断，可用 get_project_guide 取全文）");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embeds_the_project_contract_docs() {
        assert_eq!(PROJECT_GUIDES.len(), 7, "AGENTS + docs/ 全部文档都要内嵌");
        let agents = get_project_guide("project-agents").unwrap();
        let body = agents["body"].as_str().unwrap();
        assert!(body.contains("SuperTool"), "AGENTS.md 全文应内嵌");
        assert!(body.contains("提交规范"), "AGENTS.md 全文应完整内嵌");
    }

    #[test]
    fn searches_and_previews() {
        let hits = search_project_guides("产物目录", 3);
        assert!(!hits.is_empty(), "应按关键词命中 CICD 文档");
        let first = hits[0]["body"].as_str().unwrap();
        assert!(first.contains("……") || first.len() <= 600, "搜索结果是预览而非全文: {} 字", first.chars().count());
    }

    #[test]
    fn unrelated_query_returns_nothing() {
        assert!(search_project_guides("zzz完全不存在的词qqq", 5).is_empty());
        assert!(search_project_guides("  ", 5).is_empty());
    }

    #[test]
    fn every_doc_has_id_and_index_consistency() {
        for g in PROJECT_GUIDES {
            assert!(!g.id.is_empty());
            assert!(!g.body.trim().is_empty(), "{} 正文不能为空", g.id);
        }
        assert_eq!(project_guide_index().as_array().unwrap().len(), PROJECT_GUIDES.len());
    }
}
