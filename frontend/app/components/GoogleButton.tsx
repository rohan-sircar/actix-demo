import React, { useState } from 'react';
import { Alert, Text, View, Platform } from 'react-native';
import { Button } from '~/components/nativewindui/Button';
import * as WebBrowser from 'expo-web-browser';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAccentColor, getAccentSet } from '~/lib/useAccentColor';
import { Ionicons } from '@expo/vector-icons';
import { api } from '~/app/lib/api';

WebBrowser.maybeCompleteAuthSession();

const API_BASE_URL = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';
const SCHEME = 'my-expo-app';

const GoogleButton = () => {
  const setCredentials = useAuthStore((s) => s.setCredentials);
  const [loading, setLoading] = useState(false);
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const handlePress = async () => {
    if (loading) return;
    setLoading(true);

    try {
      if (Platform.OS === 'web') {
        const loginUrl = `${API_BASE_URL}/api/v1/auth/oauth/google/login?redirect=${encodeURIComponent(`${API_BASE_URL}/api/v1/auth/oauth/google/callback`)}`;
        await WebBrowser.openAuthSessionAsync(loginUrl, loginUrl);

        const userRes = await api.get<UserResponse>('/api/v1/private/user');
        setCredentials('', userRes.data);
      } else {
        const authorizeUrl = `${API_BASE_URL}/api/v1/auth/oauth/google/login`;
        const redirectUrl = `${SCHEME}://oauth/google/callback`;

        const result = await WebBrowser.openAuthSessionAsync(authorizeUrl, redirectUrl);

        if (result.type === 'success' && result.url) {
          const url = new URL(result.url);
          const code = url.searchParams.get('code');

          if (code) {
            const res = await api.post('/api/v1/auth/oauth/google/exchange', {
              code,
              state: url.searchParams.get('state'),
            });
            setCredentials(res.data.token, res.data.user);
          }
        }
      }
    } catch (err: any) {
      const message = err?.response?.data?.message || 'Google login failed';
      Alert.alert('Error', message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Button
      className="h-12 w-full flex-row items-center justify-center gap-3 rounded-xl border-2 border-[#E8D0C0]"
      androidRootClassName="rounded-xl"
      hoverColor={isDarkColorScheme ? '#3d2a22' : '#FFF0E8'}
      defaultColor={colors.card}
      onPress={handlePress}
      disabled={loading}>
      <Ionicons name="logo-google" size={20} color="#4285F4" />
      <Text className="text-base font-semibold" style={{ color: colors.text }}>
        {loading ? 'Signing in...' : 'Continue with Google'}
      </Text>
    </Button>
  );
};

export default GoogleButton;
