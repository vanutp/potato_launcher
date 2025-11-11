from pydantic import BaseModel
from enum import Enum
from typing import Any, Union


class SettingType(str, Enum):
    STRING = "string"
    BOOLEAN = "boolean"
    INT = "int"
    FLOAT = "float"
    NULL = "null"


class SettingRequest(BaseModel):
    key: str
    value: Union[str, bool]
    type: SettingType


class SettingResponse(BaseModel):
    key: str
    value: Union[str, bool]
    type: SettingType
