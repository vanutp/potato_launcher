<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import { Pencil, Trash2, Upload } from 'lucide-vue-next';
import DeleteConfirmModal from './DeleteConfirmModal.vue';
import { apiService } from '@/services/api';
import type { AuthConfig, ModpackResponse } from '@/types/api';
import { AuthKind, LoaderType } from '@/types/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

const props = defineProps<{
  modpack: ModpackResponse;
}>();

const emit = defineEmits<{
  (event: 'updated', payload: { id: number; data: Partial<ModpackResponse> }): void;
  (event: 'deleted', id: number): void;
}>();

type EditableFields = Pick<ModpackResponse, 'name' | 'minecraft_version' | 'loader' | 'loader_version'> & {
  auth_config: AuthConfig;
};

const isEditing = ref(false);
const showDeleteConfirm = ref(false);
const dragActive = ref(false);
const uploadedFiles = ref<FileList | null>(null);
const updating = ref(false);

const minecraftVersions = ref<string[]>([]);
const availableLoaders = ref<string[]>([]);
const loaderVersions = ref<string[]>([]);
const loadingVersions = ref(false);
const loadingLoaders = ref(false);
const loadingLoaderVersions = ref(false);

const editData = reactive<EditableFields>({
  name: props.modpack.name,
  minecraft_version: props.modpack.minecraft_version,
  loader: props.modpack.loader,
  loader_version: props.modpack.loader_version,
  auth_config: props.modpack.auth_config,
});

const setEditDataFromProps = () => {
  editData.name = props.modpack.name;
  editData.minecraft_version = props.modpack.minecraft_version;
  editData.loader = props.modpack.loader;
  editData.loader_version = props.modpack.loader_version;
  editData.auth_config = { ...props.modpack.auth_config };
};

watch(
  () => props.modpack.id,
  () => {
    isEditing.value = false;
    showDeleteConfirm.value = false;
    uploadedFiles.value = null;
    minecraftVersions.value = [];
    availableLoaders.value = [];
    loaderVersions.value = [];
    setEditDataFromProps();
  },
);

const loadMinecraftVersions = async () => {
  loadingVersions.value = true;
  try {
    minecraftVersions.value = await apiService.getMinecraftVersions();
  } catch (err) {
    console.error('Failed to load Minecraft versions:', err);
  } finally {
    loadingVersions.value = false;
  }
};

