pub fn print_guide() {
    println!("\n  SuperTool CLI v{} 使用指南\n", env!("CARGO_PKG_VERSION"));
    println!("  任务管理:");
    println!("    stool todo add '任务' [-p high] [-d 2024-12-31] [-t 标签]");
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
    println!("  数据管理:");
    println!("    stool db list [-j]                  stool db query -d <id> 'SQL' [-j]");
    println!("    stool db redis -d <id> keys '*'     stool db redis -d <id> ttl <key>");
    println!("  Git 管理:");
    println!("    stool git list [-j]                 stool git status --path <repo> [-j]");
    println!("    stool git log --path <repo>         stool git branches --path <repo>");
    println!("    stool git pull --path <repo>        stool git push --path <repo>");
    println!("  日志管理:");
    println!("    stool log list [-j]                 stool log search <id> 'keyword'");
    println!("    stool log tail <id> [-l 100]");
    println!("\n  连接: --api-url URL | SUPERTOOL_API_URL=URL");
}
