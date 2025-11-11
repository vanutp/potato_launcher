from typing import List
from pydantic import BaseModel
from enum import Enum

class LoaderType(str, Enum):
    FORGE = "forge"
    FABRIC = "fabric"

class ProcessPath(BaseModel):
    path: str
    override: bool
    is_recursive: bool

class Modpack(BaseModel):
    id: str
    name: str
    minecraft_version: str
    loader_type: LoaderType
    include: List[ProcessPath]