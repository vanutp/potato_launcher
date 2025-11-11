from datetime import datetime, timedelta, timezone
from fastapi import HTTPException, status
from app.models.config import config
import jwt


def create_access_token(data: dict):
    expire = datetime.now(timezone.utc) + timedelta(minutes=config.ACCESS_TOKEN_EXPIRE_MINUTES)
    to_encode = {**data, "exp": expire}
    return jwt.encode(to_encode, config.SECRET_KEY, algorithm=config.ALGORITHM)


def verify_access_token(token: str):
    try:
        payload = jwt.decode(token, config.SECRET_KEY, algorithms=[config.ALGORITHM])
        if payload.get("sub") != "single_user":
            raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid JWT payload")
        return payload
    except jwt.ExpiredSignatureError:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Token expired")
    except jwt.PyJWTError:
        raise HTTPException(status_code=status.HTTP_401_UNAUTHORIZED, detail="Invalid JWT")
    