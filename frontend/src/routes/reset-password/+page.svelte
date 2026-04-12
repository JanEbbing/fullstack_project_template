<script lang="ts">
    import { page } from '$app/stores';
    import { apiClient } from '$lib/api/client';

    let password = $state('');
    let confirmPassword = $state('');
    let error = $state('');
    let success = $state(false);
    let loading = $state(false);

    let token = $derived($page.url.searchParams.get('token') || '');

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        error = '';

        if (!token) {
            error = 'Missing reset token';
            return;
        }
        if (password !== confirmPassword) {
            error = 'Passwords do not match';
            return;
        }
        if (password.length < 8) {
            error = 'Password must be at least 8 characters';
            return;
        }

        loading = true;
        try {
            await apiClient.post('/auth/reset-password', {
                token,
                new_password: password,
            });
            success = true;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Reset failed';
        } finally {
            loading = false;
        }
    }
</script>

<div class="max-w-md mx-auto mt-16">
    <h1 class="text-3xl font-bold mb-6 text-center">Set New Password</h1>

    {#if success}
        <div class="card p-6 variant-ghost-surface text-center">
            <p class="text-lg mb-4">Password reset successful!</p>
            <p class="opacity-75 mb-4">You can now sign in with your new password.</p>
            <a href="/login" class="btn variant-filled-primary">Go to Login</a>
        </div>
    {:else if !token}
        <div class="card p-6 variant-ghost-surface text-center">
            <p class="text-lg mb-4">Invalid reset link</p>
            <p class="opacity-75 mb-4">This link appears to be invalid or expired.</p>
            <a href="/forgot-password" class="anchor">Request a new reset link</a>
        </div>
    {:else}
        {#if error}
            <div class="alert variant-filled-error mb-4 p-3 rounded">
                <p>{error}</p>
            </div>
        {/if}

        <form onsubmit={handleSubmit} class="card p-6 variant-ghost-surface space-y-4">
            <label class="label">
                <span>New Password</span>
                <input
                    type="password"
                    bind:value={password}
                    class="input px-3 py-2"
                    placeholder="At least 8 characters"
                    required
                    minlength="8"
                />
            </label>

            <label class="label">
                <span>Confirm New Password</span>
                <input
                    type="password"
                    bind:value={confirmPassword}
                    class="input px-3 py-2"
                    placeholder="Repeat your password"
                    required
                />
            </label>

            <button type="submit" class="btn variant-filled-primary w-full" disabled={loading}>
                {loading ? 'Resetting...' : 'Reset Password'}
            </button>
        </form>
    {/if}
</div>
