from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from app.services.auth_service import verify_access_token
from app.models.modpack import *

router = APIRouter(tags=["Апи для работы с модпаками"])
security = HTTPBearer()

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

@router.get("/modpacks", summary="Список модпаков из манифеста")
def get(credentials: HTTPAuthorizationCredentials = Depends(security)):
    # verify_access_token(credentials.credentials)
    return all_modpacks

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