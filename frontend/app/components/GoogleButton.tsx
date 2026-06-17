import React, { useState } from 'react';
import { Alert, Text, View, Platform } from 'react-native';
import { Button } from '~/components/nativewindui/Button';
import * as WebBrowser from 'expo-web-browser';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import * as Style from '../styles/Styles';
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

        const userRes = await api.get<UserResponse>('/api/v1/user');
        setCredentials('', userRes.data);
      } else {
        const authorizeUrl = `${API_BASE_URL}/api/v1/auth/oauth/google/login`;
        const redirectUrl = `${SCHEME}://oauth/google/callback`;

        const result = await WebBrowser.openAuthSessionAsync(authorizeUrl, redirectUrl);

        if (result.type === 'success' && result.url) {
          const url = new URL(result.url);
          const code = url.searchParams.get('code');

          if (code) {
            const res = await api.post('/auth/oauth/google/exchange', { code, state: url.searchParams.get('state') });
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
      className={Style.getSocialButtonClasses(isDarkColorScheme, accentSet)}
      hoverColor={colors.grey6}
      defaultColor={colors.card}
      onPress={handlePress}
      disabled={loading}>
      <View className="flex-row items-center gap-3">
        <Ionicons name="logo-google" size={20} color="#4285F4" />
        <Text
          className={`text-base font-medium ${Style.getHeadingTextColor(
            isDarkColorScheme,
            accentSet
          )}`}>
          {loading ? 'Signing in...' : 'Sign in with Google'}
        </Text>
      </View>
    </Button>
  );
};

export default GoogleButton;
