from enum import Enum
from fastapi import HTTPException, status
from typing import Any, List, Dict, Union
import json
from pathlib import Path
from app.models.setting import *

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

def load_file_settings(filename="spec.example.json") -> Dict[str, Union[str, bool]]:
    path = Path(filename)
    try:
        data = json.loads(path.read_text(encoding='utf-8'))
    except FileNotFoundError:
        data = {}
    res = {}
    for k, v in data.items():
        if isinstance(v, (str, bool)):
            res[k] = v
    return res

def get_all_settings() -> Dict[str, Union[str, bool]]:
    return load_file_settings()

def _validate_type(value: Any, declared_type: SettingType):
    actual = _detect_type(value)
    if actual != declared_type:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail=f"Type mismatch for value '{value}': declared '{declared_type}', actual '{actual}'"
        )
    
def get_value_with_type(value: str, setting_type: SettingType):
    if setting_type == SettingType.BOOLEAN:
        return bool(value)
    if value is None:
        return None
    return str(value)

def create_or_update_settings(body: List[SettingRequest], filename="spec.example.json") -> list[dict[str, Any]]:
    data = load_file_settings(filename)

    for setting in body:
        key = setting.key
        value = get_value_with_type(setting.value, setting.type)
        data[key] = value
        
    with open(filename, 'w', encoding='utf-8') as f:
        json.dump(data, f)
    return data