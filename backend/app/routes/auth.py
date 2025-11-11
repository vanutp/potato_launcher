# app/routes/private.py
from fastapi import APIRouter, Depends, HTTPException
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel
from app.services.auth_service import create_access_token, verify_access_token
from app.services.settings import settings

router = APIRouter()
security = HTTPBearer()


class TokenRequest(BaseModel):
    token: str


@router.post("/auth")
def auth(request: TokenRequest):
    if request.token != settings.ADMIN_ACCESS_TOKEN:
        raise HTTPException(status_code=401, detail="Invalid token")

    jwt_token = create_access_token({"sub": "single_user"})
    return {"access_token": jwt_token, "token_type": "bearer"}


@router.get("/private")
def private_route(credentials: HTTPAuthorizationCredentials = Depends(security)):
    payload = verify_access_token(credentials.credentials)
    return {"message": f"Привет, {payload['sub']}!"}


@router.get("/public")
def public_route():
    return {"message": "Это публичный маршрут, доступен всем."}
