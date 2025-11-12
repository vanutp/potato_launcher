from __future__ import annotations
import asyncio
from pathlib import Path
from typing import Any, Optional
from uuid import uuid4
from urllib.parse import urlparse

from fastapi import HTTPException

_BUILD_DIR = Path(__file__).resolve().parents[2] / "build_artifacts"
_PROJECT_ROOT = Path(__file__).resolve().parents[3]
_SPEC_FILE_NAME = "spec.json"


class RunnerService:
    def __init__(self) -> None:
        self._busy: bool = False
        self._debug: bool = True # TODO temp only for debug
        self._lock = asyncio.Lock()
        self._task: Optional[asyncio.Task] = None
        self._message: Optional[str] = None
        _BUILD_DIR.mkdir(parents=True, exist_ok=True)

    async def run_build(self, raw_spec: dict[str, Any]) -> dict[str, Any]:
        spec = self._prepare_spec(raw_spec)
        job_id = uuid4().hex
        spec_path = self._write_spec(job_id, spec)

        async with self._lock:
            if self._busy:
                raise HTTPException(status_code=409, detail="Another build is already running")
            self._busy = True
            self._message = "running"

            if self._debug:
                sleep_seconds = int(60)
                self._task = asyncio.create_task(self._dummy_build(sleep_seconds))
            else:
                self._task = asyncio.create_task(self._execute_command(spec_path))

        return {"busy": True, "message": "started"}

    async def get_status(self) -> dict[str, Any]:
        async with self._lock:
            return {"busy": self._busy, "message": self._message}

    async def _dummy_build(self, seconds: int) -> None:
        try:
            await asyncio.sleep(seconds)
            async with self._lock:
                self._message = "done (debug)"
                self._busy = False
        except asyncio.CancelledError:
            async with self._lock:
                self._message = "cancelled"
                self._busy = False
            raise
        except Exception as exc:
            async with self._lock:
                self._message = f"failed: {exc}"
                self._busy = False

    async def _execute_command(self, spec_path: Path) -> None:
        cmd = [
            "cargo",
            "run",
            "--release",
            "-p",
            "instance_builder",
            "--",
            "-s",
            str(spec_path),
        ]
        try:
            proc = await asyncio.create_subprocess_exec(
                *cmd,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=str(_PROJECT_ROOT),
            )

            stdout_b, stderr_b = await proc.communicate()
            msg = "ok" if proc.returncode == 0 else f"failed (code {proc.returncode})"
            # TODO log stdout/stderr here

            async with self._lock:
                self._message = msg
                self._busy = False
        except FileNotFoundError as exc:
            async with self._lock:
                self._message = f"command not found: {exc}"
                self._busy = False
        except Exception as exc:
            async with self._lock:
                self._message = f"failed: {exc}"
                self._busy = False

    def _prepare_spec(self, raw: dict[str, Any]) -> dict[str, Any]:
        if not isinstance(raw, dict):
            raise HTTPException(status_code=422, detail="Body must be a JSON object")

        # requaried
        for key in ["download_server_base", "resources_url_base", "version_manifest_url", "versions"]:
            if key not in raw:
                raise HTTPException(status_code=422, detail=f"Field '{key}' is required")

        if not isinstance(raw["versions"], list) or not raw["versions"]:
            raise HTTPException(status_code=422, detail="Field 'versions' must be a non-empty list")

        # urls
        for url_key in ["download_server_base", "resources_url_base", "version_manifest_url"]:
            self._ensure_url(raw[url_key], url_key)

        # version validate
        # raise HTTPException(status_code=400, detail=f"versions[{version}] not found")

        return raw

    def _ensure_url(self, value: Any, field: str) -> None:
        if not isinstance(value, str) or not value.strip():
            raise HTTPException(status_code=422, detail=f"Field '{field}' must be a non-empty string")
        parsed = urlparse(value)
        if not parsed.scheme or not parsed.netloc:
            raise HTTPException(status_code=422, detail=f"Field '{field}' must be a valid URL")

    def _write_spec(self, job_id: str, payload: dict[str, Any]) -> Path:
        job_dir = _BUILD_DIR / job_id
        job_dir.mkdir(parents=True, exist_ok=True)
        spec_path = job_dir / _SPEC_FILE_NAME

        import json
        spec_path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")
        return spec_path


runner_service = RunnerService()
