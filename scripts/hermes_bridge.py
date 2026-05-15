#!/usr/bin/env python3
"""
Hermes Agent Bridge Script

 stdin/stdout JSON protocol for Tauri/Rust to call Hermes AIAgent.

 Protocol:
   Input (stdin JSON):
     {"action": "chat", "session_id": "...", "message": "...", "model": "...", "toolsets": [...]}
     {"action": "resume", "session_id": "..."}
     {"action": "list_sessions", "limit": 20}
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
    HERMES_AVAILABLE = True
except ImportError as e:
    HERMES_AVAILABLE = False
    _IMPORT_ERROR = str(e)

# Global state
_current_agent: Optional[AIAgent] = None
_current_session_id: Optional[str] = None
_abort_flag: bool = False
_session_db: Optional[SessionDB] = None


def _ensure_session_db() -> SessionDB:
    """Get or create SessionDB instance."""
    global _session_db
    if _session_db is None:
        _session_db = SessionDB()
    return _session_db


def _output(msg: Dict[str, Any]) -> None:
    """Write JSON message to stdout."""
    sys.stdout.write(json.dumps(msg, ensure_ascii=False) + "\n")
    sys.stdout.flush()


def _create_agent(
    session_id: Optional[str] = None,
    model: Optional[str] = None,
    toolsets: Optional[List[str]] = None,
    conversation_history: Optional[List[Dict]] = None,
) -> AIAgent:
    """Create an AIAgent instance."""
    global _current_agent, _current_session_id

    # Resolve toolsets
    enabled_toolsets = None
    if toolsets:
        enabled_toolsets = resolve_multiple_toolsets(toolsets)

    # Create callbacks for streaming
    def stream_callback(delta: str) -> None:
        if not _abort_flag:
            _output({"type": "delta", "text": delta})

    def tool_start_callback(tool_name: str, tool_args: Dict) -> None:
        if not _abort_flag:
            _output({"type": "tool_start", "name": tool_name, "args": tool_args})

    def tool_complete_callback(tool_name: str, result: Any, duration_ms: float) -> None:
        if not _abort_flag:
            # Truncate large results
            result_str = str(result)
            if len(result_str) > 5000:
                result_str = result_str[:5000] + "..."
            _output({
                "type": "tool_complete",
                "name": tool_name,
                "result": result_str,
                "duration_ms": round(duration_ms, 2)
            })

    def thinking_callback(text: str) -> None:
        if not _abort_flag:
            # Truncate long thinking blocks
            if len(text) > 2000:
                text = text[:2000] + "..."
            _output({"type": "thinking", "text": text})

    # Get session DB
    session_db = _ensure_session_db()

    # Generate session ID if not provided
    if not session_id:
        session_id = str(uuid.uuid4())

    # Create agent
    agent = AIAgent(
        model=model or "anthropic/claude-sonnet-4",
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

    # Load conversation history for resume
    if conversation_history:
        agent.conversation_history = conversation_history

    _current_agent = agent
    _current_session_id = session_id

    return agent


def _handle_chat(params: Dict[str, Any]) -> None:
    """Handle chat action."""
    global _abort_flag

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
            conversation_history=conversation_history,
        )

        # Run conversation
        result = agent.run_conversation(message)

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
            # last_active: use ended_at if available, else started_at
            last_active = ended_at or started_at
            
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
            formatted.append({
                "role": m.get("role", ""),
                "content": m.get("content", "")[:1000] if m.get("content") else None,
                "timestamp": m.get("timestamp"),
                "tool_name": m.get("tool_name"),
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


def _handle_abort(params: Dict[str, Any]) -> None:
    """Handle abort action."""
    global _abort_flag, _current_agent

    _abort_flag = True

    if _current_agent:
        _current_agent.interrupt("User aborted")

    _output({"type": "aborted", "session_id": _current_session_id})


def _handle_command(cmd: Dict[str, Any]) -> None:
    """Process a single command."""
    action = cmd.get("action")

    handlers = {
        "chat": _handle_chat,
        "list_sessions": _handle_list_sessions,
        "get_session": _handle_get_session,
        "delete_session": _handle_delete_session,
        "abort": _handle_abort,
    }

    handler = handlers.get(action)
    if handler:
        handler(cmd)
    else:
        _output({"type": "error", "message": f"Unknown action: {action}"})


def _signal_handler(signum, frame):
    """Handle interrupt signals."""
    global _abort_flag, _current_agent
    _abort_flag = True
    if _current_agent:
        _current_agent.interrupt("Signal received")
    _output({"type": "aborted", "session_id": _current_session_id})
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