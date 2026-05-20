#!/usr/bin/env python3
"""
Hermes Chat HTTP Server

FastAPI-based HTTP server that wraps AIAgent.run_conversation() with NDJSON streaming.
Replaces the stdin/stdout JSON bridge for SuperTool agent chat.

Key optimization: caches AIAgent per session_id (same pattern as Hermes CLI),
eliminating system prompt rebuild, OpenAI client re-creation, and tool re-discovery
on follow-up messages.

Endpoints:
  GET  /v1/health           → Health check
  POST /v1/chat             → NDJSON streaming chat
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
from typing import Optional, Dict, Any, List, Tuple

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
# Parse reasoning_config from Hermes config (same as CLI's _parse_reasoning_config)
# ---------------------------------------------------------------------------
try:
    from hermes_constants import parse_reasoning_effort as _parse_reasoning_effort
except ImportError:
    _parse_reasoning_effort = None

# ---------------------------------------------------------------------------
# Global state
# ---------------------------------------------------------------------------
_session_db: Optional[SessionDB] = None
_running_agents: Dict[str, AIAgent] = {}       # session_id → AIAgent (currently running)
_running_agents_lock = threading.Lock()
_abort_events: Dict[str, threading.Event] = {}  # session_id → Event

# Agent cache: reuse AIAgent across turns (same pattern as Hermes CLI)
# Saves ~300ms by avoiding system prompt rebuild, tool re-discovery, OpenAI client re-creation
_agent_cache: Dict[str, AIAgent] = {}           # session_id → AIAgent (idle, cached)
_agent_cache_lock = threading.Lock()
# Per-session locks prevent concurrent run_conversation on the same cached agent
_session_locks: Dict[str, threading.Lock] = {}
_session_locks_lock = threading.Lock()

_cli_config = load_cli_config()
_model_config = _cli_config.get("model", {})
if isinstance(_model_config, dict):
    _default_model = _model_config.get("default") or _model_config.get("model") or ""
else:
    _default_model = _model_config or ""

# reasoning_config: read from agent.reasoning_effort (same path as CLI)
_reasoning_config = None
if _parse_reasoning_effort is not None:
    _reasoning_effort = _cli_config.get("agent", {}).get("reasoning_effort", "")
    if _reasoning_effort:
        _reasoning_config = _parse_reasoning_effort(_reasoning_effort)


def _ensure_session_db() -> SessionDB:
    global _session_db
    if _session_db is None:
        _session_db = SessionDB()
    return _session_db


def _get_session_lock(session_id: str) -> threading.Lock:
    """Get or create a per-session lock to serialize access to cached agent."""
    with _session_locks_lock:
        if session_id not in _session_locks:
            _session_locks[session_id] = threading.Lock()
        return _session_locks[session_id]


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
    """Run AIAgent conversation in a background thread, pushing events to asyncio queue.

    Optimized: caches AIAgent per session_id so follow-up messages skip the
    expensive __init__ (system prompt, tool discovery, OpenAI client).
    """
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

        # ── Agent cache lookup: reuse existing agent for this session ──
        cached_agent: Optional[AIAgent] = None
        with _agent_cache_lock:
            cached_agent = _agent_cache.get(session_id)

        if cached_agent is not None:
            # Reuse cached agent — no system prompt rebuild, no tool re-discovery
            agent = cached_agent
            # Use in-memory session_messages instead of reloading from SQLite
            conversation_history = getattr(agent, "_session_messages", None)
        else:
            # First message for this session — load history from DB
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

        # ── Create agent (only on first message for this session) ──
        if cached_agent is None:
            agent = AIAgent(
                model=model or _default_model,
                session_id=session_id,
                session_db=session_db,
                enabled_toolsets=enabled_toolsets,
                max_iterations=90,
                reasoning_config=_reasoning_config,
                stream_delta_callback=stream_callback,
                tool_start_callback=tool_start_callback,
                tool_complete_callback=tool_complete_callback,
                platform="supertool",
                quiet_mode=True,
            )
            # Cache agent for follow-up messages
            with _agent_cache_lock:
                _agent_cache[session_id] = agent
        else:
            # Cached agent needs callbacks updated (new queue/abort per request)
            agent._stream_delta_callback = stream_callback
            agent._tool_start_callback = tool_start_callback
            agent._tool_complete_callback = tool_complete_callback

        # Serialize access to this session's agent (prevent concurrent run_conversation)
        session_lock = _get_session_lock(session_id)
        session_lock.acquire()
        try:
            with _running_agents_lock:
                _running_agents[session_id] = agent

            # Run conversation — uses cached system prompt + warm OpenAI client
            result = agent.run_conversation(
                message,
                conversation_history=conversation_history,
            )

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
        finally:
            session_lock.release()

    except Exception as e:
        _put_event("error", {"message": str(e)})
        import traceback
        traceback.print_exc()
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

app = FastAPI(title="Hermes Chat Server", version="2.0.0")


class ChatRequest(BaseModel):
    message: str
    session_id: Optional[str] = None
    model: Optional[str] = None
    toolsets: Optional[List[str]] = None


class AbortRequest(BaseModel):
    session_id: str


@app.get("/v1/health")
async def health():
    return {"status": "ok", "version": "2.0.0"}


async def _chat_event_stream(event_queue: asyncio.Queue):
    """Newline-delimited JSON stream (same format as old bridge)."""
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

    print(f"Hermes Chat Server v2 starting on http://{host}:{port}", file=sys.stderr)
    print(f"  Agent cache enabled: will reuse AIAgent per session", file=sys.stderr)

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
