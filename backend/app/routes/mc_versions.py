from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from app.services.auth_service import verify_access_token
from typing import List

router = APIRouter(tags=["Апи для работы с версиями"])
security = HTTPBearer()

@router.get("/mc-versions", summary="Получение версий ванильного майнкрафта")
def get(credentials: HTTPAuthorizationCredentials = Depends(security)) -> List[str]:
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.get("/mc-versions/{version}/loaders", summary="Получение доступных loader-ов для выбранной версии")
def get(version: str, credentials: HTTPAuthorizationCredentials = Depends(security)) -> List[str]:
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.get("/mc-versions/{version}/{loader}", summary="Получение версий loader-ов майнкрафта для выбранной версии")
def get(version: str, loader: str, credentials: HTTPAuthorizationCredentials = Depends(security)) -> List[str]:
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")