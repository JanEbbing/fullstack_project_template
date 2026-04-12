export interface User {
    id: string;
    email: string;
    created_at: string;
}

export interface AuthResponse {
    user: User;
    access_token: string;
    refresh_token: string;
}

export interface TokenResponse {
    access_token: string;
    refresh_token: string;
}

export interface UserDataItem {
    id: string;
    user_id: string;
    title: string;
    content: string;
    created_at: string;
    updated_at: string;
}

export interface ApiError {
    error: string;
}
