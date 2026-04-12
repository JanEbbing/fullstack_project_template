<script lang="ts">
    import { onMount } from 'svelte';
    import AuthGuard from '$lib/components/AuthGuard.svelte';
    import { auth } from '$lib/stores/auth.svelte';
    import { apiClient } from '$lib/api/client';
    import type { UserDataItem } from '$lib/types';

    let items: UserDataItem[] = $state([]);
    let newTitle = $state('');
    let newContent = $state('');
    let error = $state('');
    let loadingItems = $state(true);

    onMount(async () => {
        await loadItems();
    });

    async function loadItems() {
        try {
            const data = await apiClient.get<{ data: UserDataItem[] }>('/user/data');
            items = data.data;
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to load data';
        } finally {
            loadingItems = false;
        }
    }

    async function createItem(event: SubmitEvent) {
        event.preventDefault();
        if (!newTitle.trim()) return;

        try {
            const data = await apiClient.post<{ data: UserDataItem }>('/user/data', {
                title: newTitle,
                content: newContent,
            });
            items = [data.data, ...items];
            newTitle = '';
            newContent = '';
        } catch (e) {
            error = e instanceof Error ? e.message : 'Failed to create item';
        }
    }
</script>

<AuthGuard>
    <div class="max-w-3xl mx-auto py-8">
        <h1 class="text-3xl font-bold mb-2">Dashboard</h1>
        <p class="opacity-75 mb-8">Welcome back, {auth.user?.email}</p>

        {#if error}
            <div class="alert variant-filled-error mb-4 p-3 rounded">
                <p>{error}</p>
            </div>
        {/if}

        <!-- Create new item -->
        <form onsubmit={createItem} class="card p-4 variant-ghost-surface mb-8 space-y-3">
            <h2 class="text-lg font-semibold">Add New Item</h2>
            <input
                type="text"
                bind:value={newTitle}
                class="input px-3 py-2"
                placeholder="Title"
                required
            />
            <textarea
                bind:value={newContent}
                class="textarea px-3 py-2"
                placeholder="Content (optional)"
                rows="3"
            ></textarea>
            <button type="submit" class="btn variant-filled-primary">Add Item</button>
        </form>

        <!-- Items list -->
        <h2 class="text-xl font-semibold mb-4">Your Items</h2>
        {#if loadingItems}
            <p class="opacity-50">Loading...</p>
        {:else if items.length === 0}
            <p class="opacity-50">No items yet. Create your first one above!</p>
        {:else}
            <div class="space-y-3">
                {#each items as item (item.id)}
                    <div class="card p-4 variant-ghost-surface">
                        <h3 class="font-semibold">{item.title}</h3>
                        {#if item.content}
                            <p class="opacity-75 mt-1">{item.content}</p>
                        {/if}
                        <p class="text-xs opacity-50 mt-2">
                            Created: {new Date(item.created_at).toLocaleString()}
                        </p>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
</AuthGuard>
