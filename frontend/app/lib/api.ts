import axios from 'axios';
import { Platform } from 'react-native';
import { useAuthStore } from '~/app/stores/AuthStore';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800/api/v1';
const isWeb = Platform.OS === 'web';

export const api = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
  ...(isWeb ? { withCredentials: true } : {}),
});

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
