import uvicorn
from fastapi import FastAPI, APIRouter
from starlette import status
from starlette.requests import Request
from starlette.responses import JSONResponse

from app.config import config
from app.gateway.json.modpack import JsonModpackGateway
from app.gateway.json.setting import JsonSettingGateway
from app.gateway.modpack import ModpackGateway
from app.gateway.settings import SettingGateway
from app.routers import auth
from app.routers import modpacks
from app.routers import mc_versions
from app.routers import settings


async def handle_error(_: Request, exc: Exception) -> JSONResponse:
    return JSONResponse(
        status_code=status.HTTP_500_INTERNAL_SERVER_ERROR,
        content={
            "details": str(exc)
        }
    )


def catch_exceptions(app: FastAPI) -> None:
    app.add_exception_handler(ValueError, handle_error)
    app.add_exception_handler(Exception, handle_error)


def register_routers(app: FastAPI) -> None:
    api_v1_router = APIRouter(prefix="/api/v1")

    api_v1_router.include_router(auth.router)
    api_v1_router.include_router(modpacks.router)
    api_v1_router.include_router(mc_versions.router)
    api_v1_router.include_router(settings.router)

    app.include_router(api_v1_router)


def create_app() -> FastAPI:
    app = FastAPI()

    app.dependency_overrides[SettingGateway] = lambda: JsonSettingGateway()
    app.dependency_overrides[ModpackGateway] = lambda: JsonModpackGateway()

    register_routers(app)
    catch_exceptions(app)

    return app


if __name__ == "__main__":
    app = create_app()
    uvicorn.run(app=app, host=config.HOST, port=config.PORT)
