from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from app.services.auth_service import verify_access_token

router = APIRouter(tags=["Апи для работы с модпаками"])
security = HTTPBearer()

@router.get("/modpacks", summary="Список модпаков из манифеста")
def get(credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.post("/modpacks", summary="Создание нового модпака")
def get(credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.get("/modpacks/{id}", summary="Получение данных о конкретной версии")
def get(id: str, credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.patch("/modpacks/{id}", summary="Редактирование модпака")
def get(id: str, credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.delete("/modpacks/{id}", summary="Удаление существующего модпака")
def get(id: str, credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")

@router.post("/modpacks/build", summary="Общая генерация сборок")
def get(credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")