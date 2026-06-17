import { create } from 'zustand';
import { Platform } from 'react-native';
import * as SecureStore from 'expo-secure-store';
import api from '~/app/lib/api';

const isNative = Platform.OS !== 'web';
const isWeb = Platform.OS === 'web';

export interface AuthUser {
  id: number;
  username: string;
  email: string;
}

export interface ProfileData {
  bio?: string;
  display_name?: string;
  location?: string;
  website?: string;
  social_links?: Record<string, string>;
}

export interface UserResponse {
  id: number;
  username: string;
  email: string;
  profile?: ProfileData;
}

interface AuthState {
  token: string | null;
  user: AuthUser | null;
  profile: ProfileData | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  setCredentials: (token: string, user: AuthUser) => void;
  setProfile: (profile: ProfileData) => void;
  clearCredentials: () => Promise<void>;
  hydrate: () => Promise<void>;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  token: null,
  user: null,
  profile: null,
  isAuthenticated: false,
  isLoading: true,

  setCredentials: (token, user) => {
    if (isNative) {
      SecureStore.setItemAsync('auth_token', token);
    }
    set({ token, user, isAuthenticated: true, isLoading: false });
  },

  setProfile: (profile) => {
    set({ profile });
  },

  clearCredentials: async () => {
    if (isNative) {
      await SecureStore.deleteItemAsync('auth_token');
    }
    set({
      token: null,
      user: null,
      profile: null,
      isAuthenticated: false,
      isLoading: false,
    });
  },

  hydrate: async () => {
    set({ isLoading: true });
    try {
      if (isWeb) {
        const res = await api.get<UserResponse>('/api/v1/user');
        set({
          token: null,
          user: res.data,
          profile: res.data.profile || null,
          isAuthenticated: true,
          isLoading: false,
        });
      } else {
        const token = await SecureStore.getItemAsync('auth_token');
        if (token) {
          const res = await api.get<UserResponse>('/api/v1/user');
          set({
            token,
            user: res.data,
            profile: res.data.profile || null,
            isAuthenticated: true,
            isLoading: false,
          });
        } else {
          set({ isLoading: false });
        }
      }
    } catch {
      if (isNative) {
        await SecureStore.deleteItemAsync('auth_token');
      }
      set({
        token: null,
        user: null,
        profile: null,
        isAuthenticated: false,
        isLoading: false,
      });
    }
  },
}));
