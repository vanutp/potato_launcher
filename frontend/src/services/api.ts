import type { AuthBackend, InstanceBase, InstanceResponse, SettingResponse } from '@/types/api';
import { authService } from './auth';

const API_BASE = import.meta.env.VITE_API_BASE_URL
  ? `${import.meta.env.VITE_API_BASE_URL}/api/v1`
  : '/api/v1';

class ApiService {
  private handleUnauthorized?: () => void;

  setUnauthorizedHandler(handler: () => void) {
    this.handleUnauthorized = handler;
  }

  private async request<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const authHeaders = authService.getAuthHeaders();

    const response = await fetch(`${API_BASE}${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...authHeaders,
        ...options?.headers,
      },
      ...options,
    });

    if (response.status === 401) {
      if (this.handleUnauthorized) {
        this.handleUnauthorized();
      }
      throw new Error('Unauthorized - please login again');
    }

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    if (response.status === 204) {
      return undefined as T;
    }

    return response.json();
  }

  async getInstances(): Promise<InstanceResponse[]> {
    return this.request<InstanceResponse[]>('/instances');
  }

  async getInstance(name: string): Promise<InstanceResponse> {
    const encoded = encodeURIComponent(name);
    return this.request<InstanceResponse>(`/instances/${encoded}`);
  }

  async createInstance(data: InstanceBase): Promise<InstanceResponse> {
    return this.request<InstanceResponse>('/instances', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async deleteInstance(name: string): Promise<void> {
    const encoded = encodeURIComponent(name);
    await this.request<void>(`/instances/${encoded}`, {
      method: 'DELETE',
    });
  }

  async updateInstance(
    name: string,
    data: Partial<InstanceBase & { auth_backend: AuthBackend }>,
  ): Promise<InstanceResponse> {
    const encoded = encodeURIComponent(name);
    return this.request<InstanceResponse>(`/instances/${encoded}`, {
      method: 'PATCH',
      body: JSON.stringify(data),
    });
  }

  async uploadInstanceFiles(name: string, files: FileList): Promise<void> {
    const formData = new FormData();
    Array.from(files).forEach((file) => {
      const path = file.webkitRelativePath || file.name;
      formData.append('files', file, path);
    });

    const authHeaders = authService.getAuthHeaders();
    const encoded = encodeURIComponent(name);
    const response = await fetch(`${API_BASE}/instances/${encoded}/files`, {
      method: 'POST',
      headers: {
        ...authHeaders,
      },
      body: formData,
    });

    if (response.status === 401) {
      if (this.handleUnauthorized) {
        this.handleUnauthorized();
      }
      throw new Error('Unauthorized - please login again');
    }

    if (!response.ok) {
      throw new Error(`File upload failed: ${response.status} ${response.statusText}`);
    }
  }

  async getMinecraftVersions(): Promise<string[]> {
    return this.request<string[]>('/mc-versions');
  }

  async getLoadersForVersion(version: string): Promise<string[]> {
    return this.request<string[]>(`/mc-versions/${version}/loaders`);
  }

  async getLoaderVersions(version: string, loader: string): Promise<string[]> {
    return this.request<string[]>(`/mc-versions/${version}/${loader}`);
  }

  async getSettings(): Promise<SettingResponse[]> {
    return this.request<SettingResponse[]>('/settings');
  }

  async updateSettings(settings: SettingResponse[]): Promise<void> {
    await this.request<void>('/settings', {
      method: 'POST',
      body: JSON.stringify(settings),
    });
  }

  async buildInstances(): Promise<void> {
    await this.request<void>('/instances/build', {
      method: 'POST',
    });
  }
}

export const apiService = new ApiService();
