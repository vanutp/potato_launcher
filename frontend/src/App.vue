<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import LoginForm from '@/components/LoginForm.vue';
import ModpackSidebar from '@/components/ModpackSidebar.vue';
import ModpackForm from '@/components/ModpackForm.vue';
import ModpackDetails from '@/components/ModpackDetails.vue';
import SettingsForm from '@/components/SettingsForm.vue';
import NotificationToast from '@/components/NotificationToast.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/composables/useAuth';
import { useNotification } from '@/composables/useNotification';
import { useWebSocket } from '@/composables/useWebSocket';
import { apiService } from '@/services/api';
import type { ModpackBase, ModpackResponse, SettingResponse } from '@/types/api';

const { isAuthenticated, loading: authLoading, error: authError, login, logout } = useAuth();
const { notification, hideNotification, showSuccess, showError, showInfo } = useNotification();

const modpacks = ref<ModpackResponse[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const selectedModpack = ref<number | null>(null);
const showForm = ref(false);
const showSettings = ref(false);
const building = ref(false);
const fetching = ref(false);

const selectedModpackData = computed(() =>
  modpacks.value.find((m) => m.id === selectedModpack.value) ?? null,
);

const loadModpacks = async () => {
  if (fetching.value) return;
  fetching.value = true;
  try {
    loading.value = true;
    error.value = null;
    modpacks.value = await apiService.getModpacks();
    if (selectedModpack.value) {
      const exists = modpacks.value.some((m) => m.id === selectedModpack.value);
      if (!exists) {
        selectedModpack.value = null;
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load modpacks';
  } finally {
    loading.value = false;
    fetching.value = false;
  }
};

onMounted(() => {
  apiService.setUnauthorizedHandler(() => {
    logout();
  });
});

watch(
  isAuthenticated,
  (authed) => {
    if (authed) {
      loadModpacks().catch((err) => console.error(err));
    } else {
      modpacks.value = [];
      selectedModpack.value = null;
      showForm.value = false;
      showSettings.value = false;
      loading.value = false;
      error.value = null;
    }
  },
  { immediate: true },
);

useWebSocket({
  enabled: isAuthenticated,
  onModpackChange: () => {
    loadModpacks().catch((err) => console.error(err));
  },
  onNotification: (data) => {
    if (data && typeof data === 'object' && 'message' in data) {
      showInfo((data as { message: string }).message);
    }
  },
});

const handleLogin = async (payload: { token: string }) => {
  try {
    await login(payload);
    showSuccess('Logged in successfully');
  } catch (err) {
    showError(err instanceof Error ? err.message : 'Login failed');
  }
};

const handleNewModpack = () => {
  showForm.value = true;
  showSettings.value = false;
  selectedModpack.value = null;
};

const handleSelectModpack = (id: number) => {
  selectedModpack.value = id;
  showForm.value = false;
  showSettings.value = false;
};

const handleShowSettings = () => {
  showSettings.value = true;
  showForm.value = false;
  selectedModpack.value = null;
};

const handleModpackUpdate = (payload: { id: number; data: Partial<ModpackResponse> }) => {
  modpacks.value = modpacks.value.map((modpack) =>
    modpack.id === payload.id ? { ...modpack, ...payload.data } : modpack,
  );
};

const handleModpackDelete = async (id: number) => {
  try {
    await apiService.deleteModpack(id);
    modpacks.value = modpacks.value.filter((modpack) => modpack.id !== id);
    if (selectedModpack.value === id) {
      selectedModpack.value = null;
    }
  } catch (err) {
    console.error('Failed to delete modpack:', err);
    await loadModpacks();
  }
};

const handleFormSubmit = async (_: ModpackBase) => {
  await loadModpacks();
  showForm.value = false;
  showSettings.value = false;
};

const handleSettingsSave = (settings: SettingResponse[]) => {
  showSuccess('Settings saved successfully');
  showSettings.value = false;
};

const handleBuild = async () => {
  try {
    building.value = true;
    await apiService.buildModpacks();
    showSuccess('Build started successfully!');
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Build failed';
    showError(`Build failed: ${message}`);
  } finally {
    building.value = false;
  }
};
</script>

<template>
  <div>
    <NotificationToast :type="notification.type" :message="notification.message" :is-visible="notification.isVisible"
      @close="hideNotification" />

    <LoginForm v-if="!isAuthenticated" :loading="authLoading" :error="authError" @login="handleLogin" />

    <div v-else class="flex min-h-screen">
      <ModpackSidebar :modpacks="modpacks" :selected-modpack="selectedModpack" :show-form="showForm"
        :show-settings="showSettings" :building="building" @select="handleSelectModpack" @new="handleNewModpack"
        @show-settings="handleShowSettings" @logout="logout" @build="handleBuild" />

      <main class="flex-1 p-8">
        <div v-if="loading" class="min-h-[60vh] flex items-center justify-center">
          <div class="text-xl">Loading...</div>
        </div>

        <div v-else-if="error" class="min-h-[60vh] flex items-center justify-center">
          <div class="text-center">
            <div class="text-xl mb-4">
              Error: {{ error }}
            </div>
            <Button @click="loadModpacks">
              Retry
            </Button>
          </div>
        </div>

        <div v-else>
          <ModpackForm v-if="showForm" @submitted="handleFormSubmit" />
          <SettingsForm v-else-if="showSettings" @saved="handleSettingsSave" />
          <ModpackDetails v-else-if="selectedModpackData" :key="selectedModpackData.id" :modpack="selectedModpackData"
            @updated="handleModpackUpdate" @deleted="handleModpackDelete" />
          <div v-else class="flex items-center justify-center min-h-[60vh] p-4">
            <Card class="w-full max-w-xl">
              <CardHeader class="text-center">
                <CardTitle class="text-2xl">Welcome to Modpack Manager</CardTitle>
                <CardDescription>
                  Select a modpack from the sidebar or create a new one to get started.
                </CardDescription>
              </CardHeader>
              <CardContent class="text-center">
                <Button size="lg" @click="handleNewModpack">
                  Create New Modpack
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
    </div>
  </div>
</template>
