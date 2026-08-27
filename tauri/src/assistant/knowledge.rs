//! AI 助手知识库（内置，不读本地文件）
//!
//! 内容来源是项目里踩过的坑（AGENTS.md 的 CICD 铁律 + docs/*.md 的结论），
//! 转成「字段含义 / 怎么配 / 报错怎么办」的可检索条目。
//! 助手没有文件工具，教学内容一律来自这里，避免把仓库文档暴露成可读文件。

use serde_json::{Value, json};

#[derive(Clone, Copy)]
pub struct Guide {
    pub id: &'static str,
    pub title: &'static str,
    /// 关联功能模块（用于界面「打开对应页面」跳转）
    pub module: &'static str,
    pub route: &'static str,
    /// 小写关键词，命中越多分越高
    pub keywords: &'static [&'static str],
    pub body: &'static str,
}

#[derive(Clone, Copy)]
pub struct ErrorHint {
    /// 小写子串（日志里出现即命中）
    pub pattern: &'static str,
    pub cause: &'static str,
    pub fix: &'static str,
    /// 关联 Guide.id
    pub guide: &'static str,
}

pub const GUIDES: &[Guide] = &[
    Guide {
        id: "cicd-build-vs-output-dir",
        title: "构建目录与产物目录必须分开填",
        module: "持续部署",
        route: "/cicd",
        keywords: &["构建目录", "产物目录", "parentBuildPath", "outputPath", "jar", "dist", "收不到产物"],
        body: "「构建目录」= 在哪里执行 mvn/npm（maven 留空即代码目录，跑 npm 需要该目录有 package.json）。\n\
               「产物目录」= 相对代码目录的产物位置（如 mall-server/target、build/h5）。\n\
               两者是不同字段：在哪构建不代表产物就在那。留空产物目录时会自动扫描，\n\
               若扫描结果不符预期会打「配置的产物目录 X 不存在，已回退自动扫描」的告警——看到这条就把产物目录显式填对。",
    },
    Guide {
        id: "cicd-maven-parent-path",
        title: "maven 多模块（父统一构建）构建目录要留空",
        module: "持续部署",
        route: "/cicd",
        keywords: &["maven", "父统一构建", "多模块", "聚合根", "revision", "mall", "no POM"],
        body: "开启父统一构建时，构建根就是代码目录（聚合根）。构建目录**绝不能填子模块**：\n\
               填子模块会退化成单模块构建，CI-Friendly 项目（聚合根定义 <revision>）的 ${revision} 不展开、\n\
               兄弟模块依赖解析失败。产物从配置级「产物目录」收集（例如 mall-server/target）。\n\
               另外代码若不在仓库根目录（如 SRC/mall 子目录），要用「选择目录」把代码目录定位对。",
    },
    Guide {
        id: "cicd-single-main-module",
        title: "单体部署：主模块目录 = 产物 jar 所在目录",
        module: "持续部署",
        route: "/cicd",
        keywords: &["单体", "主模块", "jar", "拿不到", "seller-api"],
        body: "单体部署的构建目录是「产物 jar 所在的那个模块目录」，经常在子目录里（如 SRC/mall/seller-api）。\n\
               后端按 主模块目录 → 构建目录 → 根目录 解析，保证在哪构建就在哪收集；\n\
               填仓库根目录会扫不到 jar。",
    },
    Guide {
        id: "cicd-npm-single",
        title: "npm/前端单体：只认配置级构建命令",
        module: "持续部署",
        route: "/cicd",
        keywords: &["npm", "pnpm", "yarn", "前端", "build", "模块行", "残留", "mvn clean package"],
        body: "npm 单体模式下模块表里的行**不参与构建**（复制配置常把别的项目的 mvn 命令带过来，会把脚本名解析成 mvn 报错）。\n\
               构建脚本用配置级的「构建脚本 / 自定义脚本」，脚本下拉来自构建目录的 package.json。\n\
               产物目录：uni-app h5 是 dist/build/h5（不是 build/h5）；Vue CLI 一般是 dist。",
    },
    Guide {
        id: "cicd-maven-lock",
        title: "maven 报 Could not acquire lock(s)",
        module: "持续部署",
        route: "/cicd",
        keywords: &["lock", "maven", "-T", "build cache", "并发", "maven.config"],
        body: "项目 .mvn/maven.config 带 -T 1C 并行 + Maven Build Cache 扩展时，CICD 子进程在聚合根构建会锁竞争。\n\
               部署引擎已在构建命令末尾追加 -T 1 与 -Dmaven.build.cache.enabled=false（CLI 参数优先于 maven.config），\n\
               求稳不求快。若仍出现，请确认构建走的是本工具的 maven 分支而非自定义构建命令。",
    },
    Guide {
        id: "cicd-lib-separate",
        title: "Jar/Lib 分离与 lib 过滤规则",
        module: "持续部署",
        route: "/cicd",
        keywords: &["lib", "分离", "过滤规则", "依赖", "瘦包", "spring-boot"],
        body: "仅 maven 生效。开启后主 jar 与依赖 lib.zip 分开上传，lib.zip 只打包过滤规则匹配到的文件（白名单，每行一个通配模式）。\n\
               多模块用模块行的过滤规则，单体用配置级的「lib 过滤规则」。关闭时会自动降级为普通上传。",
    },
    Guide {
        id: "cicd-incremental",
        title: "增量上传与回滚的关系",
        module: "持续部署",
        route: "/cicd",
        keywords: &["增量", "manifest", "hash", "回滚", "跳过上传", "未变更"],
        body: "远端 .deploy_manifest.json 记录文件 SHA-256，只传变化的文件。\n\
               回滚恢复备份后必须删掉 manifest，否则下次增量部署会误判「未变更」而跳过上传。\n\
               现象：日志里出现「N 个文件未变更跳过」但线上仍是旧版本 → 先关增量跑一次全量。",
    },
    Guide {
        id: "cicd-health-rollback",
        title: "健康检查失败与自动回滚",
        module: "持续部署",
        route: "/cicd",
        keywords: &["健康检查", "回滚", "timeout", "retries", "backup", "重启脚本"],
        body: "配了健康检查 URL 才会在覆盖前备份远端文件（.deploy_backup.tar.gz），失败时自动恢复并重跑重启脚本。\n\
               超时/重试次数可按环境配置。回滚结果不会改部署状态，而是在错误信息后追加 rolled-back:success|partial。",
    },
    Guide {
        id: "cicd-multi-env",
        title: "多环境部署配置",
        module: "持续部署",
        route: "/cicd",
        keywords: &["环境", "多环境", "environments", "部署路径", "环境变量"],
        body: "一条配置可挂多个环境（高级设置里），每个环境能单独指定部署路径、服务器、环境变量、健康检查。\n\
               部署时选环境即覆盖对应字段；选「默认（全局配置）」则用配置级字段。",
    },
    Guide {
        id: "cicd-queue",
        title: "同一配置的并发部署会排队",
        module: "持续部署",
        route: "/cicd",
        keywords: &["排队", "并发", "waiting", "acquired", "覆盖"],
        body: "同一部署配置的多次触发会排队执行，防止产物互相覆盖；日志里 stage=queue 表示排队中。\n\
               不同配置之间不互相阻塞。",
    },
    Guide {
        id: "server-fields",
        title: "新增服务器要填哪些字段",
        module: "服务器管理",
        route: "/servers",
        keywords: &["服务器", "新增", "ip", "端口", "用户名", "分组", "ssh", "密钥"],
        body: "非密码类信息：名称、IP/主机、端口（默认 22）、登录用户名、描述、标签、所属分组、是否需要审批。\n\
               密码与 SSH 私钥属于凭据，**只能你自己在表单里填**，AI 助手不接收也不返回这类字段。\n\
               配好后可以先做连通性测试（工具会用已存凭据去连，不会把凭据显示出来）。",
    },
    Guide {
        id: "server-approval",
        title: "服务器审批与危险命令",
        module: "服务器管理",
        route: "/servers",
        keywords: &["审批", "requiresApproval", "危险", "rm", "重启"],
        body: "给生产服务器打开「需要审批」后，执行类操作要先确认，降低误操作。\n\
               删除类命令属于危险操作，界面会强制二次确认。",
    },
    Guide {
        id: "db-connection-fields",
        title: "数据库连接的配置方式",
        module: "数据库管理",
        route: "/db",
        keywords: &["数据库", "连接", "mysql", "postgres", "redis", "sqlite", "端口", "库名"],
        body: "非密码类信息：连接名、类型、主机、端口、用户名、默认库、（SQLite 是文件路径）。\n\
               密码同样属于凭据，由你在表单里填写；助手可以帮你判断字段填法与连通性排查思路。",
    },
    Guide {
        id: "ai-model-setup",
        title: "怎么给 AI 助手配模型",
        module: "设置",
        route: "/settings",
        keywords: &["模型", "提供商", "api", "baseUrl", "协议", "上下文窗口", "模型ID"],
        body: "设置 → AI 模型：新增提供商时填「名称 + 协议（OpenAI 兼容 / Anthropic）+ 接口地址 + apiKey」，\n\
               再为它添加模型：模型 ID 是请求里实际发送的名字（不同网关命名不同，可自由填），\n\
               上下文窗口决定助手能带多少历史（会据此自动裁剪），输出上限用于限制单次回复长度。\n\
               接口地址支持内网/本机（如 http://127.0.0.1:11434/v1），密钥留空表示该服务不需要。\n\
               保存后用「测试」按钮验证协议/地址/密钥/模型 ID 是否真的可用。",
    },
    Guide {
        id: "deploy-freeze",
        title: "部署时界面卡顿",
        module: "持续部署",
        route: "/cicd",
        keywords: &["卡", "卡顿", "无响应", "冻结", "emit", "日志太多"],
        body: "构建日志逐行推送会在 macOS 上打满主线程导致窗口无响应，6.50.6 起后端已按 200ms 攒批发送。\n\
               若仍感觉卡顿：减少同时部署的配置数、或把只关心结果的构建改成静默模式。",
    },
];

