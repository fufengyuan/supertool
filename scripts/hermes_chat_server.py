#!/usr/bin/env python3
"""
Hermes Chat HTTP Server

FastAPI-based HTTP server that wraps AIAgent.run_conversation() with SSE streaming.
Replaces the stdin/stdout JSON bridge for SuperTool agent chat.

Endpoints:
  GET  /v1/health           → Health check
  POST /v1/chat             → SSE streaming chat
  POST /v1/abort            → Abort running conversation

Usage:
  python scripts/hermes_chat_server.py --port 18686
"""

import asyncio
import json
import os
import signal
import sys
import threading
import uuid
from pathlib import Path
from typing import Optional, Dict, Any, List

# ---------------------------------------------------------------------------
# Hermes import setup (same logic as hermes_bridge.py)
# ---------------------------------------------------------------------------
_hermes_home = Path.home() / ".hermes"
_hermes_agent_path = _hermes_home / "hermes-agent"
if _hermes_agent_path.exists():
    sys.path.insert(0, str(_hermes_agent_path))

try:
    from hermes_cli.env_loader import load_hermes_dotenv
    load_hermes_dotenv(hermes_home=_hermes_home)
except ImportError:
    env_file = _hermes_home / ".env"
    if env_file.exists():
        with open(env_file) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, val = line.split("=", 1)
                    os.environ[key.strip()] = val.strip()

from run_agent import AIAgent
from hermes_state import SessionDB, get_hermes_home
from model_tools import get_tool_definitions
from toolsets import resolve_multiple_toolsets
from cli import load_cli_config

# ---------------------------------------------------------------------------
# Global state
# ---------------------------------------------------------------------------
_session_db: Optional[SessionDB] = None
_running_agents: Dict[str, Any] = {}       # session_id → AIAgent
_running_agents_lock = threading.Lock()
_abort_events: Dict[str, threading.Event] = {}  # session_id → Event

_cli_config = load_cli_config()
_model_config = _cli_config.get("model", {})
if isinstance(_model_config, dict):
    _default_model = _model_config.get("default") or _model_config.get("model") or ""
else:
    _default_model = _model_config or ""


def _ensure_session_db() -> SessionDB:
    global _session_db
    if _session_db is None:
        _session_db = SessionDB()
    return _session_db


