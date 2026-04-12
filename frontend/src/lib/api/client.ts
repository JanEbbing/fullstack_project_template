import { auth, clearAuth } from '$lib/stores/auth.svelte';

const BASE_URL = '/api/v1';

class ApiClient {
    private async request<T>(method: string, path: string, body?: unknown): Promise<T> {
        const headers: Record<string, string> = {
            'Content-Type': 'application/json',
        };
        if (auth.accessToken) {
            headers['Authorization'] = `Bearer ${auth.accessToken}`;
        }

        const response = await fetch(`${BASE_URL}${path}`, {
            method,
            headers,
            body: body ? JSON.stringify(body) : undefined,
        });

        if (response.status === 401 && auth.accessToken) {
            const refreshed = await this.attemptRefresh();
            if (refreshed) {
                headers['Authorization'] = `Bearer ${auth.accessToken}`;
                const retryResponse = await fetch(`${BASE_URL}${path}`, {
                    method,
                    headers,
                    body: body ? JSON.stringify(body) : undefined,
                });
                if (!retryResponse.ok) {
                    throw await this.parseError(retryResponse);
                }
                return retryResponse.json();
            }
            clearAuth();
            window.location.href = '/login';
            throw new Error('Session expired');
        }

        if (!response.ok) {
            throw await this.parseError(response);
        }
        return response.json();
    }

    private async attemptRefresh(): Promise<boolean> {
        const refreshToken = localStorage.getItem('refresh_token');
        if (!refreshToken) return false;
        try {
            const res = await fetch(`${BASE_URL}/auth/refresh`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ refresh_token: refreshToken }),
            });
            if (!res.ok) return false;
            const data = await res.json();
            auth.accessToken = data.access_token;
            localStorage.setItem('refresh_token', data.refresh_token);
            return true;
        } catch {
            return false;
        }
    }

    private async parseError(response: Response): Promise<Error> {
        try {
            const data = await response.json();
            return new Error(data.error || 'Unknown error');
        } catch {
            return new Error(`HTTP ${response.status}`);
        }
    }

    get<T>(path: string): Promise<T> {
        return this.request<T>('GET', path);
    }

    post<T>(path: string, body?: unknown): Promise<T> {
        return this.request<T>('POST', path, body);
    }

    put<T>(path: string, body?: unknown): Promise<T> {
        return this.request<T>('PUT', path, body);
    }

    delete<T>(path: string): Promise<T> {
        return this.request<T>('DELETE', path);
    }
}

export const apiClient = new ApiClient();
