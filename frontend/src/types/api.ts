export enum LoaderType {
  VANILLA = 'vanilla',
  FORGE = 'forge',
  FABRIC = 'fabric',
  NEOFORGE = 'neoforge',
}

export enum AuthType {
  OFFLINE = 'offline',
  MOJANG = 'mojang',
  TELEGRAM = 'telegram',
  ELY_BY = 'ely.by',
}

export interface AuthBackend {
  type: AuthType;
  auth_base_url?: string;
  client_id?: string;
  client_secret?: string;
}

export interface InstanceResponse {
  name: string;
  minecraft_version: string;
  loader_name: LoaderType;
  loader_version?: string;
  auth_backend: AuthBackend;
}

export interface InstanceBase {
  name: string;
  minecraft_version: string;
  loader_name: LoaderType;
  loader_version?: string;
  auth_backend: AuthBackend;
}

export enum SettingType {
  STRING = 'string',
  BOOLEAN = 'boolean',
}

export interface SettingResponse {
  key: string;
  value: string | boolean;
  type: SettingType;
}
