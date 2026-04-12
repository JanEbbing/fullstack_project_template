import type { User } from '$lib/types';

interface AuthState {
    user: User | null;
    accessToken: string | null;
    isAuthenticated: boolean;
    isLoading: boolean;
}

export const auth: AuthState = $state({
    user: null,
    accessToken: null,
    isAuthenticated: false,
    isLoading: true,
});

export function setAuth(user: User, accessToken: string, refreshToken: string) {
    auth.user = user;
    auth.accessToken = accessToken;
    auth.isAuthenticated = true;
    auth.isLoading = false;
    localStorage.setItem('refresh_token', refreshToken);
}

export function clearAuth() {
    auth.user = null;
    auth.accessToken = null;
    auth.isAuthenticated = false;
    auth.isLoading = false;
    localStorage.removeItem('refresh_token');
}

export async function initAuth() {
    const refreshToken = localStorage.getItem('refresh_token');
    if (!refreshToken) {
        auth.isLoading = false;
        return;
    }
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 5000);
    try {
        const response = await fetch('/api/v1/auth/refresh', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ refresh_token: refreshToken }),
            signal: controller.signal,
        });
        if (!response.ok) {
            clearAuth();
            return;
        }
        const data = await response.json();
        // After refresh, we need to fetch user info with the new access token
        const meResponse = await fetch('/api/v1/user/me', {
            headers: { Authorization: `Bearer ${data.access_token}` },
            signal: controller.signal,
        });
        if (!meResponse.ok) {
            clearAuth();
            return;
        }
        const meData = await meResponse.json();
        setAuth(meData.user, data.access_token, data.refresh_token);
    } catch {
        clearAuth();
    } finally {
        clearTimeout(timeoutId);
    }
}
