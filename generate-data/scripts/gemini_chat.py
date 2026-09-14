"""Shared strict-JSON Gemini chat transport for data-generation scripts."""

from __future__ import annotations

import json
import time
from collections.abc import Callable
from typing import TypeVar

import requests

GEMINI_OPENAI_URL = (
    "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
)

T = TypeVar("T")


def sentence_pairs_schema(name: str) -> dict:
    return {
        "name": name,
        "strict": True,
        "schema": {
            "type": "object",
            "properties": {
                "sentences": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "target_language": {"type": "string"},
                            "native_language": {"type": "string"},
                        },
                        "required": ["target_language", "native_language"],
                        "additionalProperties": False,
                    },
                }
            },
            "required": ["sentences"],
            "additionalProperties": False,
        },
    }


def chat_json(
    api_key: str,
    model: str,
    messages: list[dict],
    schema: dict,
    *,
    parse: Callable[[dict], T],
    timeout: int,
    attempts: int,
) -> T:
    """POST strict-schema JSON; retry any request, parse, or validation failure."""
    payload = {
        "model": model,
        "temperature": 0,
        "response_format": {"type": "json_schema", "json_schema": schema},
        "messages": messages,
    }
    last_error: Exception | None = None
    for attempt in range(1, attempts + 1):
        try:
            response = requests.post(
                GEMINI_OPENAI_URL,
                headers={"Authorization": f"Bearer {api_key}"},
                json=payload,
                timeout=timeout,
            )
            response.raise_for_status()
            content = response.json()["choices"][0]["message"]["content"]
            return parse(json.loads(content))
        except Exception as error:  # noqa: BLE001 - retry all request/response failures
            last_error = error
            if attempt < attempts:
                time.sleep(min(60, 5 * 2 ** (attempt - 1)))
    raise RuntimeError(f"failed after {attempts} attempts: {last_error}") from last_error
