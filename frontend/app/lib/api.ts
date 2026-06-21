import axios from 'axios';
import * as FileSystem from 'expo-file-system/legacy';
import { useAuthStore } from '~/app/stores/AuthStore';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800/api/v1';

export const api = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
  ...(typeof window !== 'undefined' ? { withCredentials: true } : {}),
});

export const petImageApi = {
  async upload(petUuid: string, fileUri: string, mimeType: string): Promise<{ uuid: string; is_primary: boolean; sort_order: number }> {
    console.log('[petImageApi] Reading file:', fileUri);
    let bytes: Uint8Array;
    const isOnWeb = typeof window !== 'undefined';
    if (fileUri.startsWith('data:')) {
      const base64 = fileUri.split(',')[1];
      bytes = Uint8Array.from(atob(base64), c => c.charCodeAt(0));
    } else if (isOnWeb) {
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

  async list(petUuid: string): Promise<Array<{ uuid: string; is_primary: boolean; sort_order: number }>> {
    const response = await api.get(`/api/v1/private/user/pets/${petUuid}/images`);
    return response.data;
  },

  async remove(petUuid: string, imageUuid: string): Promise<void> {
    await api.delete(`/api/v1/private/user/pets/${petUuid}/images/${imageUuid}`);
  },

  async setPrimary(petUuid: string, imageUuid: string): Promise<{ uuid: string; is_primary: boolean; sort_order: number }> {
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

export default api;
