import type { AuthResponse, TokenRequest } from '@/types/auth';

class AuthService {
  private token: string | null = null;

  constructor() {
    this.token = localStorage.getItem('auth_token');
  }

  async login(tokenRequest: TokenRequest): Promise<AuthResponse> {
    const response = await fetch(`${import.meta.env.VITE_API_BASE_URL!}/auth/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(tokenRequest),
    });

    if (!response.ok) {
      throw new Error(`Login failed: ${response.status} ${response.statusText}`);
    }

    const authResponse: AuthResponse = await response.json();
    this.token = authResponse.access_token;
    localStorage.setItem('auth_token', authResponse.access_token);

    return authResponse;
  }

  logout(): void {
    this.token = null;
    localStorage.removeItem('auth_token');
  }

  getToken(): string | null {
    return this.token;
  }

  isAuthenticated(): boolean {
    return this.token !== null;
  }

  getAuthHeaders(): Record<string, string> {
    if (this.token) {
      return {
        Authorization: `Bearer ${this.token}`,
      };
    }
    return {};
  }
}

export const authService = new AuthService();

