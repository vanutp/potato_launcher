from fastapi import APIRouter, Depends
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from typing import List
from app.services.auth_service import verify_access_token
from app.services.setting_service import *
from app.models.setting import *

router = APIRouter()
security = HTTPBearer()

@router.get(
    "/settings",
    summary="Получение параметров окружения",
    response_model=List[SettingResponse])
def get_settings(credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    return get_all_settings()


@router.post(
    "/settings",
    summary="Создание или обновление массива параметров окружения",
    response_model=List[SettingResponse])
def create_settings(body: List[SettingRequest], credentials: HTTPAuthorizationCredentials = Depends(security)):
    verify_access_token(credentials.credentials)
    return create_or_update_settings([item.model_dump() for item in body])