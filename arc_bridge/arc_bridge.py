"""
ARC-AGI-3 Bridge — routes FrameData between the ARC SDK and SpatialVortex.

Two operating modes:

  1. **Iggy mode** (default when Iggy available):
     ARC SDK → Iggy [agent_observations] → Rust arc-agent-iggy → Iggy [agent_commands] → ARC SDK

  2. **Subprocess mode** (ARC_BRIDGE_MODE=subprocess):
     ARC SDK → stdin JSON → Rust arc-agent-iggy → stdout JSON → ARC SDK

The bridge translates between arcengine.FrameData and the JSON observation
protocol understood by the vortex Rust agent.

Transport strategy for Iggy (in priority order):
  1. iggy-py native client  (fastest; sub-ms latency)
  2. Iggy HTTP REST fallback (no extra deps; ~1ms overhead)

Set IGGY_TRANSPORT=http to force the REST fallback.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
import uuid
from typing import Any, Optional

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

    def _make_client() -> Any:
        return IGGY_HTTP_BASE

    def _publish(client: Any, stream: str, topic: str, payload: str) -> None:
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

    def _poll_once(client: Any, stream: str, topic: str) -> list[str]:
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


# ── FrameData → Observation JSON ──────────────────────────────────────────

def frame_to_observation(
    frame_data: dict,
    game_id: str,
    step: int,
    terminated: bool = False,
) -> dict:
    """Convert ARC-AGI-3 FrameData dict to the vortex observation protocol."""
    return {
        "frame": frame_data.get("frame", [[[0]]]),
        "game_id": game_id,
        "state": frame_data.get("state", "PLAYING"),
        "levels_completed": frame_data.get("levels_completed", 0),
        "win_levels": frame_data.get("win_levels", 0),
        "available_actions": frame_data.get("available_actions", []),
        "step": step,
        "terminated": terminated,
        "score": float(frame_data.get("levels_completed", 0)),
    }


def command_to_game_action(cmd: dict) -> tuple[int, Optional[dict]]:
    """Convert a vortex Command response to (action_id, optional_data)."""
    action_id = cmd.get("action_id", 0)
    x = cmd.get("x")
    y = cmd.get("y")
    data = None
    if x is not None and y is not None:
        data = {"x": x, "y": y}
    return action_id, data


# ── Iggy Bridge ───────────────────────────────────────────────────────────

class IggyBridge:
    """Routes observations/commands through Iggy for the Rust agent."""

    def __init__(self, game_id: str) -> None:
        self.game_id = game_id
        self.client = _make_client()
        print(f"[arc_bridge] transport={_TRANSPORT}  game={game_id}")

    def publish_observation(self, obs: dict) -> None:
        _publish(self.client, IGGY_STREAM, TOPIC_OBSERVATIONS, json.dumps(obs))

    def poll_command(self, step: int, timeout_s: float = 5.0) -> Optional[dict]:
        deadline = time.monotonic() + timeout_s
        while time.monotonic() < deadline:
            messages = _poll_once(self.client, IGGY_STREAM, TOPIC_COMMANDS)
            for raw in messages:
                try:
                    data = json.loads(raw)
                    if data.get("step") == step:
                        return data
                except json.JSONDecodeError:
                    pass
            time.sleep(0.01)
        return None


# ── Subprocess Bridge ──────────────────────────────────────────────────────

class SubprocessBridge:
    """Routes observations/commands through stdin/stdout to the Rust binary."""

    def __init__(self, binary_path: str) -> None:
        if os.name == "nt" and not binary_path.endswith(".exe"):
            binary_path += ".exe"

        if not os.path.exists(binary_path):
            raise FileNotFoundError(f"Vortex binary not found: {binary_path}")

        env = os.environ.copy()
        env["ARC_AGENT_MODE"] = "standalone"
        self.proc = subprocess.Popen(
            [binary_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env,
            bufsize=1,
        )
        print(f"[arc_bridge] subprocess PID={self.proc.pid}")

    def send_and_receive(self, obs: dict) -> dict:
        """Send observation, read back command."""
        line = json.dumps(obs) + "\n"
        self.proc.stdin.write(line.encode())
        self.proc.stdin.flush()

        response = self.proc.stdout.readline()
        if response:
            return json.loads(response.decode().strip())
        return {"action_id": 0, "confidence": 0.0, "reasoning": "no_response"}

    def close(self) -> None:
        if self.proc.poll() is None:
            self.proc.terminate()
            try:
                self.proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.proc.kill()


# ── CLI for standalone Iggy bridge ─────────────────────────────────────────

def main() -> None:
    """
    CLI for running the bridge in Iggy mode alongside the ARC-AGI-3 SDK.
    The ARC-AGI-3-Agents framework's vortex_agent.py uses SubprocessBridge
    directly, so this CLI is for the full Iggy pipeline with arc-agent-iggy.
    """
    import argparse

    parser = argparse.ArgumentParser(description="ARC-AGI-3 <-> Iggy bridge")
    parser.add_argument("--game", required=True, help="Game ID (e.g. ls20)")
    parser.add_argument("--episodes", type=int, default=1)
    parser.add_argument(
        "--mode",
        choices=["iggy", "subprocess"],
        default="iggy",
        help="Communication mode",
    )
    parser.add_argument(
        "--binary",
        default=os.path.join(
            os.path.dirname(__file__),
            "..", "aimodel", "target", "release", "arc-agent-iggy",
        ),
        help="Path to vortex agent binary (for subprocess mode)",
    )
    args = parser.parse_args()

    print(f"[arc_bridge] mode={args.mode} game={args.game}")

    try:
        from arc_agi import Arcade
        from arcengine import GameAction, GameState

        arcade = Arcade()
        env = arcade.make(args.game)
    except ImportError:
        print("ERROR: arc-agi SDK not installed. Run: pip install arc-agi")
        sys.exit(1)

    if args.mode == "subprocess":
        bridge = SubprocessBridge(args.binary)
    else:
        bridge = IggyBridge(args.game)

    for ep in range(args.episodes):
        print(f"\n=== Episode {ep + 1}/{args.episodes} ===")
        raw_frame = env.reset()
        step = 0

        while True:
            frame_dict = raw_frame if isinstance(raw_frame, dict) else {}
            obs = frame_to_observation(frame_dict, args.game, step)

            if args.mode == "subprocess":
                cmd = bridge.send_and_receive(obs)
            else:
                bridge.publish_observation(obs)
                cmd = bridge.poll_command(step, timeout_s=5.0)
                if cmd is None:
                    print(f"  step={step}: timeout — random fallback")
                    import random
                    cmd = {"action_id": random.randint(1, 5), "step": step}

            action_id, data = command_to_game_action(cmd)

            try:
                action = GameAction.from_id(action_id)
            except (ValueError, KeyError):
                action = GameAction.ACTION1

            if data:
                action.set_data(data)

            raw_frame, reward, terminated, info = env.step(action)
            step += 1

            if terminated:
                # Send terminal observation
                term_obs = frame_to_observation(
                    raw_frame if isinstance(raw_frame, dict) else {},
                    args.game, step, terminated=True,
                )
                if args.mode == "subprocess":
                    bridge.send_and_receive(term_obs)
                else:
                    bridge.publish_observation(term_obs)

                print(f"  Episode done: steps={step} reward={reward}")
                break

    if args.mode == "subprocess":
        bridge.close()


if __name__ == "__main__":
    main()
