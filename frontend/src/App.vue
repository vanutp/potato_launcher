<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import LoginForm from '@/components/LoginForm.vue';
import InstanceSidebar from '@/components/InstanceSidebar.vue';
import InstanceForm from '@/components/InstanceForm.vue';
import InstanceDetails from '@/components/InstanceDetails.vue';
import SettingsForm from '@/components/SettingsForm.vue';
import NotificationToast from '@/components/NotificationToast.vue';
import BuildLogs from '@/components/BuildLogs.vue';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { useAuth } from '@/composables/useAuth';
import { useNotification } from '@/composables/useNotification';
import { useWebSocket } from '@/composables/useWebSocket';
import { apiService } from '@/services/api';
import type { InstanceBase, InstanceResponse, SettingResponse } from '@/types/api';

const { isAuthenticated, loading: authLoading, error: authError, login, logout } = useAuth();
const { notification, hideNotification, showSuccess, showError, showInfo } = useNotification();

const instances = ref<InstanceResponse[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);
const selectedInstance = ref<string | null>(null);
const showForm = ref(false);
const showSettings = ref(false);
const building = ref(false);
const fetching = ref(false);
const showLogs = ref(false);

const selectedInstanceData = computed(() =>
  instances.value.find((m) => m.name === selectedInstance.value) ?? null,
);

const loadInstances = async () => {
  if (fetching.value) return;
  fetching.value = true;
  try {
    loading.value = true;
    error.value = null;
    instances.value = await apiService.getInstances();
    if (selectedInstance.value) {
      const exists = instances.value.some((m) => m.name === selectedInstance.value);
      if (!exists) {
        selectedInstance.value = null;
      }
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to load instances';
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
      loadInstances().catch((err) => console.error(err));
    } else {
      instances.value = [];
      selectedInstance.value = null;
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
  onInstanceChange: () => {
    loadInstances().catch((err) => console.error(err));
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

const handleNewInstance = () => {
  showForm.value = true;
  showSettings.value = false;
  selectedInstance.value = null;
};

const handleSelectInstance = (name: string) => {
  selectedInstance.value = name;
  showForm.value = false;
  showSettings.value = false;
};

const handleShowSettings = () => {
  showSettings.value = true;
  showForm.value = false;
  selectedInstance.value = null;
};

const handleInstanceUpdate = (payload: { name: string; data: Partial<InstanceResponse> }) => {
  instances.value = instances.value.map((instance) =>
    instance.name === payload.name ? { ...instance, ...payload.data } : instance,
  );
};

const handleInstanceDelete = async (name: string) => {
  try {
    await apiService.deleteInstance(name);
    instances.value = instances.value.filter((instance) => instance.name !== name);
    if (selectedInstance.value === name) {
      selectedInstance.value = null;
    }
  } catch (err) {
    console.error('Failed to delete instance:', err);
    showError(err instanceof Error ? err.message : 'Failed to delete instance');
    await loadInstances();
  }
};

const handleFormSubmit = async (_: InstanceBase) => {
  await loadInstances();
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
    showLogs.value = true; // Auto-open logs on build
    await apiService.buildInstances();
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
      <InstanceSidebar :instances="instances" :selected-instance="selectedInstance" :show-form="showForm"
        :show-settings="showSettings" :building="building" @select="handleSelectInstance" @new="handleNewInstance"
        @show-settings="handleShowSettings" @logout="logout" @build="handleBuild" @show-logs="showLogs = true" />

      <main class="flex-1 p-8">
        <div v-if="loading" class="min-h-[60vh] flex items-center justify-center">
          <div class="text-xl">Loading...</div>
        </div>

        <div v-else-if="error" class="min-h-[60vh] flex items-center justify-center">
          <div class="text-center">
            <div class="text-xl mb-4">
              Error: {{ error }}
            </div>
            <Button @click="loadInstances">
              Retry
            </Button>
          </div>
        </div>

        <div v-else>
          <InstanceForm v-if="showForm" @submitted="handleFormSubmit" />
          <SettingsForm v-else-if="showSettings" @saved="handleSettingsSave" />
          <InstanceDetails v-else-if="selectedInstanceData" :key="selectedInstanceData.name"
            :instance="selectedInstanceData" @updated="handleInstanceUpdate" @deleted="handleInstanceDelete" />
          <div v-else class="flex items-center justify-center min-h-[60vh] p-4">
            <Card class="w-full max-w-xl">
              <CardHeader class="text-center">
                <CardTitle class="text-2xl">Welcome to Instance Manager</CardTitle>
                <CardDescription>
                  Select an instance from the sidebar or create a new one to get started.
                </CardDescription>
              </CardHeader>
              <CardContent class="text-center">
                <Button size="lg" @click="handleNewInstance">
                  Create New Instance
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </main>
    </div>

    <BuildLogs :is-open="showLogs" @update:open="showLogs = $event" />
  </div>
</template>
