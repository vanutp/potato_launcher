from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from app.services.auth_service import verify_access_token

router = APIRouter(tags=["Апи для работы с параметрами окружения"])
security = HTTPBearer()

@router.get("/settings", summary="Получение параметров окружения")
def get(credentials: HTTPAuthorizationCredentials = Depends(security)) -> list[dict]:
    verify_access_token(credentials.credentials)
    raise HTTPException(status_code=422, detail="This is not implemented yet")