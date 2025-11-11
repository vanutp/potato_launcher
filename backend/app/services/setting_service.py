from enum import Enum
from fastapi import HTTPException, status
from typing import Any

class SettingType(str, Enum):
    STRING = "string"
    BOOLEAN = "boolean"
    INT = "int"

_settings_storage: list[dict[str, Any]] = []

def _detect_type(value: Any) -> SettingType:
    if isinstance(value, bool):
        return SettingType.BOOLEAN
    if isinstance(value, int):
        return SettingType.INT
    if isinstance(value, float):
        return SettingType.FLOAT
    if value is None:
        return SettingType.NULL
    return SettingType.STRING

def get_all_settings() -> list[dict[str, Any]]:
    return _settings_storage

def _validate_type(value: Any, declared_type: SettingType):
    actual = _detect_type(value)
    if actual != declared_type:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Type mismatch for value '{value}': declared '{declared_type}', actual '{actual}'"
        )

def create_or_update_settings(body: list[dict[str, Any]]) -> list[dict[str, Any]]:
    global _settings_storage

    new_settings = []
    for item in body:
        key = item["key"]
        value = item["value"]
        declared_type = SettingType(item["type"])

        _validate_type(value, declared_type)

        new_settings.append({
            "key": key,
            "value": value,
            "type": declared_type
        })

    for item in new_settings:
        existing = next((s for s in _settings_storage if s["key"] == item["key"]), None)
        if existing:
            existing.update(item)
        else:
            _settings_storage.append(item)

    return _settings_storage