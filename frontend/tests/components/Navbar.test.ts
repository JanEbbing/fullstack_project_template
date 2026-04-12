import { render, screen } from '@testing-library/svelte';
import { describe, it, expect, beforeEach } from 'vitest';
import Navbar from '$lib/components/Navbar.svelte';
import { auth } from '$lib/stores/auth.svelte';

describe('Navbar', () => {
    beforeEach(() => {
        // Reset auth state to non-loading, unauthenticated
        auth.user = null;
        auth.accessToken = null;
        auth.isAuthenticated = false;
        auth.isLoading = false;
    });

    it('shows login and register links when not authenticated', () => {
        render(Navbar);
        expect(screen.getByText('Login')).toBeTruthy();
        expect(screen.getByText('Register')).toBeTruthy();
    });

    it('shows the app name', () => {
        render(Navbar);
        expect(screen.getByText('MyApp')).toBeTruthy();
    });
});
