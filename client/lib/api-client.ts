import { ProvidersResponse, QuoteRequest, QuoteResponse } from "./types";

const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8099";

export class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_URL) {
    this.baseUrl = baseUrl;
  }

  private getAuthHeaders(): HeadersInit {
    const headers: HeadersInit = {
      "Content-Type": "application/json",
    };

    // Get token from localStorage
    if (typeof window !== "undefined") {
      const token = localStorage.getItem("auth_token");
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }
    }

    return headers;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (response.status === 401) {
      // Token expired or invalid - redirect to login
      if (typeof window !== "undefined") {
        localStorage.removeItem("auth_token");
        localStorage.removeItem("auth_user");
        window.location.href = "/login";
      }
      throw new Error("Oturum süresi doldu. Lütfen tekrar giriş yapın.");
    }

    if (!response.ok) {
      const error = await response
        .json()
        .catch(() => ({ error: { message: "Bilinmeyen hata" } }));
      throw new Error(error.error?.message || "İşlem başarısız");
    }

    return response.json();
  }

  async getProviders(): Promise<ProvidersResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/providers`, {
      headers: this.getAuthHeaders(),
    });
    return this.handleResponse<ProvidersResponse>(response);
  }

  async requestQuote(
    provider: string,
    request: QuoteRequest
  ): Promise<QuoteResponse> {
    const response = await fetch(`${this.baseUrl}/api/v1/quote/${provider}`, {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(request),
    });
    return this.handleResponse<QuoteResponse>(response);
  }

  async requestAllQuotes(request: QuoteRequest): Promise<QuoteResponse[]> {
    const response = await fetch(`${this.baseUrl}/api/v1/quote`, {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(request),
    });
    return this.handleResponse<QuoteResponse[]>(response);
  }

  async checkHealth(): Promise<{ ok: boolean }> {
    const response = await fetch(`${this.baseUrl}/health`);
    return this.handleResponse<{ ok: boolean }>(response);
  }

  // Admin endpoints
  async getUsers(limit = 50, offset = 0) {
    const response = await fetch(
      `${this.baseUrl}/api/v1/admin/users?limit=${limit}&offset=${offset}`,
      {
        headers: this.getAuthHeaders(),
      }
    );
    return this.handleResponse(response);
  }

  async getActivityLogs(limit = 100, offset = 0) {
    const response = await fetch(
      `${this.baseUrl}/api/v1/admin/logs?limit=${limit}&offset=${offset}`,
      {
        headers: this.getAuthHeaders(),
      }
    );
    return this.handleResponse(response);
  }

  async getAdminStats() {
    const response = await fetch(`${this.baseUrl}/api/v1/admin/stats`, {
      headers: this.getAuthHeaders(),
    });
    return this.handleResponse(response);
  }

  // User quotes
  async getQuotes() {
    const response = await fetch(`${this.baseUrl}/api/v1/quotes`, {
      headers: this.getAuthHeaders(),
    });
    return this.handleResponse(response);
  }

  // User policies
  async getPolicies() {
    const response = await fetch(`${this.baseUrl}/api/v1/policies`, {
      headers: this.getAuthHeaders(),
    });
    return this.handleResponse(response);
  }

  // User profile
  async updateProfile(data: { name?: string; phone?: string }) {
    const response = await fetch(`${this.baseUrl}/api/v1/users/profile`, {
      method: "PUT",
      headers: { ...this.getAuthHeaders(), "Content-Type": "application/json" },
      body: JSON.stringify(data),
    });
    return this.handleResponse(response);
  }

  // Change password
  async changePassword(currentPassword: string, newPassword: string) {
    const response = await fetch(`${this.baseUrl}/api/v1/users/password`, {
      method: "PUT",
      headers: { ...this.getAuthHeaders(), "Content-Type": "application/json" },
      body: JSON.stringify({
        current_password: currentPassword,
        new_password: newPassword,
      }),
    });

    // 204 No Content doesn't have a body
    if (response.status === 204) {
      return;
    }

    return this.handleResponse(response);
  }
}

export const apiClient = new ApiClient();
