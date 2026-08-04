pub fn print_guide() {
    println!(
        "\n  SuperTool CLI v{} 使用指南\n",
        env!("CARGO_PKG_VERSION")
    );
    println!("  📦 JSON 结构化输出：全局 --json 或各命令 -j（envelope: {{\"ok\":true,\"data\":...}}）");
    println!("  🤖 AI 接入：stool mcp serve 启动 MCP server（Claude Code/Cursor 直接调用）");
    println!("  📜 操作审计：写操作自动落库（参数脱敏），stool audit list 可查\n");
    println!("  任务管理:");
    println!("    stool todo add '任务' [-p high] [-d 2025-01-31] [-t 标签]");
    println!("    stool todo list [-j] [-l 10]       stool todo search '关键词' [-j]");
    println!("    stool todo complete <id>            stool todo stats [-j]");
    println!("  服务器管理:");
    println!("    stool server list [-j]              stool server exec <id> 'cmd'");
    println!("    stool server health <id> [-j]       stool server diagnose <id> [-j]");
    println!("    stool server read <id> <path>       stool server ls <id> [--path /dir]");
    println!("    stool server download <id> <remote> stool server mkdir <id> <path>");
    println!("    stool server java-ps <id> [-j]");
    println!("  CI/CD 部署:");
    println!("    stool cicd list [-j]                stool cicd deploy <id> [--stream|--watch]");
    println!("    stool cicd rollback <id> <ver>      stool cicd history <id> [--status success]");
    println!("  数据库:");
    println!("    stool db list [-j]                  stool db query -d <id> 'SQL' [-j]");
    println!("    stool db redis -d <id> keys '*'     stool db redis -d <id> ttl <key>");
    println!("  Git 管理:");
    println!("    stool git list [-j]                 stool git status --path <repo> [-j]");
    println!("    stool git log --path <repo>         stool git branches --path <repo>");
    println!("    stool git pull --path <repo>        stool git push --path <repo>");
    println!("  日志管理:");
    println!("    stool log list [-j]                 stool log search <id> 'keyword'");
    println!("    stool log tail <id> [-l 100]");
    println!("  MFA 管理:");
    println!("    stool mfa list [-j]                 stool mfa add '名称' <secret>");
    println!("    stool mfa code <id|序号>            stool mfa parse-uri <otpauth://...>");
    println!("  笔记管理:");
    println!("    stool note list [-j]                stool note add '标题' [--content 内容]");
    println!("    stool note update <id> [--title x]  stool note delete <id>");
    println!("    stool note groups [-j]              stool note add-group '名称'");
    println!("  记账管理:");
    println!(
        "    stool accounting list [-j]          stool accounting add <金额> --type 支出 --category 服务器"
    );
    println!("    stool accounting stats [-j]         stool accounting trend [-j]");
    println!("    stool accounting categories [-j]    stool accounting budgets [-j]");
    println!("  周报管理:");
    println!("    stool weekly list [-j]              stool weekly show <id>");
    println!("    stool weekly save '标题' --content '内容'");
    println!("  Nginx 管理:");
    println!("    stool nginx list [-j]               stool nginx add '名称'");
    println!(
        "    stool nginx fetch <server_id> <path> stool nginx deploy <server_id> <path> <content>"
    );
    println!("    stool nginx versions <preset_id>");
    println!("  数据备份:");
    println!(
        "    stool backup export [--output path]  stool backup import <file> [--mode merge|replace]"
    );
    println!("    stool backup export-csv");
    println!("  操作审计:");
    println!("    stool audit list [-j]               stool audit list --actor ai [--result failed]");
    println!("  MCP 接入:");
    println!("    stool mcp serve [--name stool]      stool mcp list-tools");
    println!("\n  直连 supertool-core 共享库，零 UDS/HTTP 依赖，完全独立运行");
}
