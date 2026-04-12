<script lang="ts">
    import { apiClient } from '$lib/api/client';

    let email = $state('');
    let submitted = $state(false);
    let error = $state('');
    let loading = $state(false);

    async function handleSubmit(event: SubmitEvent) {
        event.preventDefault();
        error = '';
        loading = true;

        try {
            await apiClient.post('/auth/forgot-password', { email });
            submitted = true;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Request failed';
        } finally {
            loading = false;
        }
    }
</script>

<div class="max-w-md mx-auto mt-16">
    <h1 class="text-3xl font-bold mb-6 text-center">Reset Password</h1>

    {#if submitted}
        <div class="card p-6 variant-ghost-surface text-center">
            <p class="text-lg mb-4">Check your email</p>
            <p class="opacity-75 mb-4">
                If an account with that email exists, we've sent a password reset link.
            </p>
            <a href="/login" class="anchor">Back to login</a>
        </div>
    {:else}
        {#if error}
            <div class="alert variant-filled-error mb-4 p-3 rounded">
                <p>{error}</p>
            </div>
        {/if}

        <form onsubmit={handleSubmit} class="card p-6 variant-ghost-surface space-y-4">
            <p class="opacity-75">
                Enter your email address and we'll send you a link to reset your password.
            </p>

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

            <button type="submit" class="btn variant-filled-primary w-full" disabled={loading}>
                {loading ? 'Sending...' : 'Send Reset Link'}
            </button>

            <p class="text-center text-sm">
                <a href="/login" class="anchor">Back to login</a>
            </p>
        </form>
    {/if}
</div>
