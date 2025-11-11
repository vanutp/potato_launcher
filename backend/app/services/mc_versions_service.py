from typing import List, Optional

from app.integrations.vanilla_versions import get_vanilla_version_list
from app.integrations.forge_loader import (
    forge_has_loader_for,
    get_forge_loader_versions,
)
from app.integrations.neoforge_loader import (
    neoforge_has_loader_for,
    get_neoforge_loader_versions,
)
from app.integrations.fabric_loader import (
    fabric_has_loader_for,
    get_fabric_loader_versions,
)
from async_lru import alru_cache


@alru_cache
async def get_vanilla_versions(version_type: Optional[str] = None) -> List[str]:
    return await get_vanilla_version_list(version_type)


@alru_cache
async def get_loaders_for_version(version: str) -> List[str]:
    # TODO проверить, что для переданный version в принципе существует в игре
    loaders: List[str] = ["vanilla"]

    if await fabric_has_loader_for(version):
        loaders.append("fabric")

    if await forge_has_loader_for(version):
        loaders.append("forge")

    if await neoforge_has_loader_for(version):
        loaders.append("neoforge")

    return loaders


@alru_cache
async def get_loader_versions(version: str, loader: str) -> list[str]:
    loader = loader.lower()

    if loader == "vanilla":
        return [version]

    elif loader == "fabric":
        return await get_fabric_loader_versions(version)

    elif loader == "forge":
        return await get_forge_loader_versions(version)

    elif loader == "neoforge":
        return await get_neoforge_loader_versions(version)

    else:
        return []
