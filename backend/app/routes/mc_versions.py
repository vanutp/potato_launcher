from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from app.services.auth_service import verify_access_token
from typing import List

from app.services.mc_versions_service import (
    get_vanilla_versions,
    get_loaders_for_version,
    get_loader_versions,
)

router = APIRouter(tags=["Апи для работы с версиями"])
security = HTTPBearer()

@router.get("/mc-versions", response_model=List[str], summary="Получение версий ванильного майнкрафта")
async def get(credentials: HTTPAuthorizationCredentials = Depends(security)) -> List[str]:
    verify_access_token(credentials.credentials)
    
    versions = await get_vanilla_versions()
    return versions

@router.get("/mc-versions/{version}/loaders", response_model=List[str], summary="Получение доступных loader-ов для выбранной версии")
async def get(version: str, credentials: HTTPAuthorizationCredentials = Depends(security)) -> List[str]:
    verify_access_token(credentials.credentials)

    loaders = await get_loaders_for_version(version)
    if loaders is None:
        raise HTTPException(status_code=404, detail="Minecraft version not found")
    return loaders


@router.get("/mc-versions/{version}/{loader}", response_model=List[str], summary="Получение версий loader-ов майнкрафта для выбранной версии")
async def get(version: str, loader: str, credentials: HTTPAuthorizationCredentials = Depends(security)) -> List[str]:
    verify_access_token(credentials.credentials)

    loader_versions = await get_loader_versions(version, loader)
    if loader_versions is None:
        raise HTTPException(status_code=404, detail="Loader or version not found")
    return loader_versions
