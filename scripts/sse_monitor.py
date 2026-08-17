#!/usr/bin/env python3
"""SSE endpoint observer + event trigger for the Music Licensing backend.

Connects to the license-events SSE endpoint, parses the event stream, and
optionally drives the license workflow API to generate live events.

Environment variables (all optional):
    SSE_BASE_URL            base URL (default http://localhost:8080)
    SSE_MOVIE_ID            reuse existing movie (else auto-create)
    SSE_SCENE_ID            reuse existing scene (else auto-create)
    SSE_TRACK_ID            reuse existing track (else auto-create)
    SSE_MAX_EVENTS          stop after this many events (default 20)
    SSE_TIMEOUT_SECONDS     hard overall timeout (default 30)
    SSE_CONNECT_TIMEOUT     per-request connect timeout (default 5)
    SSE_IDLE_WINDOW         max seconds without data before closing (default 15)

Outputs a JSON report:
    {"status": ..., "event_count": ..., "sample_updates": [...], "errors_encountered": ...}
"""

import json
import os
import sys
import threading
import time

import requests


def _env(name, default):
    return os.environ.get(name, default)


def _env_int(name, default):
    try:
        return int(_env(name, ""))
    except ValueError:
        return default


def config():
    return {
        "base_url": _env("SSE_BASE_URL", "http://localhost:8080").rstrip("/"),
        "movie_id": _env("SSE_MOVIE_ID", ""),
        "scene_id": _env("SSE_SCENE_ID", ""),
        "track_id": _env("SSE_TRACK_ID", ""),
        "max_events": _env_int("SSE_MAX_EVENTS", 20),
        "timeout_seconds": _env_int("SSE_TIMEOUT_SECONDS", 30),
        "connect_timeout": _env_int("SSE_CONNECT_TIMEOUT", 5),
        "idle_window": _env_int("SSE_IDLE_WINDOW", 15),
    }


def _post(base_url, path, payload):
    resp = requests.post(
        f"{base_url}{path}",
        json=payload,
        timeout=10,
    )
    resp.raise_for_status()
    return resp.json()


def _patch(base_url, path, payload):
    resp = requests.patch(
        f"{base_url}{path}",
        json=payload,
        timeout=10,
    )
    resp.raise_for_status()
    return resp.json()


def resolve_track(cfg):
    """Return (movie_id, scene_id, track_id) reusing or creating resources."""
    if cfg["movie_id"] and cfg["scene_id"] and cfg["track_id"]:
        return cfg["movie_id"], cfg["scene_id"], cfg["track_id"]

    movie_id = cfg["movie_id"]
    if not movie_id:
        movie = _post(cfg["base_url"], "/movies", {
            "title": "SSE Monitor Movie",
            "release_date": "2020-01-01",
        })
        movie_id = str(movie["id"])

    scene_id = cfg["scene_id"]
    if not scene_id:
        scene = _post(cfg["base_url"], f"/movies/{movie_id}/scenes", {
            "scene_number": 1,
        })
        scene_id = str(scene["id"])

    track_id = cfg["track_id"]
    if not track_id:
        track = _post(cfg["base_url"], f"/movies/{movie_id}/scenes/{scene_id}/tracks", {
            "track_order": 1,
            "name": "SSE Monitor Track",
            "song": {
                "title": "Monitoring Theme",
                "artist": "Auto",
                "durationSecStart": 0,
                "durationSecEnd": 180,
            },
        })
        track_id = str(track["id"])

    return movie_id, scene_id, track_id


def _transition_sequence(max_events):
    """Build a sequence of target_status values, one per published event.

    Initiate produces DRAFT. Transitions follow the valid workflow paths:
    DRAFT -> NEGOTIATING -> (REJECTED <-> NEGOTIATING)* -> APPROVED.
    """
    if max_events <= 0:
        return []
    seq = ["NEGOTIATING"]
    for _ in range(max_events - 2):
        seq.append("REJECTED" if seq[-1] == "NEGOTIATING" else "NEGOTIATING")
    if max_events > 1:
        seq.append("APPROVED")
    return seq


def trigger_events(cfg, movie_id, scene_id, track_id, stop_event):
    """Drive the license workflow to publish events. Runs in a daemon thread."""
    errors = []
    base = cfg["base_url"]
    license_path = f"/movies/{movie_id}/scenes/{scene_id}/tracks/{track_id}/licenses"
    try:
        _post(base, license_path, {
            "label_name": "Monitor Label",
            "negotiation_notes": {"source": "sse_monitor"},
        })
    except Exception as e:  # noqa: BLE001 - report and continue
        errors.append(f"initiate failed: {e}")
        stop_event.set()
        return errors

    for target in _transition_sequence(cfg["max_events"]):
        if stop_event.is_set():
            break
        try:
            _patch(base, f"{license_path}/status-transition", {
                "target_status": target,
                "notes": {"source": "sse_monitor"},
            })
        except Exception as e:  # noqa: BLE001
            errors.append(f"transition to {target} failed: {e}")
            stop_event.set()
            break
        time.sleep(0.2)
    return errors


