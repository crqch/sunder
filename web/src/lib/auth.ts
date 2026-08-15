import { writable, get } from 'svelte/store';
import { apiFetch } from '$lib/api';

export interface User {
  id: string;
  username: string;
  flags?: string[];
}

export interface AuthState {
  isAuthenticated: boolean;
  user: User | null;
  accessToken: string | null;
  refreshToken: string | null;
  loading: boolean;
}

const initialState: AuthState = {
  isAuthenticated: false,
  user: null,
  accessToken: null,
  refreshToken: null,
  loading: true
};

export const authStore = writable<AuthState>(initialState);

export async function initAuth() {
  if (typeof window === 'undefined') return;

  const storedRefreshToken = localStorage.getItem('sunder_refresh_token');

  if (storedRefreshToken) {
    authStore.update((s) => ({ ...s, refreshToken: storedRefreshToken, loading: true }));
    try {
      await refreshAccessToken();
    } catch (e) {
      console.error('Failed to init auth', e);
      // We don't call logout() here because a network error (like refreshing the page while fetching)
      // shouldn't clear the refresh token. refreshAccessToken() will handle legitimate auth failures.
      authStore.update((s) => ({ ...s, loading: false }));
    }
  } else {
    authStore.update((s) => ({ ...s, loading: false }));
  }
}

export async function login(login: string, pass: string) {
  const res = await apiFetch('/auth/login', {
    method: 'POST',
    body: JSON.stringify({ login, pass })
  });

  const data = await res.json();

  if (!res.ok) {
    throw new Error(data.message || 'Login failed');
  }

  localStorage.setItem('sunder_refresh_token', data.refresh_token);

  authStore.update((s) => ({
    ...s,
    refreshToken: data.refresh_token,
    loading: true
  }));

  await refreshAccessToken();
}

export async function register(invite: string, email: string, username: string, pass: string) {
  const res = await apiFetch(`/auth/register/${encodeURIComponent(invite)}`, {
    method: 'POST',
    body: JSON.stringify({ email, username, pass })
  });

  const data = await res.json();

  if (!res.ok) {
    throw data;
  }

  return data;
}

export async function refreshAccessToken() {
  const state = get(authStore);
  const token =
    state.refreshToken ||
    (typeof window !== 'undefined' ? localStorage.getItem('sunder_refresh_token') : null);

  if (!token) {
    throw new Error('No refresh token available');
  }

  const res = await apiFetch('/auth/refresh_token', {
    method: 'POST',
    body: JSON.stringify({ token })
  });

  if (!res.ok) {
    logout();
    throw new Error('Failed to refresh token');
  }

  const data = await res.json();

  if (typeof document !== 'undefined') {
    document.cookie = `authorization=${data.access_token}; path=/`;
  }

  const profileRes = await apiFetch('/dashboard/', {
    credentials: 'include'
  });

  if (!profileRes.ok) {
    throw new Error('Failed to fetch user profile');
  }

  const profileData = await profileRes.json();

  authStore.update((s) => ({
    ...s,
    isAuthenticated: true,
    accessToken: data.access_token,
    user: {
      id: profileData.id,
      username: profileData.username,
      flags: profileData.flags
    },
    loading: false
  }));
}

export function logout() {
  if (typeof window !== 'undefined') {
    localStorage.removeItem('sunder_refresh_token');
    if (!window.location.pathname.startsWith('/auth/')) {
      window.location.href = '/auth/sign-in';
    }
  }

  authStore.set({
    isAuthenticated: false,
    user: null,
    accessToken: null,
    refreshToken: null,
    loading: false
  });
}

export async function authenticatedFetch(
  endpoint: string,
  options: RequestInit = {}
): Promise<Response> {
  let state = get(authStore);

  if (!state.accessToken) {
    if (state.refreshToken) {
      await refreshAccessToken();
      state = get(authStore);
    } else {
      throw new Error('Not authenticated');
    }
  }

  let headers = new Headers(options.headers || {});
  // headers.set('Authorization', `Bearer ${state.accessToken}`);

  if (typeof document !== 'undefined') {
    document.cookie = `authorization=${state.accessToken}; path=/`;
  }

  let res = await apiFetch(endpoint, {
    ...options,
    headers,
    credentials: 'include'
  });

  if (res.status === 401) {
    await refreshAccessToken();
    state = get(authStore);
    // headers.set('Authorization', `Bearer ${state.accessToken}`);
    if (typeof document !== 'undefined') {
      document.cookie = `authorization=${state.accessToken}; path=/`;
    }

    res = await apiFetch(endpoint, {
      ...options,
      headers,
      credentials: 'include'
    });
  }

  return res;
}
