<script lang="ts">
    import { apiClient } from '$lib/api/client';
    import { setAuth } from '$lib/stores/auth.svelte';
    import { goto } from '$app/navigation';
    import type { AuthResponse } from '$lib/types';

    let email = $state('');
    let password = $state('');
    let error = $state('');
    let loading = $state(false);

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        error = '';
        loading = true;

        try {
            const data = await apiClient.post<AuthResponse>('/auth/login', { email, password });
            setAuth(data.user, data.access_token, data.refresh_token);
            goto('/dashboard');
        } catch (e) {
            error = e instanceof Error ? e.message : 'Login failed';
        } finally {
            loading = false;
        }
    }
</script>

<div class="max-w-md mx-auto mt-16">
    <h1 class="text-3xl font-bold mb-6 text-center">Sign In</h1>

    {#if error}
        <div class="alert variant-filled-error mb-4 p-3 rounded">
            <p>{error}</p>
        </div>
    {/if}

    <form onsubmit={handleSubmit} class="card p-6 variant-ghost-surface space-y-4">
        <label class="label">
            <span>Email</span>
            <input
                type="email"
                bind:value={email}
                class="input px-3 py-2"
                placeholder="you@example.com"
                required
            />
        </label>

        <label class="label">
            <span>Password</span>
            <input
                type="password"
                bind:value={password}
                class="input px-3 py-2"
                placeholder="********"
                required
            />
        </label>

        <button type="submit" class="btn variant-filled-primary w-full" disabled={loading}>
            {loading ? 'Signing in...' : 'Sign In'}
        </button>

        <div class="text-center text-sm space-y-1">
            <p>
                <a href="/forgot-password" class="anchor">Forgot your password?</a>
            </p>
            <p>
                Don't have an account? <a href="/register" class="anchor">Register</a>
            </p>
        </div>
    </form>
</div>