def parse_sse_line(line):
    """Parse one SSE line. Returns (field, value) or None for keep-alive/blank."""
    line = line.decode("utf-8", errors="replace") if isinstance(line, bytes) else line
    if not line:
        return None
    if line.startswith(":"):
        return None  # keep-alive comment
    if line.startswith("data:"):
        return ("data", line[len("data:"):].lstrip())
    if line.startswith("event:"):
        return ("event", line[len("event:"):].strip())
    if line.startswith("id:"):
        return ("id", line[len("id:"):].strip())
    return None


def collect_events(cfg, movie_id, scene_id, track_id):
    """Connect to the SSE endpoint and collect events until a stop condition."""
    url = (
        f"{cfg['base_url']}/movies/{movie_id}/scenes/{scene_id}/"
        f"tracks/{track_id}/licenses/events"
    )
    events = []
    errors = []
    status = "timeout"

    headers = {
        "Accept": "text/event-stream",
        "Content-Type": "text/event-stream",
    }

    deadline = time.monotonic() + cfg["timeout_seconds"]
    stop_event = threading.Event()

    trigger_thread = threading.Thread(
        target=trigger_events,
        args=(cfg, movie_id, scene_id, track_id, stop_event),
        daemon=True,
    )
    trigger_thread.start()

    try:
        with requests.get(
            url,
            headers=headers,
            stream=True,
            timeout=(cfg["connect_timeout"], cfg["timeout_seconds"]),
        ) as resp:
            if resp.status_code != 200:
                status = "error"
                errors.append(f"HTTP {resp.status_code}: {resp.text}")
                return status, events, errors

            current = {}
            last_data = time.monotonic()

            for raw_line in resp.iter_lines(decode_unicode=True):
                if time.monotonic() > deadline:
                    status = "timeout"
                    break

                # Any received line (including keep-alives) means the stream is alive.
                last_data = time.monotonic()

                parsed = parse_sse_line(raw_line)
                if parsed is None:
                    if raw_line == "" and current:
                        # blank line terminates a multi-line event
                        events.append(current)
                        current = {}
                        if len(events) >= cfg["max_events"]:
                            status = "success"
                            break
                    continue
                field, value = parsed

                if field == "data":
                    prefix = "\n" if current.get("data") else ""
                    current["data"] = current.get("data", "") + prefix + value
                elif field == "event":
                    current["event"] = value
                elif field == "id":
                    current["id"] = value

                if time.monotonic() - last_data > cfg["idle_window"]:
                    status = "timeout"
                    errors.append("idle window exceeded (no data)")
                    break

    except requests.exceptions.ReadTimeout as e:
        status = "timeout"
        errors.append(f"read timed out (no data): {e}")
    except requests.exceptions.ConnectTimeout as e:
        status = "timeout"
        errors.append(f"connect timed out: {e}")
    except requests.exceptions.ConnectionError as e:
        status = "error"
        errors.append(f"connection error: {e}")
    except requests.exceptions.RequestException as e:
        status = "error"
        errors.append(f"request error: {e}")
    finally:
        stop_event.set()

    if trigger_thread.is_alive():
        trigger_thread.join(timeout=2.0)

    return status, events, errors


def main():
    cfg = config()

    if cfg["max_events"] <= 0 or cfg["timeout_seconds"] <= 0:
        print(json.dumps({
            "status": "error",
            "event_count": 0,
            "sample_updates": [],
            "errors_encountered": "SSE_MAX_EVENTS and SSE_TIMEOUT_SECONDS must be > 0",
        }))
        return 2

    try:
        movie_id, scene_id, track_id = resolve_track(cfg)
    except Exception as e:  # noqa: BLE001
        print(json.dumps({
            "status": "error",
            "event_count": 0,
            "sample_updates": [],
            "errors_encountered": f"resource setup failed: {e}",
        }))
        return 1

    status, events, errors = collect_events(cfg, movie_id, scene_id, track_id)

    report = {
        "status": status,
        "event_count": len(events),
        "sample_updates": events[-5:],
        "errors_encountered": "; ".join(errors) if errors else None,
    }
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
