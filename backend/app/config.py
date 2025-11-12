from pathlib import Path

from pydantic_settings import BaseSettings

ROOT = Path(__file__).resolve().parent.parent


class ConfigEnv(BaseSettings):
    ADMIN_JWT_SECRET: str
    ADMIN_SECRET_TOKEN: str
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 1440
    ALGORITHM: str = "HS256"
    HOST: str = "0.0.0.0"
    PORT: int = 8000

    class Config:
        env_file=str(ROOT / ".env")
        env_file_encoding = "utf-8"


config = ConfigEnv()
