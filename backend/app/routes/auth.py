from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from app.services.auth_service import create_access_token, verify_access_token
from app.models.config import config
from app.models.token import TokenRequest

router = APIRouter(tags=["Апи для авторизации"])
security = HTTPBearer()


@router.post("/auth/login", summary="Авторизация по токену")
def auth(request: TokenRequest):
    if request.token != config.ADMIN_ACCESS_TOKEN:
        raise HTTPException(status_code=401, detail="Invalid token")

    jwt_token = create_access_token({"sub": "single_user"})
    return {"access_token": jwt_token, "token_type": "bearer"}
