export enum LoaderType {
  VANILLA = "vanilla",
  FORGE = "forge",
  FABRIC = "fabric",
  NEOFORGE = "neoforge"
}

export interface ModpackResponse {
  id: number;
  name: string;
  minecraft_version: string;
  loader: LoaderType;
  loader_version: string;
}

export interface ModpackBase {
  name: string;
  minecraft_version: string;
  loader: LoaderType;
  loader_version: string;
}

export enum SettingType {
  STRING = "string",
  BOOLEAN = "boolean"
}

export interface SettingResponse {
  key: string;
  value: string | boolean;
  type: SettingType;
}