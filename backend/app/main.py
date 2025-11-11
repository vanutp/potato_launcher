from fastapi import FastAPI
from app.routes import auth
from app.routes import modpacks
from app.routes import mc_versions
from app.routes import settings

app = FastAPI()

app.include_router(auth.router, prefix="/api/v1")
app.include_router(modpacks.router, prefix="/api/v1")
app.include_router(mc_versions.router, prefix="/api/v1")
app.include_router(settings.router, prefix="/api/v1")