from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException
from starlette import status

from app.gateway.modpack import ModpackGateway
from app.models.modpack import ModpackResponse, CreateModpackRequest, UpdateModpackRequest
from app.utils.security import verify_access_token
from app.utils.stub import Stub

router = APIRouter(prefix="/modpacks", tags=["Modpacks"], dependencies=[Depends(verify_access_token)])


@router.get(
    path="",
    summary="List modpacks",
    response_model=list[ModpackResponse]
)
def get(
        modpack_gateway: Annotated[ModpackGateway, Depends(Stub(ModpackGateway))]
):
    return modpack_gateway.get_all()


@router.post("", summary="Create new modpack", response_model=ModpackResponse)
def get(
        modpack_gateway: Annotated[ModpackGateway, Depends(Stub(ModpackGateway))],
        body: CreateModpackRequest
):
    return modpack_gateway.save(body.to_model())


@router.get("/{id}", summary="Get info about the modpack", response_model=ModpackResponse)
def get(
        modpack_gateway: Annotated[ModpackGateway, Depends(Stub(ModpackGateway))],
        id: int
):
    modpack = modpack_gateway.get_by_id(id)
    if modpack is None:
        raise HTTPException(status_code=404, detail="Modpack not found")
    return modpack


@router.patch("/{id}", summary="Edit modpack")
def get(
        id: int,
        body: UpdateModpackRequest,
        modpack_gateway: Annotated[ModpackGateway, Depends(Stub(ModpackGateway))]
):
    return modpack_gateway.update(body.as_model(id))


@router.delete("/{id}", summary="Delete modpack", status_code=status.HTTP_204_NO_CONTENT)
def get(
        id: int,
        modpack_gateway: Annotated[ModpackGateway, Depends(Stub(ModpackGateway))],
):
    modpack = modpack_gateway.get_by_id(id)
    if modpack is None:
        raise HTTPException(status_code=404, detail="Modpack not found")
    modpack_gateway.delete(id)


@router.post("/build", summary="Run build")
def build():
    raise HTTPException(status_code=422, detail="This is not implemented yet")
