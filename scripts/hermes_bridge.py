#!/usr/bin/env python3
"""
Hermes Agent Bridge Script

 stdin/stdout JSON protocol for Tauri/Rust to call Hermes AIAgent.

 Protocol:
   Input (stdin JSON):
     {"action": "chat", "session_id": "...", "message": "...", "model": "...", "toolsets": [...]}
     {"action": "resume", "session_id": "..."}
     {"action": "list_sessions", "limit": 20}
     {"action": "search_sessions", "query": "...", "limit": 20, "offset": 0}
     {"action": "get_session", "session_id": "..."}
     {"action": "delete_session", "session_id": "..."}
     {"action": "abort"}

   Output (stdout JSON, line-by-line):
     {"type": "delta", "text": "..."}        # Streaming text chunk
     {"type": "tool_start", "name": "...", "args": {...}}
     {"type": "tool_complete", "name": "...", "result": "...", "duration_ms": ...}
     {"type": "thinking", "text": "..."}     # Reasoning/thinking block
     {"type": "done", "response": "...", "session_id": "...", "message_count": ...}
     {"type": "error", "message": "..."}
     {"type": "sessions", "data": [...]}     # For list_sessions
     {"type": "search_results", "data": [...], "query": "..."}  # For search_sessions
     {"type": "session", "data": {...}}      # For get_session
     {"type": "deleted", "session_id": "..."} # For delete_session
     {"type": "aborted", "session_id": "..."}

 Usage:
   python hermes_bridge.py
   # Then send JSON commands via stdin, receive JSON responses via stdout
"""

import json
import sys
import os
import signal
import threading
import time
import uuid
from pathlib import Path
from typing import Optional, Dict, Any, List, Callable

# Ensure Hermes is importable
_hermes_home = Path.home() / ".hermes"
_hermes_agent_path = _hermes_home / "hermes-agent"
if _hermes_agent_path.exists():
    sys.path.insert(0, str(_hermes_agent_path))

# Also try installed location
try:
    import hermes_constants
    _installed_path = Path(hermes_constants.__file__).parent
    if str(_installed_path) not in sys.path:
        sys.path.insert(0, str(_installed_path))
except ImportError:
    pass

# Load Hermes environment
try:
    from hermes_cli.env_loader import load_hermes_dotenv
    load_hermes_dotenv(hermes_home=_hermes_home)
except ImportError:
    # Fallback: load .env manually
    env_file = _hermes_home / ".env"
    if env_file.exists():
        with open(env_file) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, val = line.split("=", 1)
                    os.environ[key.strip()] = val.strip()

# Now import Hermes modules
try:
    from run_agent import AIAgent
    from hermes_state import SessionDB, get_hermes_home
    from model_tools import get_tool_definitions
    from toolsets import resolve_multiple_toolsets
    from cli import load_cli_config
    HERMES_AVAILABLE = True
    
    # 加载配置获取默认模型
    _CLI_CONFIG = load_cli_config()
    _MODEL_CONFIG = _CLI_CONFIG.get("model", {})
    if isinstance(_MODEL_CONFIG, dict):
        _DEFAULT_MODEL = _MODEL_CONFIG.get("default") or _MODEL_CONFIG.get("model") or ""
    else:
        _DEFAULT_MODEL = _MODEL_CONFIG or ""
except ImportError as e:
    HERMES_AVAILABLE = False
    _IMPORT_ERROR = str(e)
    _DEFAULT_MODEL = ""
    
    # 检查是否缺少 fire 模块，给出安装指引
    if "fire" in str(e):
        _IMPORT_ERROR = "Missing 'fire' dependency. Install Hermes Agent with: pip install -e ~/.hermes/hermes-agent '[all]' --break-system-packages"

# Global state
_current_agent: Optional[AIAgent] = None
_current_session_id: Optional[str] = None
_abort_flag: bool = False
_session_db: Optional[SessionDB] = None
# Track accumulated messages for signal handler (updated by stream callbacks)
_accumulated_messages: List[Dict[str, Any]] = []
_user_message: Optional[str] = None  # Store the user message for signal handler


def _ensure_session_db() -> SessionDB:
    """Get or create SessionDB instance."""
    global _session_db
    if _session_db is None:
        _session_db = SessionDB()
    return _session_db


