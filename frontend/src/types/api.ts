export enum LoaderType {
  VANILLA = 'vanilla',
  FORGE = 'forge',
  FABRIC = 'fabric',
  NEOFORGE = 'neoforge',
}

export enum AuthKind {
  OFFLINE = 'offline',
  MOJANG = 'mojang',
  TELEGRAM = 'telegram',
  ELY_BY = 'ely.by',
}

export interface AuthConfig {
  kind: AuthKind;
  auth_base_url?: string;
  client_id?: string;
  client_secret?: string;
}

export interface ModpackResponse {
  id: number;
  name: string;
  minecraft_version: string;
  loader: LoaderType;
  loader_version: string;
  auth_config: AuthConfig;
}

export interface ModpackBase {
  name: string;
  minecraft_version: string;
  loader: LoaderType;
  loader_version: string;
  auth_config: AuthConfig;
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

