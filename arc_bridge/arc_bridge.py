"""
ARC-AGI-3 Bridge — publishes ARC observations to Iggy and reads agent actions back.

Transport strategy (in priority order):
  1. iggy-py native client  (fastest; sub-ms latency)
  2. Iggy HTTP REST fallback (no extra deps; ~1ms overhead; always available)

Set IGGY_TRANSPORT=http to force the REST fallback regardless of iggy-py status.
"""

from __future__ import annotations

import json
import os
import time
import uuid
from typing import Any

# ── Iggy transport ─────────────────────────────────────────────────────────

IGGY_URL = os.getenv("IGGY_URL", "iggy://iggy:iggy@127.0.0.1:8090")
IGGY_HTTP_BASE = os.getenv("IGGY_HTTP_BASE", "http://127.0.0.1:3000")
IGGY_STREAM = "eustress"
TOPIC_OBSERVATIONS = "agent_observations"
TOPIC_COMMANDS = "agent_commands"

_USE_HTTP = os.getenv("IGGY_TRANSPORT", "").lower() == "http"

try:
    if _USE_HTTP:
        raise ImportError("HTTP transport forced via IGGY_TRANSPORT=http")
    from iggy.client import IggyClient as _IggyClient

    def _make_client() -> Any:
        return _IggyClient(IGGY_URL)

    def _publish(client: Any, stream: str, topic: str, payload: str) -> None:
        client.publish(stream, topic, payload)

    def _poll_once(client: Any, stream: str, topic: str) -> list[str]:
        msg = client.poll(stream, topic, count=1)
        return msg if msg else []

    _TRANSPORT = "iggy-py"

except ImportError:
    import urllib.request

    def _make_client() -> Any:  # type: ignore[misc]
        """HTTP REST client — just returns the base URL as a sentinel."""
        return IGGY_HTTP_BASE

    def _publish(client: Any, stream: str, topic: str, payload: str) -> None:  # type: ignore[misc]
        body = json.dumps(
            {"messages": [{"id": str(uuid.uuid4()), "payload": payload}]}
        ).encode()
        req = urllib.request.Request(
            f"{client}/streams/{stream}/topics/{topic}/messages",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        urllib.request.urlopen(req, timeout=3)

    def _poll_once(client: Any, stream: str, topic: str) -> list[str]:  # type: ignore[misc]
        req = urllib.request.Request(
            f"{client}/streams/{stream}/topics/{topic}/messages"
            "?consumer_id=arc_bridge&partition_id=1&count=1&auto_commit=true",
            method="GET",
        )
        try:
            with urllib.request.urlopen(req, timeout=2) as resp:
                data = json.loads(resp.read())
                return [m["payload"] for m in data.get("messages", [])]
        except Exception:
            return []

    _TRANSPORT = "http-rest"

# ── ARC SDK imports ────────────────────────────────────────────────────────

from arc_agi import Arcade  # pip install arc-agi
from arcengine import GameAction  # pip install arc-agi


# ── Bridge ─────────────────────────────────────────────────────────────────


class ArcBridge:
    """
    Runs one ARC-AGI-3 episode, routing observations through Iggy so the
    Rust agent (eustress-arc-agent) can process them.
    """

    def __init__(self, task_id: str) -> None:
        self.task_id = task_id
        self.arc = Arcade()
        self.env = self.arc.make(task_id, render_mode="terminal")
        self.client = _make_client()
        print(f"[arc_bridge] transport={_TRANSPORT}  task={task_id}")

    # ── Public API ─────────────────────────────────────────────────────────

    def run_episode(self) -> dict:
        """
        Run one full episode.  Returns the ARC scorecard dict when the
        environment signals termination.
        """
        obs = self.env.reset()
        step = 0

        while True:
            # ── Publish observation ────────────────────────────────────────
            payload = {
                "kind": "EnvironmentState",
                "json": json.dumps(obs),
                "step": step,
                "terminated": False,
            }
            _publish(self.client, IGGY_STREAM, TOPIC_OBSERVATIONS, json.dumps(payload))

            # ── Wait for action from Rust agent ────────────────────────────
            action_name = self._poll_action(step, timeout_s=5.0)
            if action_name is None:
                print(f"[arc_bridge] step={step}: action timeout — using fallback")
                action_name = self._fallback_action(obs)

            # ── Step environment ───────────────────────────────────────────
            game_action = getattr(GameAction, action_name, None)
            if game_action is None:
                print(
                    f"[arc_bridge] unknown action '{action_name}' — using fallback"
                )
                game_action = self._fallback_game_action(obs)

            obs, _reward, terminated, _info = self.env.step(game_action)
            step += 1

            # ── Publish terminal observation ───────────────────────────────
            if terminated:
                scorecard = self.arc.get_scorecard()
                terminal_payload = {
                    "kind": "EnvironmentState",
                    "json": json.dumps(obs),
                    "step": step,
                    "terminated": True,
                }
                _publish(
                    self.client,
                    IGGY_STREAM,
                    TOPIC_OBSERVATIONS,
                    json.dumps(terminal_payload),
                )
                print(
                    f"[arc_bridge] episode done  steps={step}  "
                    f"score={scorecard.get('score', '?')}"
                )
                return scorecard

    # ── Internal helpers ───────────────────────────────────────────────────

    def _poll_action(self, step: int, timeout_s: float) -> str | None:
        """
        Poll TOPIC_COMMANDS until a message matching *step* arrives or
        *timeout_s* elapses.
        """
        deadline = time.monotonic() + timeout_s
        while time.monotonic() < deadline:
            messages = _poll_once(self.client, IGGY_STREAM, TOPIC_COMMANDS)
            for raw in messages:
                try:
                    data = json.loads(raw)
                    if data.get("step") == step:
                        return data.get("action")
                except json.JSONDecodeError:
                    pass
            time.sleep(0.01)
        return None

    @staticmethod
    def _fallback_action(obs: Any) -> str:
        """Random fallback action drawn from obs['available_actions']."""
        import random

        actions: list[str] = []
        if isinstance(obs, dict):
            actions = obs.get("available_actions", [])
        return random.choice(actions) if actions else "ACTION1"

    @staticmethod
    def _fallback_game_action(obs: Any) -> Any:
        action_name = ArcBridge._fallback_action(obs)
        return getattr(GameAction, action_name, GameAction.ACTION1)


# ── CLI entry point ────────────────────────────────────────────────────────


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="ARC-AGI-3 ↔ Iggy bridge")
    parser.add_argument(
        "--task",
        required=True,
        help="ARC task ID to run (e.g. ls20)",
    )
    parser.add_argument(
        "--episodes",
        type=int,
        default=1,
        help="Number of episodes to run (default: 1)",
    )
    args = parser.parse_args()

    bridge = ArcBridge(args.task)
    for ep in range(args.episodes):
        print(f"\n=== Episode {ep + 1}/{args.episodes} ===")
        scorecard = bridge.run_episode()
        print(f"Scorecard: {scorecard}")


if __name__ == "__main__":
    main()