def _output(msg: Dict[str, Any]) -> None:
    """Write JSON message to stdout."""
    global _current_session_id
    # 所有事件添加 session_id（如果已知）
    if _current_session_id is not None and "session_id" not in msg:
        msg["session_id"] = _current_session_id
    sys.stdout.write(json.dumps(msg, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def _create_agent(
    session_id: Optional[str] = None,
    model: Optional[str] = None,
    toolsets: Optional[List[str]] = None,
) -> AIAgent:
    """Create an AIAgent instance."""
    global _current_agent, _current_session_id, _accumulated_messages

    # Resolve toolsets
    enabled_toolsets = None
    if toolsets:
        enabled_toolsets = resolve_multiple_toolsets(toolsets)

    # Accumulated assistant text for signal handler
    _accumulated_assistant_text: List[str] = []

    # Create callbacks for streaming
    def stream_callback(delta: str) -> None:
        if not _abort_flag:
            # Accumulate assistant text for signal handler
            _accumulated_assistant_text.append(delta)
            _output({"type": "delta", "text": delta})

    def tool_start_callback(tool_call_id: str, tool_name: str, tool_args: Dict) -> None:
        if not _abort_flag:
            _output({"type": "tool_start", "id": tool_call_id, "name": tool_name, "args": tool_args})

    def tool_complete_callback(tool_call_id: str, tool_name: str, tool_args: Dict, result: Any) -> None:
        if not _abort_flag:
            # Calculate duration (approximate, since we don't have start time here)
            # Truncate large results
            result_str = str(result)
            if len(result_str) > 5000:
                result_str = result_str[:5000] + "..."
            _output({
                "type": "tool_complete",
                "id": tool_call_id,
                "name": tool_name,
                "result": result_str,
                "duration_ms": 0  # Duration not provided by Hermes callback
            })

    def thinking_callback(text: str) -> None:
        if not _abort_flag:
            # Truncate long thinking blocks with truncation marker
            truncated = False
            if len(text) > 2000:
                text = text[:2000] + "\n...[思考内容过长，已截断]"
                truncated = True
            _output({"type": "thinking", "text": text, "truncated": truncated})

    # Get session DB
    session_db = _ensure_session_db()

    # Generate session ID if not provided
    if not session_id:
        session_id = str(uuid.uuid4())

    # Initialize accumulated messages with user message for signal handler
    # The user message will be added by run_conversation, but we need it
    # in the signal handler before run_conversation completes
    global _user_message
    if _user_message:
        # Track user message for signal handler
        _accumulated_messages = [{"role": "user", "content": _user_message}]

    # CRITICAL: Set _current_session_id BEFORE creating AIAgent
    # If signal is received during AIAgent creation, we need session_id available
    _current_session_id = session_id

    # Create agent
    agent = AIAgent(
        model=model or _DEFAULT_MODEL,
        session_id=session_id,
        session_db=session_db,
        enabled_toolsets=enabled_toolsets,
        max_iterations=50,
        stream_delta_callback=stream_callback,
        tool_start_callback=tool_start_callback,
        tool_complete_callback=tool_complete_callback,
        thinking_callback=thinking_callback,
        platform="supertool",
        quiet_mode=True,
    )

    # Store accumulated assistant text reference on agent for signal handler
    agent._bridge_accumulated_text = _accumulated_assistant_text

    _current_agent = agent
    # Update session_id from agent if it was None (agent may have created new session)
    if session_id is None:
        _current_session_id = getattr(agent, "session_id", None)

    return agent


def _handle_chat(params: Dict[str, Any]) -> None:
    """Handle chat action."""
    global _abort_flag, _accumulated_messages, _user_message

    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}"})
        return

    message = params.get("message", "")
    if not message:
        _output({"type": "error", "message": "Missing 'message' field"})
        return

    session_id = params.get("session_id")
    model = params.get("model")
    toolsets = params.get("toolsets")

    # Reset accumulated messages for this turn
    _accumulated_messages = []
    _user_message = message

    # Resume existing session or create new
    conversation_history = None
    if session_id:
        session_db = _ensure_session_db()
        try:
            # Resolve short ID to full ID
            resolved_id = session_db.resolve_resume_session_id(session_id)
            if resolved_id:
                session_id = resolved_id
            # Load history
            conversation_history = session_db.get_messages_as_conversation(session_id)
        except Exception as e:
            _output({"type": "error", "message": f"Failed to load session: {e}"})
            return

    try:
        _abort_flag = False
        agent = _create_agent(
            session_id=session_id,
            model=model,
            toolsets=toolsets,
        )

        # Run conversation with history
        result = agent.run_conversation(message, conversation_history=conversation_history)

        # Clear accumulated messages after successful completion
        _accumulated_messages = []
        _user_message = None

        if _abort_flag:
            _output({"type": "aborted", "session_id": _current_session_id})
        else:
            final_response = result.get("final_response", "")
            # AIAgent uses _session_messages internally
            session_messages = getattr(agent, "_session_messages", [])
            message_count = len(session_messages) if session_messages else 0

            _output({
                "type": "done",
                "response": final_response,
                "session_id": _current_session_id,
                "message_count": message_count,
            })

    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_list_sessions(params: Dict[str, Any]) -> None:
    """Handle list_sessions action."""
    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}"})
        return

    limit = params.get("limit", 20)

    try:
        session_db = _ensure_session_db()
        sessions = session_db.list_sessions_rich(limit=limit)

        # Format sessions
        formatted = []
        for s in sessions:
            started_at = s.get("started_at")
            ended_at = s.get("ended_at")
            
            # last_active: use ended_at if available, else latest message timestamp, else started_at
            last_active = ended_at
            if not last_active:
                # 查询该 session 最新一条消息的时间
                try:
                    messages = session_db.get_messages(s.get("id"))
                    if messages:
                        last_active = messages[-1].get("timestamp")
                except Exception:
                    pass
            if not last_active:
                last_active = started_at
            
            formatted.append({
                "id": s.get("id", ""),
                "title": s.get("title"),
                "model": s.get("model", ""),
                "source": s.get("source", ""),
                "started_at": started_at,
                "ended_at": ended_at,
                "message_count": s.get("message_count", 0),
                "preview": s.get("preview", "")[:200],
                "last_active": last_active,
            })

        _output({"type": "sessions", "data": formatted, "total": len(formatted)})

    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_search_sessions(params: Dict[str, Any]) -> None:
    """Handle search_sessions action - search across all session content."""
    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}"})
        return

    query = params.get("query", "")
    if not query.strip():
        _output({"type": "error", "message": "Missing 'query' field"})
        return

    limit = params.get("limit", 20)
    offset = params.get("offset", 0)

    try:
        session_db = _ensure_session_db()
        matches = session_db.search_messages(
            query=query,
            limit=limit,
            offset=offset,
        )

        # Format results with session info
        formatted = []
        for m in matches:
            formatted.append({
                "session_id": m.get("session_id", ""),
                "session_title": m.get("session_title", ""),
                "message_id": m.get("id", ""),
                "role": m.get("role", ""),
                "snippet": m.get("snippet", ""),
                "content": m.get("content", ""),
                "timestamp": m.get("timestamp"),
                "source": m.get("source", ""),
                "model": m.get("model", ""),
            })

        _output({"type": "search_results", "data": formatted, "total": len(formatted), "query": query})

    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_get_session(params: Dict[str, Any]) -> None:
    """Handle get_session action."""
    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}"})
        return

    session_id = params.get("session_id")
    if not session_id:
        _output({"type": "error", "message": "Missing 'session_id' field"})
        return

    try:
        session_db = _ensure_session_db()
        # Resolve short ID
        resolved_id = session_db.resolve_resume_session_id(session_id)
        if resolved_id:
            session_id = resolved_id

        messages = session_db.get_messages(session_id)

        # Format messages
        formatted = []
        for m in messages:
            content = m.get("content")
            # 不截断历史消息内容，用户可能需要完整上下文
            formatted.append({
                "role": m.get("role", ""),
                "content": content,
                "timestamp": m.get("timestamp"),
                "tool_name": m.get("name"),  # 工具名称（tool 消息的 name 字段）
                "tool_call_id": m.get("tool_call_id"),  # 工具调用 ID
                "tool_calls": m.get("tool_calls"),  # assistant 消息的 tool_calls
                "thinking": m.get("reasoning") or m.get("reasoning_content"),  # 思考内容
            })

        _output({"type": "session", "session_id": session_id, "messages": formatted})

    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_delete_session(params: Dict[str, Any]) -> None:
    """Handle delete_session action."""
    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}"})
        return

    session_id = params.get("session_id")
    if not session_id:
        _output({"type": "error", "message": "Missing 'session_id' field"})
        return

    try:
        session_db = _ensure_session_db()
        # Resolve short ID
        resolved_id = session_db.resolve_resume_session_id(session_id)
        if resolved_id:
            session_id = resolved_id

        session_db.delete_session(session_id)
        _output({"type": "deleted", "session_id": session_id})

    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_rename_session(params: Dict[str, Any]) -> None:
    """Handle rename_session action."""
    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}"})
        return

    session_id = params.get("session_id")
    title = params.get("title")
    if not session_id or not title:
        _output({"type": "error", "message": "Missing 'session_id' or 'title' field"})
        return

    try:
        session_db = _ensure_session_db()
        # Resolve short ID
        resolved_id = session_db.resolve_resume_session_id(session_id)
        if resolved_id:
            session_id = resolved_id

        session_db.set_session_title(session_id, title)
        _output({"type": "renamed", "session_id": session_id, "title": title})

    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_abort(params: Dict[str, Any]) -> None:
    """Handle abort action."""
    global _abort_flag, _current_agent

    _abort_flag = True

    if _current_agent:
        _current_agent.interrupt("User aborted")

    _output({"type": "aborted", "session_id": _current_session_id})


