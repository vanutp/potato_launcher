from pydantic_settings import BaseSettings

class ConfigEnv(BaseSettings):
    SECRET_KEY: str
    ADMIN_ACCESS_TOKEN: str
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 60
    ALGORITHM: str = "HS256"

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"

config = ConfigEnv()