const loadLoaders = async (version: string) => {
  if (!version) {
    availableLoaders.value = [];
    return;
  }
  loadingLoaders.value = true;
  try {
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
  loadingLoaderVersions.value = true;
  try {
    loaderVersions.value = await apiService.getLoaderVersions(version, loader);
  } catch (err) {
    console.error('Failed to load loader versions:', err);
    loaderVersions.value = [];
  } finally {
    loadingLoaderVersions.value = false;
  }
};

const handleEdit = async () => {
  setEditDataFromProps();
  await loadMinecraftVersions();
  isEditing.value = true;
};

const handleCancel = () => {
  isEditing.value = false;
  showDeleteConfirm.value = false;
  uploadedFiles.value = null;
  availableLoaders.value = [];
  loaderVersions.value = [];
  setEditDataFromProps();
};

const handleUpdate = async () => {
  updating.value = true;
  try {
    if (uploadedFiles.value && uploadedFiles.value.length > 0) {
      await apiService.uploadModpackFiles(props.modpack.id, uploadedFiles.value);
    }
    const payload = {
      ...editData,
      auth_config: { ...editData.auth_config },
    };
    await apiService.updateModpack(props.modpack.id, payload);
    emit('updated', { id: props.modpack.id, data: payload });
    handleCancel();
  } catch (err) {
    console.error('Failed to update modpack:', err);
  } finally {
    updating.value = false;
  }
};

const handleDelete = () => {
  emit('deleted', props.modpack.id);
  showDeleteConfirm.value = false;
};

const handleInputChange = (field: keyof EditableFields, value: string | LoaderType) => {
  (editData as Record<string, unknown>)[field] = value;
};

const handleAuthConfigChange = (field: keyof AuthConfig, value: string | AuthKind) => {
  editData.auth_config = {
    ...editData.auth_config,
    [field]: value,
    ...(field === 'kind'
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

watch(
  () => ({ editing: isEditing.value, version: editData.minecraft_version }),
  ({ editing, version }) => {
    if (editing && version) {
      loadLoaders(version).catch((err) => console.error(err));
    }
  },
);

watch(
  () => ({ editing: isEditing.value, version: editData.minecraft_version, loader: editData.loader }),
  ({ editing, version, loader }) => {
    if (editing && version && loader) {
      loadLoaderVersions(version, loader).catch((err) => console.error(err));
    }
  },
);

const authKindLabel = computed(() => editData.auth_config.kind);
</script>

<template>
  <div class="space-y-6 p-4">
    <Card>
      <CardHeader>
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <CardTitle>{{ isEditing ? 'Edit Modpack' : props.modpack.name }}</CardTitle>
            <CardDescription>
              {{
                isEditing
                  ? 'Update the configuration or upload new files.'
                  : 'Review the active configuration for this modpack.'
              }}
            </CardDescription>
          </div>
          <div v-if="!isEditing" class="flex flex-wrap gap-2">
            <Button size="sm" class="gap-2" @click="handleEdit">
              <Pencil class="h-4 w-4" />
              Update
            </Button>
            <Button v-if="!showDeleteConfirm" size="sm" variant="destructive" class="gap-2"
              @click="showDeleteConfirm = true">
              <Trash2 class="h-4 w-4" />
              Delete
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent class="space-y-6">
        <template v-if="isEditing">
          <form class="space-y-5" @submit.prevent="handleUpdate">
            <div class="grid gap-4 sm:grid-cols-2">
              <div class="space-y-2 sm:col-span-2">
                <Label for="edit-name">Modpack Name *</Label>
                <Input id="edit-name" :model-value="editData.name" :disabled="updating" placeholder="Enter a name"
                  @update:modelValue="(value) => handleInputChange('name', value?.toString() ?? '')" />
              </div>
              <div class="space-y-2">
                <Label>Minecraft Version *</Label>
                <Select :model-value="editData.minecraft_version || undefined" :disabled="loadingVersions"
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
                <p v-if="loadingVersions" class="text-sm">Loading versions...</p>
              </div>
              <div class="space-y-2">
                <Label>Mod Loader *</Label>
                <Select :model-value="editData.loader || undefined"
                  :disabled="loadingLoaders || !editData.minecraft_version || availableLoaders.length === 0"
                  @update:modelValue="(value) =>
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
                <p v-if="!editData.minecraft_version" class="text-sm">
                  Pick a Minecraft version first.
                </p>
                <p v-else-if="availableLoaders.length === 0" class="text-sm">
                  No loaders available.
                </p>
              </div>
              <div class="space-y-2">
                <Label>Loader Version *</Label>
                <Select :model-value="editData.loader_version || undefined"
                  :disabled="loadingLoaderVersions || !editData.loader || loaderVersions.length === 0"
                  @update:modelValue="(value) => handleInputChange('loader_version', value?.toString() ?? '')">
                  <SelectTrigger>
                    <SelectValue placeholder="Select loader version" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem v-for="version in loaderVersions" :key="version" :value="version">
                      {{ version }}
                    </SelectItem>
                  </SelectContent>
                </Select>
                <p v-if="!editData.loader" class="text-sm">
                  Select a loader first.
                </p>
                <p v-else-if="loaderVersions.length === 0" class="text-sm">
                  No versions for this loader.
                </p>
              </div>
              <div class="space-y-2">
                <Label>Authentication Type *</Label>
                <Select :model-value="editData.auth_config.kind"
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
              </div>
            </div>
            <div v-if="editData.auth_config.kind === AuthKind.TELEGRAM" class="space-y-2">
              <Label for="auth-base">Auth Base URL *</Label>
              <Input id="auth-base" type="url" :model-value="editData.auth_config.auth_base_url || ''"
                placeholder="https://your-telegram-auth-server.com"
                @update:modelValue="(value) => handleAuthConfigChange('auth_base_url', value?.toString() ?? '')" />
            </div>
            <div v-if="editData.auth_config.kind === AuthKind.ELY_BY" class="grid gap-4 sm:grid-cols-2">
              <div class="space-y-2">
                <Label for="client-id">Client ID *</Label>
                <Input id="client-id" :model-value="editData.auth_config.client_id || ''" placeholder="Client ID"
                  @update:modelValue="(value) => handleAuthConfigChange('client_id', value?.toString() ?? '')" />
              </div>
              <div class="space-y-2">
                <Label for="client-secret">Client Secret *</Label>
                <Input id="client-secret" type="password" :model-value="editData.auth_config.client_secret || ''"
                  placeholder="Client secret"
                  @update:modelValue="(value) => handleAuthConfigChange('client_secret', value?.toString() ?? '')" />
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
            <div class="flex flex-wrap justify-end gap-3">
              <Button type="button" :disabled="updating" @click="handleCancel">
                Cancel
              </Button>
              <Button type="submit" :disabled="updating">
                <span v-if="updating">Saving...</span>
                <span v-else>Save Changes</span>
              </Button>
            </div>
          </form>
        </template>
        <template v-else>
          <dl class="grid gap-4 sm:grid-cols-2">
            <div>
              <dt class="text-sm">Minecraft Version</dt>
              <dd class="text-sm font-medium">{{ props.modpack.minecraft_version }}</dd>
            </div>
            <div>
              <dt class="text-sm">Mod Loader</dt>
              <dd class="text-sm font-medium capitalize">{{ props.modpack.loader }}</dd>
            </div>
            <div>
              <dt class="text-sm">Loader Version</dt>
              <dd class="text-sm font-medium">{{ props.modpack.loader_version }}</dd>
            </div>
            <div>
              <dt class="text-sm">Authentication Type</dt>
              <dd class="text-sm font-medium capitalize">{{ authKindLabel }}</dd>
            </div>
            <div v-if="props.modpack.auth_config.kind === AuthKind.TELEGRAM && props.modpack.auth_config.auth_base_url"
              class="sm:col-span-2">
              <dt class="text-sm">Auth Base URL</dt>
              <dd class="text-sm font-medium wrap-break-word">{{ props.modpack.auth_config.auth_base_url }}</dd>
            </div>
            <template v-if="props.modpack.auth_config.kind === AuthKind.ELY_BY">
              <div>
                <dt class="text-sm">Client ID</dt>
                <dd class="text-sm font-medium wrap-break-word">{{ props.modpack.auth_config.client_id }}</dd>
              </div>
              <div>
                <dt class="text-sm">Client Secret</dt>
                <dd class="text-sm font-medium">••••••••••</dd>
              </div>
            </template>
          </dl>
        </template>
      </CardContent>
    </Card>
    <DeleteConfirmModal :is-open="showDeleteConfirm" :modpack-name="props.modpack.name" @confirm="handleDelete"
      @cancel="showDeleteConfirm = false" />
  </div>
</template>
