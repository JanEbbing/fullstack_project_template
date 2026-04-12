<script lang="ts">
    import { auth, clearAuth } from '$lib/stores/auth.svelte';
    import { apiClient } from '$lib/api/client';
    import { goto } from '$app/navigation';

    async function logout() {
        const refreshToken = localStorage.getItem('refresh_token');
        if (refreshToken) {
            try {
                await apiClient.post('/auth/logout', { refresh_token: refreshToken });
            } catch {
                // Proceed with local logout even if server call fails
            }
        }
        clearAuth();
        goto('/');
    }
</script>

<nav class="bg-surface-800 text-white px-6 py-3 flex items-center justify-between">
    <a href="/" class="text-xl font-bold hover:opacity-80">MyApp</a>

    <div class="flex items-center gap-4">
        {#if auth.isLoading}
            <span class="text-sm opacity-50">Loading...</span>
        {:else if auth.isAuthenticated}
            <span class="text-sm opacity-75">{auth.user?.email}</span>
            <a href="/dashboard" class="btn btn-sm variant-ghost-surface">Dashboard</a>
            <button onclick={logout} class="btn btn-sm variant-filled-error">Logout</button>
        {:else}
            <a href="/login" class="btn btn-sm variant-ghost-surface">Login</a>
            <a href="/register" class="btn btn-sm variant-filled-primary">Register</a>
        {/if}
    </div>
</nav>
