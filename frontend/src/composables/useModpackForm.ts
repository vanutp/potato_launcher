import { reactive, ref, watch, type Ref } from 'vue';
import { apiService } from '@/services/api';
import type { AuthBackend, ModpackBase } from '@/types/api';
import { AuthType, LoaderType } from '@/types/api';

type PartialModpackBase = Partial<Omit<ModpackBase, 'auth_backend'>> & {
  auth_backend?: Partial<AuthBackend>;
};

interface UseModpackFormOptions {
  initialData?: PartialModpackBase;
  guard?: Ref<boolean>;
  mode?: 'create' | 'edit';
}

const buildAuthBackend = (source?: Partial<AuthBackend>): AuthBackend => ({
  type: source?.type ?? AuthType.OFFLINE,
  auth_base_url: source?.auth_base_url,
  client_id: source?.client_id,
  client_secret: source?.client_secret,
});

const buildFormData = (source?: PartialModpackBase): ModpackBase => ({
  name: source?.name ?? '',
  minecraft_version: source?.minecraft_version ?? '',
  loader_name: source?.loader_name ?? LoaderType.VANILLA,
  loader_version: source?.loader_version ?? '',
  auth_backend: buildAuthBackend(source?.auth_backend),
});

export const useModpackForm = (options: UseModpackFormOptions = {}) => {
  const mode = options.mode ?? 'create';
  const guardRef = options.guard ?? ref(true);

  const formData = reactive<ModpackBase>(buildFormData(options.initialData));
  const minecraftVersions = ref<string[]>([]);
  const availableLoaders = ref<string[]>([]);
  const loaderVersions = ref<string[]>([]);

  const loadingMinecraftVersions = ref(false);
  const loadingLoaders = ref(false);
  const loadingLoaderVersions = ref(false);

  const uploadedFiles = ref<FileList | null>(null);
  const dragActive = ref(false);

  const setLoaderDefault = () => {
    formData.loader_name = LoaderType.VANILLA;
  };

  const resetLoaderVersion = () => {
    formData.loader_version = '';
  };

  const clearLoaderVersions = () => {
    loaderVersions.value = [];
    resetLoaderVersion();
  };

  const resetFormData = (next?: PartialModpackBase) => {
    const data = buildFormData(next);
    formData.name = data.name;
    formData.minecraft_version = data.minecraft_version;
    formData.loader_name = data.loader_name;
    formData.loader_version = data.loader_version;
    formData.auth_backend = { ...data.auth_backend };
  };

  const resetUploads = () => {
    uploadedFiles.value = null;
  };

  const loadMinecraftVersions = async () => {
    try {
      loadingMinecraftVersions.value = true;
      minecraftVersions.value = await apiService.getMinecraftVersions();
    } catch (err) {
      console.error('Failed to load Minecraft versions:', err);
      minecraftVersions.value = [];
    } finally {
      loadingMinecraftVersions.value = false;
    }
  };

  const loadLoaders = async (version: string) => {
    if (!version) {
      availableLoaders.value = [];
      return;
    }

    try {
      loadingLoaders.value = true;
      availableLoaders.value = await apiService.getLoadersForVersion(version);
    } catch (err) {
      console.error('Failed to load loaders:', err);
      availableLoaders.value = [];
    } finally {
      loadingLoaders.value = false;
    }
  };

  const loadLoaderVersions = async (version: string, loader: string) => {
    if (!version || !loader) {
      loaderVersions.value = [];
      return;
    }

    if (loader === LoaderType.VANILLA) {
      loaderVersions.value = [];
      resetLoaderVersion();
      return;
    }

    try {
      loadingLoaderVersions.value = true;
      loaderVersions.value = await apiService.getLoaderVersions(version, loader);
    } catch (err) {
      console.error('Failed to load loader versions:', err);
      loaderVersions.value = [];
    } finally {
      loadingLoaderVersions.value = false;
    }
  };

  watch(
    () => [guardRef.value, formData.minecraft_version] as const,
    ([guard, mcVersion]) => {
      if (!guard) {
        availableLoaders.value = [];
        loaderVersions.value = [];
        return;
      }

      if (!mcVersion) {
        availableLoaders.value = [];
        loaderVersions.value = [];
        setLoaderDefault();
        resetLoaderVersion();
        return;
      }

      loadLoaders(mcVersion).catch((err) => console.error(err));
      if (mode === 'create') {
        setLoaderDefault();
        resetLoaderVersion();
      }
    },
    { immediate: true },
  );

  watch(
    () => [guardRef.value, formData.minecraft_version, formData.loader_name] as const,
    ([guard, mcVersion, loader]) => {
      if (!guard) {
        loaderVersions.value = [];
        return;
      }

      if (!mcVersion || !loader) {
        loaderVersions.value = [];
        resetLoaderVersion();
        return;
      }

      if (loader === LoaderType.VANILLA) {
        clearLoaderVersions();
        return;
      }

      loadLoaderVersions(mcVersion, loader).catch((err) => console.error(err));
      if (mode === 'create') {
        resetLoaderVersion();
      }
    },
    { immediate: true },
  );

  const handleInputChange = (field: keyof ModpackBase, value: string | LoaderType) => {
    (formData as Record<string, unknown>)[field] = value;
  };

  const handleAuthBackendChange = (field: keyof AuthBackend, value: string | AuthType) => {
    formData.auth_backend = {
      ...formData.auth_backend,
      [field]: value,
      ...(field === 'type'
        ? {
          auth_base_url: undefined,
          client_id: undefined,
          client_secret: undefined,
        }
        : {}),
    };
  };

  const handleDrag = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
    if (event.type === 'dragenter' || event.type === 'dragover') {
      dragActive.value = true;
    } else if (event.type === 'dragleave') {
      dragActive.value = false;
    }
  };

  const handleDrop = (event: DragEvent) => {
    event.preventDefault();
    event.stopPropagation();
    dragActive.value = false;
    if (event.dataTransfer?.files?.length) {
      uploadedFiles.value = event.dataTransfer.files;
    }
  };

  const handleFileInput = (event: Event) => {
    const target = event.target as HTMLInputElement;
    if (target.files?.length) {
      uploadedFiles.value = target.files;
    }
  };

  return {
    formData,
    minecraftVersions,
    availableLoaders,
    loaderVersions,
    loadingMinecraftVersions,
    loadingLoaders,
    loadingLoaderVersions,
    uploadedFiles,
    dragActive,
    loadMinecraftVersions,
    loadLoaders,
    loadLoaderVersions,
    handleInputChange,
    handleAuthBackendChange,
    handleDrag,
    handleDrop,
    handleFileInput,
    resetFormData,
    resetUploads,
  };
};

