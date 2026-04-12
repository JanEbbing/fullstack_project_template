<script lang="ts">
    import { onMount } from 'svelte';
    import { initAuth } from '$lib/stores/auth.svelte';
    import Navbar from '$lib/components/Navbar.svelte';
    import '../app.css';

    let { children } = $props();

    onMount(() => {
        initAuth();
    });
</script>

<Navbar />
<main class="container mx-auto p-4">
    {@render children()}
</main>