def _handle_get_models(params: Dict[str, Any]) -> None:
    """Handle get_models action - read custom models from Hermes config."""
    try:
        # 重新加载配置（可能有外部修改）
        config = load_cli_config()
        
        # 从配置读取用户自定义模型列表
        custom_models = config.get("custom_models", [])
        
        # 同时返回默认模型
        default_model = ""
        model_config = config.get("model", {})
        if isinstance(model_config, dict):
            default_model = model_config.get("default") or model_config.get("model") or ""
        
        _output({
            "type": "models",
            "custom_models": custom_models,
            "default_model": default_model,
        })
    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_add_model(params: Dict[str, Any]) -> None:
    """Handle add_model action - add a model to Hermes config."""
    model = params.get("model")
    if not model or not model.strip():
        _output({"type": "error", "message": "Missing 'model' field"})
        return
    
    model = model.strip()
    
    try:
        config = load_cli_config()
        
        # 获取现有自定义模型列表
        custom_models = config.get("custom_models", [])
        
        # 检查是否已存在
        if model in custom_models:
            _output({"type": "error", "message": f"Model '{model}' already exists"})
            return
        
        # 添加新模型
        custom_models.append(model)
        config["custom_models"] = custom_models
        
        # 写回配置文件
        import yaml
        config_path = _hermes_home / "config.yaml"
        with open(config_path, "w") as f:
            yaml.dump(config, f, default_flow_style=False, allow_unicode=True, sort_keys=False)
        
        _output({"type": "model_added", "model": model, "custom_models": custom_models})
    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_remove_model(params: Dict[str, Any]) -> None:
    """Handle remove_model action - remove a model from Hermes config."""
    model = params.get("model")
    if not model or not model.strip():
        _output({"type": "error", "message": "Missing 'model' field"})
        return
    
    model = model.strip()
    
    try:
        config = load_cli_config()
        
        # 获取现有自定义模型列表
        custom_models = config.get("custom_models", [])
        
        # 检查是否存在
        if model not in custom_models:
            _output({"type": "error", "message": f"Model '{model}' not found"})
            return
        
        # 移除模型
        custom_models.remove(model)
        config["custom_models"] = custom_models
        
        # 写回配置文件
        import yaml
        config_path = _hermes_home / "config.yaml"
        with open(config_path, "w") as f:
            yaml.dump(config, f, default_flow_style=False, allow_unicode=True, sort_keys=False)
        
        _output({"type": "model_removed", "model": model, "custom_models": custom_models})
    except Exception as e:
        _output({"type": "error", "message": str(e)})


