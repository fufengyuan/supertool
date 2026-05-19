#!/usr/bin/env python3
"""
单元测试：验证 IPC 打断保存消息的逻辑

测试场景：
1. 启动会话并执行 tool 调用
2. 在 tool 调用过程中模拟 SIGTERM
3. 验证消息是否被正确保存到数据库

运行方式：
python3 tests/test_ipc_interrupt.py
"""

import json
import os
import signal
import sqlite3
import subprocess
import sys
import time
import uuid
from pathlib import Path

# 项目根目录
PROJECT_ROOT = Path(__file__).parent.parent
BRIDGE_SCRIPT = PROJECT_ROOT / "scripts" / "hermes_bridge.py"
DB_PATH = Path.home() / ".hermes" / "state.db"


class IPCTestRunner:
    """运行 hermes_bridge.py 作为子进程，模拟 IPC 调用"""

    def __init__(self):
        self.process = None
        self.stdout_lines = []
        self.stderr_lines = []

    def start(self):
        """启动 bridge 进程"""
        self.process = subprocess.Popen(
            [sys.executable, str(BRIDGE_SCRIPT)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,  # 行缓冲
        )
        print(f"[INFO] Bridge process started (PID: {self.process.pid})")

    def send_command(self, cmd: dict) -> dict:
        """发送命令并等待响应"""
        cmd_line = json.dumps(cmd) + "\n"
        self.process.stdin.write(cmd_line)
        self.process.stdin.flush()

        # 等待响应（可能有多行，最后一行是完整响应）
        response_lines = []
        while True:
            line = self.process.stdout.readline()
            if not line:
                break
            response_lines.append(line.strip())
            # 检查是否是最终响应（type: done, error, aborted）
            try:
                data = json.loads(line.strip())
                if data.get("type") in ("done", "error", "aborted"):
                    return data
            except json.JSONDecodeError:
                continue

        # 如果没有收到完整响应，返回最后一行
        if response_lines:
            try:
                return json.loads(response_lines[-1])
            except json.JSONDecodeError:
                return {"type": "unknown", "raw": response_lines[-1]}
        return {"type": "timeout"}

    def interrupt(self):
        """发送 SIGTERM 打断"""
        if self.process:
            print(f"[INFO] Sending SIGTERM to PID {self.process.pid}")
            self.process.send_signal(signal.SIGTERM)

    def wait(self, timeout=5):
        """等待进程结束"""
        if self.process:
            try:
                stdout, stderr = self.process.communicate(timeout=timeout)
                print(f"[INFO] Process exited with code {self.process.returncode}")
                return stdout, stderr
            except subprocess.TimeoutExpired:
                print("[WARN] Process did not exit, killing...")
                self.process.kill()
                return self.process.communicate()

    def close(self):
        """关闭进程"""
        if self.process and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                self.process.kill()


def test_basic_chat():
    """测试基本对话（无打断）"""
    print("\n=== Test 1: Basic Chat (No Interrupt) ===")

    runner = IPCTestRunner()
    runner.start()

    # 发送 chat 命令
    session_id = f"test_{uuid.uuid4().hex[:16]}"
    
    # 使用 glm-5 模型（阿里云可用）
    response = runner.send_command({
        "action": "chat",
        "message": "What is the capital of France? Just answer with one word.",
        "session_id": session_id,
        "model": "glm-5",
    })

    print(f"[RESPONSE] type={response.get('type')}, session_id={response.get('session_id')}")

    # 验证数据库中有消息
    conn = sqlite3.connect(str(DB_PATH))
    messages = conn.execute(
        "SELECT role, content FROM messages WHERE session_id = ?",
        (session_id,)
    ).fetchall()
    print(f"[DB] {len(messages)} messages saved:")
    for m in messages:
        content_preview = (m[1][:50] + "...") if m[1] and len(m[1]) > 50 else m[1] or "(null)"
        print(f"  [{m[0]}] {content_preview}")

    runner.close()

    assert len(messages) >= 2, "Should have at least user and assistant messages"
    print("[PASS] Basic chat test")


def test_interrupt_simple():
    """测试打断简单对话"""
    print("\n=== Test 2: Interrupt Simple Chat ===")

    runner = IPCTestRunner()
    runner.start()

    session_id = f"test_interrupt_{uuid.uuid4().hex[:16]}"

    # 发送一个需要 longer 响应的命令
    runner.process.stdin.write(json.dumps({
        "action": "chat",
        "message": "Count from 1 to 100, one number per line",
        "session_id": session_id,
        "model": "glm-5",
    }) + "\n")
    runner.process.stdin.flush()

    # 等待足够时间让 _handle_chat 开始执行（至少 2 秒）
    time.sleep(2)

    # 发送 SIGTERM 打断
    runner.interrupt()

    # 等待进程结束
    stdout, stderr = runner.wait(timeout=5)

    # 检查 stderr 中是否有保存消息的日志
    if "[INFO] Session" in stderr and "persisted on signal" in stderr:
        print("[INFO] Signal handler saved messages")
    else:
        print("[WARN] Signal handler did not save messages")
        print(f"[STDERR] {stderr[:200] if stderr else '(empty)'}")

    # 验证数据库
    conn = sqlite3.connect(str(DB_PATH))
    messages = conn.execute(
        "SELECT role, content FROM messages WHERE session_id = ?",
        (session_id,)
    ).fetchall()
    print(f"[DB] {len(messages)} messages saved:")
    for m in messages:
        content_preview = (m[1][:50] + "...") if m[1] and len(m[1]) > 50 else m[1] or "(null)"
        print(f"  [{m[0]}] {content_preview}")

    runner.close()

    # 至少应该有用户消息
    assert len(messages) >= 1, "Should have at least user message after interrupt"
    print("[PASS] Interrupt test")


def test_interrupt_with_tools():
    """测试打断 tool 调用过程"""
    print("\n=== Test 3: Interrupt During Tool Calls ===")

    runner = IPCTestRunner()
    runner.start()

    session_id = f"test_tool_interrupt_{uuid.uuid4().hex[:16]}"

    # 发送一个需要 tool 调用的命令
    runner.process.stdin.write(json.dumps({
        "action": "chat",
        "message": "Search the web for 'current weather in Beijing' and summarize",
        "session_id": session_id,
        "model": "glm-5",
        "toolsets": ["web"],
    }) + "\n")
    runner.process.stdin.flush()

    # 等待足够时间让 tool 调用开始（至少 3 秒）
    time.sleep(3)

    # 发送 SIGTERM 打断
    runner.interrupt()

    # 等待进程结束
    stdout, stderr = runner.wait(timeout=10)

    # 检查 stderr（完整）
    print(f"[STDERR] {stderr if stderr else '(empty)'}")

    # 验证数据库
    conn = sqlite3.connect(str(DB_PATH))
    messages = conn.execute(
        "SELECT role, content, tool_calls, tool_name FROM messages WHERE session_id = ?",
        (session_id,)
    ).fetchall()
    print(f"[DB] {len(messages)} messages saved:")
    for m in messages:
        role, content, tool_calls, tool_name = m
        content_preview = (content[:30] + "...") if content and len(content) > 30 else content or "(null)"
        print(f"  [{role}] {content_preview}")
        if tool_calls:
            print(f"    tool_calls: {tool_calls[:80]}...")
        if tool_name:
            print(f"    tool_name: {tool_name}")

    runner.close()

    # 关键验证：应该保存用户消息
    has_user = any(m[0] == "user" for m in messages)
    # assistant 消息可能不存在（tool 调用还没完成）

    print(f"[CHECK] has_user={has_user}")

    if not has_user:
        print("[FAIL] User message not saved!")

    assert has_user, "User message should be saved"
    print("[PASS] Interrupt with tools test - user message saved")


def main():
    print("=" * 60)
    print("IPC Interrupt Test Suite")
    print("=" * 60)

    # 检查 Hermes 是否可用（从 ~/.hermes/hermes-agent）
    hermes_path = Path.home() / ".hermes" / "hermes-agent"
    if hermes_path.exists():
        sys.path.insert(0, str(hermes_path))
        print(f"[INFO] Added Hermes path: {hermes_path}")
    try:
        from run_agent import AIAgent
        print("[INFO] Hermes agent available")
    except ImportError as e:
        print(f"[ERROR] Hermes not available: {e}")
        print("[SKIP] Tests require Hermes agent")
        return

    # 检查数据库
    if not DB_PATH.exists():
        print(f"[ERROR] Database not found: {DB_PATH}")
        return
    print(f"[INFO] Database: {DB_PATH}")

    # 运行测试
    try:
        test_basic_chat()
    except Exception as e:
        print(f"[ERROR] Test 1 failed: {e}")

    try:
        test_interrupt_simple()
    except Exception as e:
        print(f"[ERROR] Test 2 failed: {e}")

    try:
        test_interrupt_with_tools()
    except Exception as e:
        print(f"[ERROR] Test 3 failed: {e}")

    print("\n" + "=" * 60)
    print("Test Suite Complete")
    print("=" * 60)


if __name__ == "__main__":
    main()