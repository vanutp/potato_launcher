import { ref } from 'vue';
import { authService } from '@/services/auth';
import type { TokenRequest } from '@/types/auth';

const isAuthenticated = ref(authService.isAuthenticated());
const loading = ref(false);
const error = ref<string | null>(null);

export function useAuth() {
  const login = async (tokenRequest: TokenRequest) => {
    try {
      loading.value = true;
      error.value = null;
      await authService.login(tokenRequest);
      isAuthenticated.value = true;
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed';
      isAuthenticated.value = false;
      throw err;
    } finally {
      loading.value = false;
    }
  };

  const logout = () => {
    authService.logout();
    isAuthenticated.value = false;
    error.value = null;
  };

  return {
    isAuthenticated,
    loading,
    error,
    login,
    logout,
  };
}

