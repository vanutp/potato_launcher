/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL?: string;
  readonly VITE_DOWNLOAD_SERVER_BASE?: string;
  readonly VITE_RESOURCES_URL_BASE?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
