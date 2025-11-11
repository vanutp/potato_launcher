from app.gateway.json.db import read_file, save_file
from app.gateway.modpack import ModpackGateway
from app.models.modpack import Modpack


class JsonModpackGateway(ModpackGateway):

    def save(self, modpack: Modpack) -> Modpack:
        data = read_file()
        modpacks = data.get("modpacks", [])

        new_id = (max((int(m.get("id", 0)) for m in modpacks), default=0) + 1)
        modpacks.append(
            {
                "id": new_id,
                "name": modpack.name,
                "minecraft_version": modpack.minecraft_version,
                "loader": modpack.loader,
                "loader_version": modpack.loader_version,
            }
        )
        data["modpacks"] = modpacks
        save_file(data)

        return Modpack(
            id=new_id,
            name=modpack.name,
            minecraft_version=modpack.minecraft_version,
            loader=modpack.loader.value,
            loader_version=modpack.loader_version,
        )

    def get_by_id(self, id: int) -> Modpack | None:
        data = read_file()
        modpacks = data.get("modpacks", [])

        for raw_modpack in modpacks:
            if raw_modpack["id"] == id:
                return Modpack(**raw_modpack)
        return None

    def update(self, modpack: Modpack) -> Modpack:
        data = read_file()
        modpacks = data.get("modpacks", [])

        modpack_id = modpack.id
        for i, raw in enumerate(modpacks):
            if raw.get("id") == modpack_id:
                modpacks[i] = {
                    "id": raw.get("id"),
                    "name": modpack.name,
                    "minecraft_version": modpack.minecraft_version,
                    "loader": modpack.loader.value,
                    "loader_version": modpack.loader_version,
                }
                data["modpacks"] = modpacks
                save_file(data)
                return Modpack(**modpacks[i])

        raise Exception("Modpack not found")

    def get_all(self) -> list[Modpack]:
        data = read_file()
        raw_modpacks = data.get("modpacks", [])
        return [Modpack(**raw) for raw in raw_modpacks]

    def delete(self, id: int) -> None:
        data = read_file()
        modpacks = data.get("modpacks", [])
        new_modpacks = [m for m in modpacks if m.get("id") != id]

        if len(new_modpacks) == len(modpacks):
            raise Exception(f"Modpack id {id} doesn't exist")

        data["modpacks"] = new_modpacks
        save_file(data)