/// 报错特征 → 原因 + 处理办法（pattern 一律小写子串匹配）
pub const ERROR_HINTS: &[ErrorHint] = &[
    ErrorHint { pattern: "could not acquire lock", cause: "maven 并行构建 + Build Cache 扩展争抢本地仓库锁", fix: "构建命令追加 -T 1 -Dmaven.build.cache.enabled=false（本工具已内置，自定义构建命令需自行加上）", guide: "cicd-maven-lock" },
    ErrorHint { pattern: "no pom in this directory", cause: "构建目录里没有 pom.xml", fix: "把构建目录改成聚合根或填对代码目录；maven 父统一构建时该字段留空", guide: "cicd-maven-parent-path" },
    ErrorHint { pattern: "the following artifacts could not be resolved", cause: "依赖拉不到（私服地址或 settings.xml 不对）", fix: "检查 maven settings 路径与私服可达性；父统一构建不要只构建子模块", guide: "cicd-maven-parent-path" },
    ErrorHint { pattern: "non-resolvable parent pom", cause: "父 POM 解析失败，常见于 ${revision} 未展开", fix: "构建目录留空（在聚合根构建），不要在子模块单独构建", guide: "cicd-maven-parent-path" },
    ErrorHint { pattern: "build failure", cause: "构建本身失败", fix: "看日志里第一个 [ERROR] 行定位源码问题，报错行不会被攒批，会即时出现在实时日志", guide: "cicd-build-vs-output-dir" },
    ErrorHint { pattern: "missing script", cause: "package.json 里没有该构建脚本", fix: "改用配置级「构建脚本」下拉里的可用项，或填自定义脚本", guide: "cicd-npm-single" },
    ErrorHint { pattern: "command not found", cause: "构建工具不在子进程 PATH 里（nvm/homebrew 安装）", fix: "在高级设置里填 javaHome / mavenHome / nodeHome 等绝对路径", guide: "cicd-build-vs-output-dir" },
    ErrorHint { pattern: "java_home", cause: "JAVA_HOME 配置无效或不存在", fix: "填真实 JDK 目录（注意 sdkman 的实际版本号），或清空让子进程继承", guide: "cicd-build-vs-output-dir" },
    ErrorHint { pattern: "未找到产物", cause: "构建成功但产物目录没对上", fix: "显式填写产物目录（maven 填 xxx/target；uni-app h5 填 dist/build/h5）", guide: "cicd-build-vs-output-dir" },
    ErrorHint { pattern: "已回退自动扫描", cause: "配置的产物目录不存在（不是错误，是提示）", fix: "把产物目录改成实际存在的相对路径；uni-app 注意是 dist/build/h5", guide: "cicd-npm-single" },
    ErrorHint { pattern: "permission denied (publickey", cause: "SSH 认证方式不对（用户名/密钥/密码）", fix: "核对登录用户名，并在服务器上补密钥或密码（凭据需你自己在表单填写）", guide: "server-fields" },
    ErrorHint { pattern: "unable to authenticate", cause: "凭据被服务端拒绝", fix: "同上，确认用户名与认证方式；生产机常禁用密码登录只允许密钥", guide: "server-fields" },
    ErrorHint { pattern: "connection refused", cause: "端口没开或服务未监听", fix: "确认端口（默认 22）、目标机 sshd 状态与安全组", guide: "server-fields" },
    ErrorHint { pattern: "connection timed out", cause: "网络不通 / 防火墙 / IP 写错", fix: "核对 IP，检查中间网络与防火墙规则", guide: "server-fields" },
    ErrorHint { pattern: "no space left on device", cause: "远端磁盘满", fix: "清理目标机磁盘（尤其 /opt 与日志目录）后重试；工具里有磁盘清理功能", guide: "cicd-incremental" },
    ErrorHint { pattern: "unzip: command not found", cause: "远端缺少 unzip，dist 包无法解压", fix: "在目标机安装 unzip，或改为上传已解压目录", guide: "cicd-npm-single" },
    ErrorHint { pattern: "健康检查未通过", cause: "服务没起来或健康检查地址/超时不对", fix: "核对健康检查 URL 与端口、加大超时与重试、检查重启脚本是否真正拉起进程", guide: "cicd-health-rollback" },
    ErrorHint { pattern: "start job for unit already queued", cause: "systemd 重启请求排队", fix: "重启脚本里先 stop 再 start，或等待后重试健康检查", guide: "cicd-health-rollback" },
    ErrorHint { pattern: "please commit your changes or stash them", cause: "本地有未提交改动挡住了拉取", fix: "提交或还原本地改动；工具会尝试 stash，冲突时需人工处理", guide: "cicd-single-main-module" },
    ErrorHint { pattern: "not a git repository", cause: "本地目录不是 git 仓库（或代码在子目录）", fix: "用「选择目录」指向真实代码目录；纯本地目录会跳过分支切换", guide: "cicd-maven-parent-path" },
    ErrorHint { pattern: "couldn't find remote ref", cause: "部署分支在远端不存在", fix: "改对分支名（区分 release-dev / feat-xxx 等），或先推送该分支", guide: "cicd-multi-env" },
    ErrorHint { pattern: "rolled-back:partial", cause: "自动回滚只恢复了部分文件", fix: "手动核对线上文件版本，必要时用回滚功能重做一次", guide: "cicd-health-rollback" },
    ErrorHint { pattern: "未变更跳过", cause: "增量上传判定无需传输（可能 manifest 与线上不一致）", fix: "回滚/手工替换过文件后删掉远端 .deploy_manifest.json，或先关增量跑一次全量", guide: "cicd-incremental" },
];

