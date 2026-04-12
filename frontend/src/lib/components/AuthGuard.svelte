<!--
    UX-only redirect guard. Not a security boundary — all protected data
    is enforced server-side via Bearer token validation.
-->
<script lang="ts">
    import { auth } from '$lib/stores/auth.svelte';
    import { goto } from '$app/navigation';
    import type { Snippet } from 'svelte';

    let { children }: { children: Snippet } = $props();

    $effect(() => {
        if (!auth.isLoading && !auth.isAuthenticated) {
            goto('/login');
        }
    });
</script>

{#if auth.isLoading}
    <div class="flex justify-center items-center h-64">
        <p class="text-lg opacity-50">Loading...</p>
    </div>
{:else if auth.isAuthenticated}
    {@render children()}
{/if}
