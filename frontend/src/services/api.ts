import { ModpackResponse, ModpackBase, SettingResponse } from '../types/api';
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
      // Handle unauthorized - trigger re-authentication
      if (this.handleUnauthorized) {
        this.handleUnauthorized();
      }
      throw new Error('Unauthorized - please login again');
    }

    if (!response.ok) {
      throw new Error(`API Error: ${response.status} ${response.statusText}`);
    }

    return response.json();
  }

  // Modpacks
  async getModpacks(): Promise<ModpackResponse[]> {
    return this.request<ModpackResponse[]>('/modpacks');
  }

  async getModpack(id: number): Promise<ModpackResponse> {
    return this.request<ModpackResponse>(`/modpacks/${id}`);
  }

  async createModpack(data: ModpackBase): Promise<ModpackResponse> {
    return this.request<ModpackResponse>('/modpacks', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async deleteModpack(id: number): Promise<void> {
    await this.request<void>(`/modpacks/${id}`, {
      method: 'DELETE',
    });
  }

  // Minecraft versions and loaders
  async getMinecraftVersions(): Promise<string[]> {
    return this.request<string[]>('/mc-versions');
  }

  async getLoadersForVersion(version: string): Promise<string[]> {
    return this.request<string[]>(`/mc-versions/${version}/loaders`);
  }

  async getLoaderVersions(version: string, loader: string): Promise<string[]> {
    return this.request<string[]>(`/mc-versions/${version}/${loader}`);
  }

  // Settings
  async getSettings(): Promise<SettingResponse[]> {
    return this.request<SettingResponse[]>('/settings');
  }

  async updateSettings(settings: SettingResponse[]): Promise<void> {
    await this.request<void>('/settings', {
      method: 'POST',
      body: JSON.stringify(settings),
    });
  }
}

export const apiService = new ApiService();