fn score(guide: &Guide, query: &str) -> usize {
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

/// 检索教学内容（返回正文，助手直接引用）
pub fn search_guides(query: &str, limit: usize) -> Vec<Value> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let mut hits: Vec<(usize, &Guide)> = GUIDES
        .iter()
        .map(|g| (score(g, &q), g))
        .filter(|(s, _)| *s > 0)
        .collect();
    hits.sort_by(|a, b| b.0.cmp(&a.0));
    hits.into_iter()
        .take(limit)
        .map(|(s, g)| {
            json!({
                "id": g.id, "title": g.title, "module": g.module,
                "route": g.route, "relevance": s, "body": g.body,
            })
        })
        .collect()
}

/// 按报错文本匹配已知特征
pub fn match_error_hints(text: &str) -> Vec<Value> {
    let lower = text.to_lowercase();
    ERROR_HINTS
        .iter()
        .filter(|h| lower.contains(h.pattern))
        .map(|h| {
            json!({
                "matched": h.pattern, "cause": h.cause, "fix": h.fix, "seeGuide": h.guide,
            })
        })
        .collect()
}

/// 供系统提示词用的目录（只给标题，正文靠工具按需取，省 token）
pub fn guide_index() -> Value {
    Value::Array(
        GUIDES
            .iter()
            .map(|g| json!({ "id": g.id, "title": g.title, "module": g.module }))
            .collect(),
    )
}

