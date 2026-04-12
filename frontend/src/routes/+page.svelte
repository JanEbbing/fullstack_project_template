<script lang="ts">
    import { auth } from '$lib/stores/auth.svelte';
</script>

<div class="max-w-4xl mx-auto py-12">
    <!-- Hero -->
    <section class="text-center mb-16">
        <h1 class="text-5xl font-bold mb-4">Build Something Great</h1>
        <p class="text-xl opacity-75 mb-8 max-w-2xl mx-auto">
            A full-stack project template with authentication, database integration, and modern
            tooling. Ready for your next hackathon or side project.
        </p>
        {#if !auth.isAuthenticated && !auth.isLoading}
            <div class="flex gap-4 justify-center">
                <a href="/register" class="btn variant-filled-primary text-lg px-8 py-3">
                    Get Started
                </a>
                <a href="/login" class="btn variant-ghost-surface text-lg px-8 py-3">Sign In</a>
            </div>
        {:else if auth.isAuthenticated}
            <a href="/dashboard" class="btn variant-filled-primary text-lg px-8 py-3">
                Go to Dashboard
            </a>
        {/if}
    </section>

    <!-- Features -->
    <section class="grid grid-cols-1 md:grid-cols-3 gap-8 mb-16">
        <div class="card p-6 variant-ghost-surface">
            <h3 class="text-xl font-semibold mb-2">Rust Backend</h3>
            <p class="opacity-75">
                High-performance API built with Axum. JWT authentication, input validation, and
                structured error handling out of the box.
            </p>
        </div>
        <div class="card p-6 variant-ghost-surface">
            <h3 class="text-xl font-semibold mb-2">Svelte Frontend</h3>
            <p class="opacity-75">
                Modern reactive UI with SvelteKit. Skeleton UI components, Tailwind CSS, and
                automatic token refresh for seamless auth.
            </p>
        </div>
        <div class="card p-6 variant-ghost-surface">
            <h3 class="text-xl font-semibold mb-2">Ready to Deploy</h3>
            <p class="opacity-75">
                Dockerized with a single container. CI/CD pipeline included. Just add your cloud
                provider credentials and ship.
            </p>
        </div>
    </section>

    <!-- Details -->
    <section class="text-center opacity-60">
        <p>
            Built with Svelte 5 &middot; Axum &middot; SQLite &middot; Tailwind CSS &middot; Docker
        </p>
    </section>
</div>
