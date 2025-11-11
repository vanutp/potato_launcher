from fastapi import FastAPI
from starlette import status
from starlette.requests import Request
from starlette.responses import JSONResponse

from app.gateway.json.modpack import JsonModpackGateway
from app.gateway.json.setting import JsonSettingGateway
from app.gateway.modpack import ModpackGateway
from app.gateway.settings import SettingGateway
from app.routers import auth
from app.routers import modpacks
from app.routers import mc_versions
from app.routers import settings

app = FastAPI()

app.dependency_overrides[SettingGateway] = lambda: JsonSettingGateway()
app.dependency_overrides[ModpackGateway] = lambda: JsonModpackGateway()

app.include_router(auth.router, prefix="/api/v1")
app.include_router(modpacks.router, prefix="/api/v1")
app.include_router(mc_versions.router, prefix="/api/v1")
app.include_router(settings.router, prefix="/api/v1")


async def handle_error(_: Request, exc: Exception) -> JSONResponse:
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={
            "details": str(exc)
        }
    )


app.add_exception_handler(ValueError, handle_error)
app.add_exception_handler(Exception, handle_error)
