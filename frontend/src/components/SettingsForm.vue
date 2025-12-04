<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { Save } from 'lucide-vue-next';
import { apiService } from '@/services/api';
import type { SettingResponse } from '@/types/api';
import { SettingType } from '@/types/api';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';

const emit = defineEmits<{
  (event: 'saved', payload: SettingResponse[]): void;
}>();

const defaultSettings: SettingResponse[] = [
  { key: 'download_server_base', value: '', type: SettingType.STRING },
  { key: 'resources_url_base', value: '', type: SettingType.STRING },
  { key: 'replace_download_urls', value: false, type: SettingType.BOOLEAN },
  { key: 'version_manifest_url', value: '', type: SettingType.STRING },
];

const displaySettings = ref<SettingResponse[]>([]);
const originalSettings = ref<SettingResponse[]>([]);
const loading = ref(true);
const saving = ref(false);
const error = ref<string | null>(null);

const mergedDefaultSettings = () =>
  defaultSettings.map((setting) => {
    const serverValue = originalSettings.value.find((s) => s.key === setting.key);
    return serverValue ?? setting;
  });

const hasChanges = computed(() =>
  displaySettings.value.some((setting) => {
    const original = originalSettings.value.find((s) => s.key === setting.key);
    const fallback = defaultSettings.find((s) => s.key === setting.key);
    const originalValue = original?.value ?? fallback?.value;
    return setting.value !== originalValue;
  }),
);

const getSetting = (key: string) => displaySettings.value.find((s) => s.key === key);

const loadSettings = async () => {
  try {
    loading.value = true;
    error.value = null;
    const data = await apiService.getSettings();
    originalSettings.value = data;
    displaySettings.value = mergedDefaultSettings().map((setting) => {
      const serverSetting = data.find((s) => s.key === setting.key);
      return serverSetting ?? setting;
    });
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load settings';
    displaySettings.value = mergedDefaultSettings();
  } finally {
    loading.value = false;
  }
};

const handleInputChange = (key: string, value: string | boolean) => {
  displaySettings.value = displaySettings.value.map((setting) =>
    setting.key === key ? { ...setting, value } : setting,
  );
};

const handleSave = async () => {
  try {
    saving.value = true;
    error.value = null;

    const changedSettings = displaySettings.value.filter((setting) => {
      const original = originalSettings.value.find((s) => s.key === setting.key);
      const fallback = defaultSettings.find((s) => s.key === setting.key);
      const originalValue = original?.value ?? fallback?.value;
      return setting.value !== originalValue;
    });

    await apiService.updateSettings(changedSettings);
    originalSettings.value = displaySettings.value.map((setting) => ({ ...setting }));
    emit('saved', displaySettings.value);
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to save settings';
  } finally {
    saving.value = false;
  }
};

onMounted(() => {
  loadSettings().catch((err) => console.error(err));
});
</script>

<template>
  <div class="p-4">
    <Card>
      <CardHeader>
        <CardTitle>Application Settings</CardTitle>
        <CardDescription>Update launcher URLs and deployment toggles.</CardDescription>
      </CardHeader>
      <CardContent>
        <div v-if="loading" class="text-sm">Loading settings...</div>
        <template v-else>
          <Alert v-if="error" variant="destructive" class="mb-4">
            <AlertDescription class="flex items-center justify-between gap-2">
              <span>{{ error }}</span>
              <Button size="sm" @click="loadSettings">
                Retry
              </Button>
            </AlertDescription>
          </Alert>
          <div v-if="displaySettings.length" class="space-y-6">
            <div v-if="getSetting('download_server_base')" class="space-y-2">
              <Label for="download_server_base">Download Server Base URL</Label>
              <Input id="download_server_base" :model-value="getSetting('download_server_base')?.value as string | ''"
                placeholder="https://your-server.com"
                @update:modelValue="(value) => handleInputChange('download_server_base', value?.toString() ?? '')" />
              <p class="text-sm">
                Base URL used when generating download links.
              </p>
            </div>
            <div v-if="getSetting('resources_url_base')" class="space-y-2">
              <Label for="resources_url_base">Resources URL Base</Label>
              <Input id="resources_url_base" :model-value="getSetting('resources_url_base')?.value as string | ''"
                placeholder="https://your-server.com/assets/objects"
                @update:modelValue="(value) => handleInputChange('resources_url_base', value?.toString() ?? '')" />
              <p class="text-sm">
                Leave blank to fall back to Mojang asset endpoints.
              </p>
            </div>
            <div v-if="getSetting('replace_download_urls')" class="space-y-2">
              <Label>Replace Download URLs</Label>
              <Select :model-value="getSetting('replace_download_urls')?.value?.toString() ?? 'false'"
                @update:modelValue="(value) => handleInputChange('replace_download_urls', value === 'true')">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="false">false</SelectItem>
                  <SelectItem value="true">true</SelectItem>
                </SelectContent>
              </Select>
              <p class="text-sm">
                Toggle to redirect every download through your infrastructure.
              </p>
            </div>
            <div v-if="getSetting('version_manifest_url')" class="space-y-2">
              <Label for="version_manifest_url">Version Manifest URL</Label>
              <Input id="version_manifest_url" :model-value="getSetting('version_manifest_url')?.value as string | ''"
                placeholder="https://your-server.com/version_manifest.json"
                @update:modelValue="(value) => handleInputChange('version_manifest_url', value?.toString() ?? '')" />
              <p class="text-sm">
                Merge remote manifest entries with locally defined versions.
              </p>
            </div>
            <div>
              <Button class="gap-2" :disabled="!hasChanges || saving" @click="handleSave">
                <Save class="h-4 w-4" />
                {{ saving ? 'Saving...' : 'Save Settings' }}
              </Button>
              <p v-if="hasChanges" class="mt-2 text-sm">
                Changes are local until you save.
              </p>
            </div>
          </div>
        </template>
      </CardContent>
    </Card>
  </div>
</template>
