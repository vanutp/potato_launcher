<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { Pencil, Trash2 } from 'lucide-vue-next';
import DeleteConfirmModal from './DeleteConfirmModal.vue';
import { apiService } from '@/services/api';
import type { AuthBackend, ModpackBase, ModpackResponse } from '@/types/api';
import { AuthType, LoaderType } from '@/types/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useModpackForm } from '@/composables/useModpackForm';
import ModpackFormFields from '@/components/ModpackFormFields.vue';

const props = defineProps<{
  modpack: ModpackResponse;
}>();

const emit = defineEmits<{
  (event: 'updated', payload: { id: number; data: Partial<ModpackResponse> }): void;
  (event: 'deleted', id: number): void;
}>();

type EditableFields = ModpackBase;

const toEditableFields = (modpack: ModpackResponse): EditableFields => ({
  name: modpack.name,
  minecraft_version: modpack.minecraft_version,
  loader_name: modpack.loader_name,
  loader_version: modpack.loader_version,
  auth_backend: { ...modpack.auth_backend },
});

const isEditing = ref(false);
const showDeleteConfirm = ref(false);
const updating = ref(false);

const guard = computed(() => isEditing.value);

const {
  formData: editData,
  minecraftVersions,
  availableLoaders,
  loaderVersions,
  loadingMinecraftVersions,
  loadingLoaders,
  loadingLoaderVersions,
  uploadedFiles,
  handleInputChange: setFieldValue,
  handleAuthBackendChange: setAuthFieldValue,
  handleDrag,
  handleDrop,
  handleFileInput,
  loadMinecraftVersions,
  resetFormData,
  resetUploads,
} = useModpackForm({
  initialData: toEditableFields(props.modpack),
  guard,
  mode: 'edit',
});

const setEditDataFromProps = () => {
  resetFormData(toEditableFields(props.modpack));
};

watch(
  () => props.modpack.id,
  () => {
    isEditing.value = false;
    showDeleteConfirm.value = false;
    resetUploads();
    minecraftVersions.value = [];
    availableLoaders.value = [];
    loaderVersions.value = [];
    setEditDataFromProps();
  },
);

const handleEdit = async () => {
  setEditDataFromProps();
  await loadMinecraftVersions();
  isEditing.value = true;
};

const handleCancel = () => {
  isEditing.value = false;
  showDeleteConfirm.value = false;
  resetUploads();
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
    const payload: ModpackBase = {
      ...editData,
      auth_backend: { ...editData.auth_backend },
    };

    if (payload.loader_name === LoaderType.VANILLA) {
      delete payload.loader_version;
    }

    const updated = await apiService.updateModpack(props.modpack.id, payload);
    emit('updated', { id: props.modpack.id, data: updated });
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

const updateField = (field: keyof EditableFields, value: string | LoaderType) => {
  setFieldValue(field, value);
};

const updateAuthField = (field: keyof AuthBackend, value: string | AuthType) => {
  setAuthFieldValue(field, value);
};

const authTypeLabel = computed(() => editData.auth_backend.type);
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
            <ModpackFormFields id-prefix="edit" :form-data="editData" :minecraft-versions="minecraftVersions"
              :available-loaders="availableLoaders" :loader-versions="loaderVersions"
              :loading-minecraft-versions="loadingMinecraftVersions" :loading-loaders="loadingLoaders"
              :loading-loader-versions="loadingLoaderVersions" :uploaded-files="uploadedFiles" :disabled="updating"
              @update-field="updateField" @update-auth-field="updateAuthField" @file-drag="handleDrag"
              @file-drop="handleDrop" @file-input="handleFileInput" />
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
              <dd class="text-sm font-medium capitalize">{{ props.modpack.loader_name }}</dd>
            </div>
            <div>
              <dt class="text-sm">Loader Version</dt>
              <dd class="text-sm font-medium">{{ props.modpack.loader_version }}</dd>
            </div>
            <div>
              <dt class="text-sm">Authentication Type</dt>
              <dd class="text-sm font-medium capitalize">{{ authTypeLabel }}</dd>
            </div>
            <div
              v-if="props.modpack.auth_backend.type === AuthType.TELEGRAM && props.modpack.auth_backend.auth_base_url"
              class="sm:col-span-2">
              <dt class="text-sm">Auth Base URL</dt>
              <dd class="text-sm font-medium wrap-break-word">{{ props.modpack.auth_backend.auth_base_url }}</dd>
            </div>
            <template v-if="props.modpack.auth_backend.type === AuthType.ELY_BY">
              <div>
                <dt class="text-sm">Client ID</dt>
                <dd class="text-sm font-medium wrap-break-word">{{ props.modpack.auth_backend.client_id }}</dd>
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
