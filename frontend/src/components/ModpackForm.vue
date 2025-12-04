<script setup lang="ts">
import { onMounted, reactive, ref, watch } from 'vue';
import { Upload } from 'lucide-vue-next';
import { apiService } from '@/services/api';
import type { AuthConfig, ModpackBase } from '@/types/api';
import { AuthKind, LoaderType } from '@/types/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

const emit = defineEmits<{
  (event: 'submitted', payload: ModpackBase): void;
}>();

const formData = reactive<ModpackBase>({
  name: '',
  minecraft_version: '',
  loader: LoaderType.VANILLA,
  loader_version: '',
  auth_config: {
    kind: AuthKind.OFFLINE,
  },
});

const minecraftVersions = ref<string[]>([]);
const availableLoaders = ref<string[]>([]);
const loaderVersions = ref<string[]>([]);
const loading = ref(false);
const dragActive = ref(false);
const uploadedFiles = ref<FileList | null>(null);
const errors = reactive<Record<string, string>>({});

const validate = () => {
  const newErrors: Record<string, string> = {};
  if (!formData.name.trim()) newErrors.name = 'Name is required';
  if (!formData.minecraft_version) newErrors.minecraft_version = 'Minecraft version is required';
  if (!formData.loader) newErrors.loader = 'Loader is required';
  if (!formData.loader_version) newErrors.loader_version = 'Loader version is required';
  if (!formData.auth_config.kind) newErrors.auth_kind = 'Authentication type is required';

  if (formData.auth_config.kind === AuthKind.TELEGRAM && !formData.auth_config.auth_base_url?.trim()) {
    newErrors.auth_base_url = 'Auth base URL is required for Telegram';
  }

  if (formData.auth_config.kind === AuthKind.ELY_BY) {
    if (!formData.auth_config.client_id?.trim()) {
      newErrors.client_id = 'Client ID is required for Ely.by';
    }
    if (!formData.auth_config.client_secret?.trim()) {
      newErrors.client_secret = 'Client Secret is required for Ely.by';
    }
  }

  Object.keys(errors).forEach((key) => delete errors[key]);
  Object.assign(errors, newErrors);

  return Object.keys(newErrors).length === 0;
};

const resetForm = () => {
  formData.name = '';
  formData.minecraft_version = '';
  formData.loader = LoaderType.VANILLA;
  formData.loader_version = '';
  formData.auth_config = {
    kind: AuthKind.OFFLINE,
  };
  uploadedFiles.value = null;
};

const handleSubmit = async () => {
  if (!validate()) {
    return;
  }

  try {
    loading.value = true;
    const created = await apiService.createModpack(formData);
    if (uploadedFiles.value && uploadedFiles.value.length > 0) {
      await apiService.uploadModpackFiles(created.id, uploadedFiles.value);
    }
    const payload = {
      ...formData,
      auth_config: { ...formData.auth_config },
    };
    emit('submitted', payload);
    resetForm();
  } catch (err) {
    errors.submit = err instanceof Error ? err.message : 'Failed to create modpack';
  } finally {
    loading.value = false;
  }
};

const handleInputChange = (field: keyof ModpackBase, value: string | LoaderType) => {
  (formData as Record<string, unknown>)[field] = value;
  if (errors[field as string]) {
    delete errors[field as string];
  }
};

