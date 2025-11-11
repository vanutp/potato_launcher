from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel
from enum import Enum
from typing import Dict, List

from app.services.auth_service import create_access_token, verify_access_token
from app.services.settings import settings

router = APIRouter()
security = HTTPBearer()

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

all_modpacks: List[Modpack] = [
    Modpack(
        id="1",
        name="Modpack 1",
        minecraft_version="1.21.8",
        loader_type=LoaderType.FABRIC,
        include=[
            ProcessPath(
                path="mods",
                override=True,
                is_recursive=False,
            ),
            ProcessPath(
                path="config",
                override=False,
                is_recursive=False,
            )
        ]
    )
]

@router.post("/modpacks")
def get_modpacks(credentials: HTTPAuthorizationCredentials = Depends(security)):
    _ = verify_access_token(credentials.credentials)
    return all_modpacks

    