pub fn guide_body(id: &str) -> Option<&'static Guide> {
    GUIDES.iter().find(|g| g.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_output_directory_pitfall() {
        let hits = search_guides("产物目录填什么收不到 jar", 3);
        assert!(!hits.is_empty());
        assert!(
            ["cicd-build-vs-output-dir", "cicd-single-main-module"]
                .contains(&hits[0]["id"].as_str().unwrap()),
            "首个命中应最相关: {:?}",
            hits
        );
        assert!(hits[0]["body"].as_str().unwrap().contains("构建目录"));
    }

    #[test]
    fn unrelated_query_returns_nothing() {
        assert!(search_guides("zzz完全无关的东西qqq", 5).is_empty());
        assert!(search_guides("  ", 5).is_empty());
    }

    #[test]
    fn error_signatures_map_to_actionable_fix() {
        let hints = match_error_hints(
            "[ERROR] The build could not read 1 project -> no POM in this directory /Users/x/app",
        );
        assert!(hints.iter().any(|h| h["seeGuide"] == "cicd-maven-parent-path"));
        assert!(match_error_hints("健康检查未通过: timeout").iter().any(|h| h["matched"] == "健康检查未通过"));
        assert!(match_error_hints("Permission denied (publickey)").iter().any(|h| h["fix"].as_str().unwrap().contains("凭据")));
        assert!(match_error_hints("一切正常").is_empty());
    }

    #[test]
    fn every_error_hint_points_at_a_real_guide() {
        for h in ERROR_HINTS {
            assert!(guide_body(h.guide).is_some(), "{} 指向了不存在的条目", h.pattern);
        }
        assert_eq!(guide_index().as_array().unwrap().len(), GUIDES.len());
    }
}