const handleAuthConfigChange = (field: keyof AuthConfig, value: string | AuthKind) => {
  formData.auth_config = {
    ...formData.auth_config,
    [field]: value,
    ...(field === 'kind'
      ? {
        auth_base_url: undefined,
        client_id: undefined,
        client_secret: undefined,
      }
      : {}),
  };

  if (errors[field as string]) {
    delete errors[field as string];
  }
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

const loadMinecraftVersions = async () => {
  try {
    minecraftVersions.value = await apiService.getMinecraftVersions();
  } catch (err) {
    console.error('Failed to load Minecraft versions:', err);
  }
};

const loadLoaders = async (version: string) => {
  if (!version) {
    availableLoaders.value = [];
    return;
  }
  try {
    availableLoaders.value = await apiService.getLoadersForVersion(version);
  } catch (err) {
    console.error('Failed to load loaders:', err);
    availableLoaders.value = [];
  }
};

const loadLoaderVersions = async (version: string, loader: string) => {
  if (!version || !loader) {
    loaderVersions.value = [];
    return;
  }
  try {
    loaderVersions.value = await apiService.getLoaderVersions(version, loader);
  } catch (err) {
    console.error('Failed to load loader versions:', err);
    loaderVersions.value = [];
  }
};

onMounted(() => {
  loadMinecraftVersions().catch((err) => console.error(err));
});

watch(
  () => formData.minecraft_version,
  (mcVersion) => {
    if (mcVersion) {
      loadLoaders(mcVersion).catch((err) => console.error(err));
      formData.loader = LoaderType.VANILLA;
      formData.loader_version = '';
    } else {
      availableLoaders.value = [];
      loaderVersions.value = [];
      formData.loader = LoaderType.VANILLA;
      formData.loader_version = '';
    }
  },
);

watch(
  () => [formData.minecraft_version, formData.loader] as const,
  ([mcVersion, loader]) => {
    if (mcVersion && loader) {
      loadLoaderVersions(mcVersion, loader).catch((err) => console.error(err));
      formData.loader_version = '';
    } else {
      loaderVersions.value = [];
      formData.loader_version = '';
    }
  },
);
</script>

<template>
  <div class="p-4">
    <Card>
      <CardHeader>
        <CardTitle>Create New Modpack</CardTitle>
        <CardDescription>Provision a new entry for Potato Launcher.</CardDescription>
      </CardHeader>
      <CardContent>
        <form class="space-y-5" @submit.prevent="handleSubmit">
          <Alert v-if="errors.submit" variant="destructive">
            <AlertDescription>{{ errors.submit }}</AlertDescription>
          </Alert>
          <div class="grid gap-4 sm:grid-cols-2">
            <div class="space-y-2 sm:col-span-2">
              <Label for="name">Modpack Name *</Label>
              <Input id="name" :model-value="formData.name" placeholder="Enter modpack name"
                @update:modelValue="(value) => handleInputChange('name', value?.toString() ?? '')" />
              <p v-if="errors.name" class="text-sm">{{ errors.name }}</p>
            </div>
            <div class="space-y-2">
              <Label>Minecraft Version *</Label>
              <Select :model-value="formData.minecraft_version || undefined"
                @update:modelValue="(value) => handleInputChange('minecraft_version', value?.toString() ?? '')">
                <SelectTrigger>
                  <SelectValue placeholder="Select version" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="version in minecraftVersions" :key="version" :value="version">
                    {{ version }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <p v-if="errors.minecraft_version" class="text-sm">{{ errors.minecraft_version }}</p>
            </div>
            <div class="space-y-2">
              <Label>Mod Loader *</Label>
              <Select :model-value="formData.loader || undefined"
                :disabled="!formData.minecraft_version || availableLoaders.length === 0" @update:modelValue="(value) =>
                  handleInputChange(
                    'loader',
                    (typeof value === 'string' && value.length ? value : LoaderType.VANILLA) as LoaderType,
                  )">
                <SelectTrigger>
                  <SelectValue placeholder="Select loader" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="loader in availableLoaders" :key="loader" :value="loader">
                    {{ loader }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <p v-if="errors.loader" class="text-sm">{{ errors.loader }}</p>
              <p v-else-if="!formData.minecraft_version" class="text-sm">
                Select a Minecraft version first.
              </p>
              <p v-else-if="availableLoaders.length === 0" class="text-sm">No loaders available.
              </p>
            </div>
            <div class="space-y-2">
              <Label>Loader Version *</Label>
              <Select :model-value="formData.loader_version || undefined"
                :disabled="!formData.loader || loaderVersions.length === 0"
                @update:modelValue="(value) => handleInputChange('loader_version', value?.toString() ?? '')">
                <SelectTrigger>
                  <SelectValue placeholder="Select version" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="version in loaderVersions" :key="version" :value="version">
                    {{ version }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <p v-if="errors.loader_version" class="text-sm">{{ errors.loader_version }}</p>
              <p v-else-if="!formData.loader" class="text-sm">Select a loader first.</p>
              <p v-else-if="loaderVersions.length === 0" class="text-sm">No versions available.
              </p>
            </div>
            <div class="space-y-2">
              <Label>Authentication Type *</Label>
              <Select :model-value="formData.auth_config.kind"
                @update:modelValue="(value) => handleAuthConfigChange('kind', (value as AuthKind) ?? AuthKind.OFFLINE)">
                <SelectTrigger>
                  <SelectValue placeholder="Select authentication" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem :value="AuthKind.OFFLINE">Offline</SelectItem>
                  <SelectItem :value="AuthKind.MOJANG">Mojang</SelectItem>
                  <SelectItem :value="AuthKind.TELEGRAM">Telegram</SelectItem>
                  <SelectItem :value="AuthKind.ELY_BY">Ely.by</SelectItem>
                </SelectContent>
              </Select>
              <p v-if="errors.auth_kind" class="text-sm">{{ errors.auth_kind }}</p>
            </div>
          </div>
          <div v-if="formData.auth_config.kind === AuthKind.TELEGRAM" class="space-y-2">
            <Label for="auth_base_url">Auth Base URL *</Label>
            <Input id="auth_base_url" type="url" :model-value="formData.auth_config.auth_base_url || ''"
              placeholder="https://your-telegram-auth-server.com"
              @update:modelValue="(value) => handleAuthConfigChange('auth_base_url', value?.toString() ?? '')" />
            <p v-if="errors.auth_base_url" class="text-sm">{{ errors.auth_base_url }}</p>
          </div>
          <div v-if="formData.auth_config.kind === AuthKind.ELY_BY" class="grid gap-4 sm:grid-cols-2">
            <div class="space-y-2">
              <Label for="client_id">Client ID *</Label>
              <Input id="client_id" :model-value="formData.auth_config.client_id || ''" placeholder="Ely.by client ID"
                @update:modelValue="(value) => handleAuthConfigChange('client_id', value?.toString() ?? '')" />
              <p v-if="errors.client_id" class="text-sm">{{ errors.client_id }}</p>
            </div>
            <div class="space-y-2">
              <Label for="client_secret">Client Secret *</Label>
              <Input id="client_secret" type="password" :model-value="formData.auth_config.client_secret || ''"
                placeholder="Ely.by client secret"
                @update:modelValue="(value) => handleAuthConfigChange('client_secret', value?.toString() ?? '')" />
              <p v-if="errors.client_secret" class="text-sm">{{ errors.client_secret }}</p>
            </div>
          </div>
          <div class="space-y-3">
            <Label>Upload Modpack Files (optional)</Label>
            <div class="relative rounded-md border border-dashed p-6 text-center text-sm" @dragenter="handleDrag"
              @dragleave="handleDrag" @dragover="handleDrag" @drop="handleDrop">
              <input type="file" multiple class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
                webkitdirectory="" @change="handleFileInput" />
              <Upload class="mx-auto mb-3 h-10 w-10" />
              <p>Drag a folder here or click to browse.</p>
            </div>
            <Alert v-if="uploadedFiles">
              <AlertDescription>{{ uploadedFiles.length }} file(s) selected</AlertDescription>
            </Alert>
          </div>
          <div>
            <Button type="submit" class="w-full" :disabled="loading">
              <span v-if="loading">Creating...</span>
              <span v-else>Create Modpack</span>
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  </div>
</template>