# ---------------------------------------------------------------------------
# Chat agent runner (thread-based, to work with FastAPI's async)
# ---------------------------------------------------------------------------
def _run_chat_in_thread(
    session_id: str,
    message: str,
    model: Optional[str],
    toolsets: Optional[List[str]],
    event_queue: asyncio.Queue,
):
    """Run AIAgent conversation in a background thread, pushing events to asyncio queue."""
    abort_event = _abort_events.get(session_id)
    if abort_event is None:
        abort_event = threading.Event()
        _abort_events[session_id] = abort_event
    abort_event.clear()

    enabled_toolsets = None
    if toolsets:
        enabled_toolsets = resolve_multiple_toolsets(toolsets)

    try:
        session_db = _ensure_session_db()
        if not session_id:
            session_id = str(uuid.uuid4())

        # Load conversation history
        conversation_history = None
        try:
            resolved_id = session_db.resolve_resume_session_id(session_id)
            if resolved_id:
                session_id = resolved_id
            conversation_history = session_db.get_messages_as_conversation(session_id)
        except Exception:
            pass

        # Create callbacks
        def _put_event(etype: str, data: dict):
            """Put event onto queue, checking abort between iterations."""
            data["type"] = etype
            data["session_id"] = session_id
            # Use call_soon_threadsafe to put from thread into async loop
            asyncio.run_coroutine_threadsafe(
                event_queue.put(data), _main_loop
            )

        def stream_callback(delta: str):
            if not abort_event.is_set():
                _put_event("delta", {"text": delta})

        def tool_start_callback(tool_call_id: str, tool_name: str, tool_args: Dict):
            if not abort_event.is_set():
                _put_event("tool_start", {
                    "id": tool_call_id, "name": tool_name, "args": tool_args
                })

        def tool_complete_callback(tool_call_id: str, tool_name: str, tool_args: Dict, result: Any):
            if not abort_event.is_set():
                result_str = str(result)
                if len(result_str) > 5000:
                    result_str = result_str[:5000] + "..."
                _put_event("tool_complete", {
                    "id": tool_call_id, "name": tool_name, "result": result_str, "duration_ms": 0
                })

        def thinking_callback(text: str):
            if not abort_event.is_set():
                if len(text) > 2000:
                    text = text[:2000] + "\n...[truncated]"
                _put_event("thinking", {"text": text})

        # Create agent
        agent = AIAgent(
            model=model or _default_model,
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

        with _running_agents_lock:
            _running_agents[session_id] = agent

        # Run conversation
        result = agent.run_conversation(message, conversation_history=conversation_history)

        with _running_agents_lock:
            _running_agents.pop(session_id, None)

        if abort_event.is_set():
            _put_event("aborted", {})
        else:
            final_response = result.get("final_response", "")
            session_messages = getattr(agent, "_session_messages", [])
            message_count = len(session_messages) if session_messages else 0
            _put_event("done", {
                "response": final_response,
                "message_count": message_count,
            })

    except Exception as e:
        _put_event("error", {"message": str(e)})
    finally:
        _abort_events.pop(session_id, None)
        # Signal end of stream
        asyncio.run_coroutine_threadsafe(event_queue.put(None), _main_loop)


# Global reference to main event loop (set at startup)
_main_loop: Optional[asyncio.AbstractEventLoop] = None


# ---------------------------------------------------------------------------
# FastAPI app
# ---------------------------------------------------------------------------
from fastapi import FastAPI, Request
from fastapi.responses import StreamingResponse, JSONResponse
from pydantic import BaseModel

app = FastAPI(title="Hermes Chat Server", version="1.0.0")


class ChatRequest(BaseModel):
    message: str
    session_id: Optional[str] = None
    model: Optional[str] = None
    toolsets: Optional[List[str]] = None


class AbortRequest(BaseModel):
    session_id: str


@app.get("/v1/health")
async def health():
    return {"status": "ok", "version": "1.0.0"}


async def _chat_event_stream(event_queue: asyncio.Queue):
    """Newline-delimited JSON stream (same format as old bridge, simpler to parse)."""
    while True:
        event = await event_queue.get()
        if event is None:
            break
        yield json.dumps(event, ensure_ascii=False) + "\n"


@app.post("/v1/chat")
async def chat(req: ChatRequest):
    """Start a streaming chat conversation. Returns newline-delimited JSON."""
    session_id = req.session_id or str(uuid.uuid4())
    event_queue: asyncio.Queue = asyncio.Queue()

    # Run AIAgent in a background thread
    thread = threading.Thread(
        target=_run_chat_in_thread,
        args=(session_id, req.message, req.model, req.toolsets, event_queue),
        daemon=True,
    )
    thread.start()

    return StreamingResponse(
        _chat_event_stream(event_queue),
        media_type="application/x-ndjson",
        headers={
            "X-Session-Id": session_id,
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
        },
    )


@app.post("/v1/abort")
async def abort(req: AbortRequest):
    """Abort a running conversation."""
    # Set abort event
    abort_event = _abort_events.get(req.session_id)
    if abort_event:
        abort_event.set()

    # Also call interrupt on the agent
    with _running_agents_lock:
        agent = _running_agents.get(req.session_id)
        if agent:
            agent.interrupt("User aborted")

    return {"ok": True, "session_id": req.session_id}


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
def main():
    global _main_loop
    import uvicorn

    port = int(os.environ.get("HERMES_CHAT_PORT", "18686"))
    host = os.environ.get("HERMES_CHAT_HOST", "127.0.0.1")

    print(f"Hermes Chat Server starting on http://{host}:{port}", file=sys.stderr)

    # Set up signal handlers
    def _signal_handler(signum, frame):
        print(f"Signal {signum} received, shutting down...", file=sys.stderr)
        sys.exit(0)

    signal.signal(signal.SIGINT, _signal_handler)
    signal.signal(signal.SIGTERM, _signal_handler)

    _main_loop = asyncio.new_event_loop()
    asyncio.set_event_loop(_main_loop)

    config = uvicorn.Config(
        app,
        host=host,
        port=port,
        log_level="warning",
        loop="asyncio",
    )
    server = uvicorn.Server(config)
    _main_loop.run_until_complete(server.serve())


if __name__ == "__main__":
    main()
