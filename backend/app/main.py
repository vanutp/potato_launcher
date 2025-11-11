from fastapi import FastAPI
from app.routes import auth
from app.routes import modpack

app = FastAPI()

app.include_router(auth.router, prefix="/api/v1")
app.include_router(modpack.router, prefix="/api/v1")
