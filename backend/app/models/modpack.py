from dataclasses import dataclass
from typing import cast
from pydantic import BaseModel
from enum import Enum

from app.models.base import ApiModel


class LoaderType(str, Enum):
    FORGE = "forge"
    FABRIC = "fabric"
    NEOFORGE = "neoforge"


class ProcessPath(BaseModel):
    path: str
    override: bool
    is_recursive: bool


@dataclass
class Modpack:
    id: int
    name: str
    minecraft_version: str
    loader: LoaderType
    loader_version: str


class ModpackBase(ApiModel):
    name: str
    minecraft_version: str
    loader: LoaderType
    loader_version: str

    def to_model(self) -> Modpack:
        return Modpack(
            id=cast(int, None),
            name=self.name,
            minecraft_version=self.minecraft_version,
            loader=self.loader,
            loader_version=self.loader_version,
        )


class CreateModpackRequest(ModpackBase):
    ...


class UpdateModpackRequest(ModpackBase):

    def as_model(self, id: int) -> Modpack:
        return Modpack(
            id=id,
            name=self.name,
            minecraft_version=self.minecraft_version,
            loader=self.loader,
            loader_version=self.loader_version,
        )


class ModpackResponse(ModpackBase):
    id: int

    def to_model(self) -> Modpack:
        return Modpack(
            id=self.id,
            name=self.name,
            minecraft_version=self.minecraft_version,
            loader=self.loader,
            loader_version=self.loader_version,
        )
