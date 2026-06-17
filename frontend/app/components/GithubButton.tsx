import React, { useState } from 'react';
import { Alert, Text, Platform } from 'react-native';
import { Button } from '~/components/nativewindui/Button';
import * as WebBrowser from 'expo-web-browser';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import { Ionicons } from '@expo/vector-icons';
import { api } from '~/app/lib/api';

WebBrowser.maybeCompleteAuthSession();

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';
const SCHEME = 'my-expo-app';

const GithubButton = () => {
  const setCredentials = useAuthStore((s) => s.setCredentials);
  const [loading, setLoading] = useState(false);

  const handlePress = async () => {
    if (loading) return;
    setLoading(true);

    try {
      if (Platform.OS === 'web') {
        const frontendUrl = `${window.location.protocol}//${window.location.host}`;
        const loginUrl = `${API_BASE_URL}/api/v1/auth/oauth/github/login?redirect=${encodeURIComponent(frontendUrl)}`;
        await WebBrowser.openAuthSessionAsync(loginUrl, frontendUrl);

        try {
          const userRes = await api.get<UserResponse>('/api/v1/user');
          setCredentials('', userRes.data);
        } catch {
          Alert.alert('Error', 'Authentication failed. Please try again.');
        }
      } else {
        const authorizeUrl = `${API_BASE_URL}/api/v1/auth/oauth/github/login?redirect=${encodeURIComponent(`${SCHEME}://oauth/github/callback`)}`;
        const redirectUrl = `${SCHEME}://oauth/github/callback`;

        const result = await WebBrowser.openAuthSessionAsync(authorizeUrl, redirectUrl);

        if (result.type === 'success' && result.url) {
          const url = new URL(result.url);
          const code = url.searchParams.get('code');

          if (code) {
            const res = await api.post('/auth/oauth/github/exchange', {
              code,
              state: url.searchParams.get('state'),
            });
            setCredentials(res.data.token, res.data.user);
          }
        }
      }
    } catch (err: any) {
      const message = err?.response?.data?.message || 'GitHub login failed';
      Alert.alert('Error', message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      className="h-12 w-full flex-row items-center justify-center gap-3 rounded-lg"
      hoverColor="rgb(48, 48, 43)"
      defaultColor="rgb(40, 41, 35)"
      onPress={handlePress}
      disabled={loading}>
      <Ionicons name="logo-github" size={20} color="#fff" />
      <Text className="text-base font-medium text-white">
        {loading ? 'Signing in...' : 'Sign in with GitHub'}
      </Text>
    </Button>
  );
};

export default GithubButton;
