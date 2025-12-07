<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue';
import { apiService } from '@/services/api';
import type { AuthBackend, ModpackBase } from '@/types/api';
import { AuthType, LoaderType } from '@/types/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useModpackForm } from '@/composables/useModpackForm';
import ModpackFormFields from '@/components/ModpackFormFields.vue';

const emit = defineEmits<{
  (event: 'submitted', payload: ModpackBase): void;
}>();

const {
  formData,
  minecraftVersions,
  availableLoaders,
  loaderVersions,
  loadingMinecraftVersions,
  loadingLoaders,
  loadingLoaderVersions,
  uploadedFiles,
  handleInputChange,
  handleAuthBackendChange,
  handleDrag,
  handleDrop,
  handleFileInput,
  loadMinecraftVersions,
  resetFormData,
  resetUploads,
} = useModpackForm({ mode: 'create' });

const loading = ref(false);
const errors = reactive<Record<string, string>>({});

const validate = () => {
  const newErrors: Record<string, string> = {};
  if (!formData.name.trim()) newErrors.name = 'Name is required';
  if (!formData.minecraft_version) newErrors.minecraft_version = 'Minecraft version is required';
  if (!formData.loader_name) newErrors.loader_name = 'Loader is required';
  if (formData.loader_name !== LoaderType.VANILLA && !formData.loader_version) {
    newErrors.loader_version = 'Loader version is required';
  }
  if (!formData.auth_backend.type) newErrors.auth_type = 'Authentication type is required';

  if (formData.auth_backend.type === AuthType.TELEGRAM && !formData.auth_backend.auth_base_url?.trim()) {
    newErrors.auth_base_url = 'Auth base URL is required for Telegram';
  }

  if (formData.auth_backend.type === AuthType.ELY_BY) {
    if (!formData.auth_backend.client_id?.trim()) {
      newErrors.client_id = 'Client ID is required for Ely.by';
    }
    if (!formData.auth_backend.client_secret?.trim()) {
      newErrors.client_secret = 'Client Secret is required for Ely.by';
    }
  }

  Object.keys(errors).forEach((key) => delete errors[key]);
  Object.assign(errors, newErrors);

  return Object.keys(newErrors).length === 0;
};

const resetForm = () => {
  resetFormData();
  resetUploads();
};

const handleSubmit = async () => {
  if (!validate()) {
    return;
  }

  try {
    loading.value = true;
    const payload: ModpackBase = {
      ...formData,
      auth_backend: { ...formData.auth_backend },
    };

    if (payload.loader_name === LoaderType.VANILLA) {
      delete payload.loader_version;
    }

    const created = await apiService.createModpack(payload);
    if (uploadedFiles.value && uploadedFiles.value.length > 0) {
      await apiService.uploadModpackFiles(created.id, uploadedFiles.value);
    }
    emit('submitted', payload);
    resetForm();
  } catch (err) {
    errors.submit = err instanceof Error ? err.message : 'Failed to create modpack';
  } finally {
    loading.value = false;
  }
};

const updateField = (field: keyof ModpackBase, value: string | LoaderType) => {
  handleInputChange(field, value);
  if (errors[field as string]) {
    delete errors[field as string];
  }
};

const updateAuthField = (field: keyof AuthBackend, value: string | AuthType) => {
  handleAuthBackendChange(field, value);
  if (errors[field as string]) {
    delete errors[field as string];
  }
};

onMounted(() => {
  loadMinecraftVersions().catch((err) => console.error(err));
});
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
          <ModpackFormFields id-prefix="create" :form-data="formData" :errors="errors"
            :minecraft-versions="minecraftVersions" :available-loaders="availableLoaders"
            :loader-versions="loaderVersions" :loading-minecraft-versions="loadingMinecraftVersions"
            :loading-loaders="loadingLoaders" :loading-loader-versions="loadingLoaderVersions"
            :uploaded-files="uploadedFiles" @update-field="updateField" @update-auth-field="updateAuthField"
            @file-drag="handleDrag" @file-drop="handleDrop" @file-input="handleFileInput" />
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