def _handle_command(cmd: Dict[str, Any]) -> None:
    """Process a single command."""
    action = cmd.get("action")

    handlers = {
        "chat": _handle_chat,
        "list_sessions": _handle_list_sessions,
        "search_sessions": _handle_search_sessions,
        "get_session": _handle_get_session,
        "delete_session": _handle_delete_session,
        "rename_session": _handle_rename_session,
        "abort": _handle_abort,
        "get_models": _handle_get_models,
        "add_model": _handle_add_model,
        "remove_model": _handle_remove_model,
    }

    handler = handlers.get(action)
    if handler:
        handler(cmd)
    else:
        _output({"type": "error", "message": f"Unknown action: {action}"})


def _signal_handler(signum, frame):
    """Handle interrupt signals - persist session before exit."""
    global _abort_flag, _current_agent, _current_session_id, _accumulated_messages

    _abort_flag = True

    # Immediately persist session to prevent message loss on forced termination
    # This is critical because SIGTERM may be followed by SIGKILL after timeout
    if _current_session_id:
        try:
            session_db = _ensure_session_db()

            # Get accumulated assistant text from agent (if available)
            assistant_text = ""
            if _current_agent:
                accumulated_list = getattr(_current_agent, "_bridge_accumulated_text", [])
                if accumulated_list:
                    assistant_text = "".join(accumulated_list)

            # Build messages to save:
            # 1. User message from global state
            # 2. Accumulated assistant text (if any)
            messages_to_save = list(_accumulated_messages)  # Start with user message

            if assistant_text:
                messages_to_save.append({
                    "role": "assistant",
                    "content": assistant_text,
                })

            # Also try to get messages from Hermes agent's internal state
            if _current_agent:
                agent_messages = getattr(_current_agent, "_session_messages", [])
                if agent_messages:
                    # Prefer agent's internal messages if available
                    messages_to_save = agent_messages

            # Ensure session row exists
            session_db.ensure_session(
                _current_session_id,
                source="supertool",
                model=_current_agent.model if _current_agent else "",
            )

            # Save messages
            if messages_to_save:
                for msg in messages_to_save:
                    role = msg.get("role", "unknown")
                    content = msg.get("content")
                    tool_calls_data = None
                    if isinstance(msg.get("tool_calls"), list):
                        tool_calls_data = msg["tool_calls"]
                    session_db.append_message(
                        session_id=_current_session_id,
                        role=role,
                        content=content,
                        tool_name=msg.get("tool_name"),
                        tool_calls=tool_calls_data,
                        tool_call_id=msg.get("tool_call_id"),
                        finish_reason=msg.get("finish_reason"),
                        reasoning=msg.get("reasoning") if role == "assistant" else None,
                    )

                # Force flush to disk
                if hasattr(session_db, "_conn"):
                    session_db._conn.commit()

                sys.stderr.write(f"[INFO] Session {_current_session_id} persisted on signal ({len(messages_to_save)} messages)\n")

        except Exception as e:
            sys.stderr.write(f"[WARN] Failed to persist session on signal: {e}\n")

    # Also call interrupt() and persist via Hermes internal method as fallback
    if _current_agent:
        _current_agent.interrupt("Signal received")
        try:
            if getattr(_current_agent, '_session_messages', None):
                _current_agent._persist_session(_current_agent._session_messages)
        except Exception:
            pass

    # Send aborted event (use stderr to avoid stdout JSON parsing issues during signal)
    sys.stderr.write(json.dumps({"type": "aborted", "session_id": _current_session_id}) + "\n")
    sys.stderr.flush()

    sys.exit(0)


def main():
    """Main entry point - read commands from stdin."""
    # Set up signal handlers
    signal.signal(signal.SIGINT, _signal_handler)
    signal.signal(signal.SIGTERM, _signal_handler)

    # Check Hermes availability
    if not HERMES_AVAILABLE:
        _output({"type": "error", "message": f"Hermes not available: {_IMPORT_ERROR}. Please install Hermes first."})
        # Still continue to handle other non-agent commands

    # Read commands from stdin line by line
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue

        try:
            cmd = json.loads(line)
            _handle_command(cmd)
        except json.JSONDecodeError as e:
            _output({"type": "error", "message": f"Invalid JSON: {e}"})


if __name__ == "__main__":
    main()