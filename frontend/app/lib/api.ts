import axios from 'axios';
import * as FileSystem from 'expo-file-system/legacy';
import { Platform } from 'react-native';
import { useAuthStore } from '~/app/stores/AuthStore';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800/api/v1';

export const api = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
  ...(Platform.OS === 'web' ? { withCredentials: true } : {}),
});

export const petImageApi = {
  async upload(petUuid: string, fileUri: string, mimeType: string): Promise<{ id: number; uuid: string; format: string; is_primary: boolean; sort_order: number; created_at: string }> {
    console.log('[petImageApi] Reading file:', fileUri);
    let bytes: Uint8Array;
    if (fileUri.startsWith('data:')) {
      const base64 = fileUri.split(',')[1];
      bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
    } else if (Platform.OS === 'web') {
      const response = await fetch(fileUri);
      const blob = await response.blob();
      bytes = new Uint8Array(await blob.arrayBuffer());
    } else {
      const base64 = await FileSystem.readAsStringAsync(fileUri, { encoding: 'base64' });
      console.log('[petImageApi] File read, base64 length:', base64.length);
      bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
    }
    console.log('[petImageApi] Sending upload request, bytes length:', bytes.length);
    const response = await api.post(`/api/v1/private/user/pets/${petUuid}/images`, bytes, {
      headers: { 'Content-Type': mimeType },
    });
    console.log('[petImageApi] Upload response:', response.status);
    return response.data;
  },

  async list(petUuid: string): Promise<Array<{ id: number; uuid: string; format: string; is_primary: boolean; sort_order: number; created_at: string }>> {
    const response = await api.get(`/api/v1/private/user/pets/${petUuid}/images`);
    return response.data;
  },

  async remove(petUuid: string, imageUuid: string): Promise<void> {
    await api.delete(`/api/v1/private/user/pets/${petUuid}/images/${imageUuid}`);
  },

  async setPrimary(petUuid: string, imageUuid: string): Promise<{ id: number; uuid: string; format: string; is_primary: boolean; sort_order: number; created_at: string }> {
    const response = await api.patch(`/api/v1/private/user/pets/${petUuid}/images/${imageUuid}`, { is_primary: true });
    return response.data;
  },
};

api.interceptors.request.use(async (config) => {
  const token = useAuthStore.getState().token;
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      const store = useAuthStore.getState();
      await store.clearCredentials();
    }
    return Promise.reject(error);
  }
);

export const petApi = {
  async updatePet(petUuid: string, data: {
    name?: string;
    species?: string;
    breed?: string | null;
    date_of_birth?: string | null;
    gender?: string | null;
    weight?: number | null;
    color_markings?: string | null;
    description?: string | null;
    traits?: Array<{ id: number; name: string }>;
  }): Promise<{
    id: number;
    pet_uuid: string;
    name: string;
    species: string;
    breed?: string | null;
    date_of_birth?: string | null;
    gender?: string | null;
    weight?: number | null;
    color_markings?: string | null;
    description?: string | null;
    traits?: Array<{ id: number; name: string }>;
    primary_image: { id: number; uuid: string; format: string; is_primary: boolean; sort_order: number; created_at: string } | null;
  }> {
    const response = await api.patch(`/api/v1/private/user/pets/${petUuid}`, data);
    return response.data;
  },
};

export const discoverApi = {
  async next(): Promise<import('~/app/models/pets').PublicPet | null> {
    const response = await api.get('/api/v1/private/discover/next');
    return response.data;
  },

  async list(query: import('~/app/models/pets').DiscoverQuery): Promise<import('~/app/models/pets').PaginatedResponse<import('~/app/models/pets').PublicPet>> {
    const response = await api.get('/api/v1/private/discover/pets', { params: query });
    return response.data;
  },
};

export const likesApi = {
  async create(petUuid: string, direction: 'like' | 'dislike'): Promise<import('~/app/models/pets').LikeRecord> {
    const response = await api.post('/api/v1/private/likes', { pet_uuid: petUuid, direction });
    return response.data;
  },
};

export const profileApi = {
  async get(): Promise<import('~/app/models/pets').UserProfile> {
    const response = await api.get('/api/v1/private/user/profile');
    return response.data;
  },
};

export default api;
