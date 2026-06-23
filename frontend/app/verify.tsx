import { View, Text, TouchableOpacity } from 'react-native';
import React, { useEffect, useState } from 'react';
import { useLocalSearchParams, useRouter } from 'expo-router';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '~/app/styles/Styles';
import { LinearGradient } from 'expo-linear-gradient';
import { Ionicons } from '@expo/vector-icons';
import { BaseAccentGradients } from '~/theme/colors';

const VerifyWebScreen = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const router = useRouter();
  const [status, setStatus] = useState<'verifying' | 'success' | 'error'>('verifying');
  const [message, setMessage] = useState('Verifying your email...');

  const searchParams = useLocalSearchParams();
  const token = searchParams?.token as string | undefined;

  useEffect(() => {
    const verifyEmail = async () => {
      if (!token) {
        setStatus('error');
        setMessage('Missing verification token.');
        return;
      }

      try {
        await api.post('/api/v1/auth/verify-email', { token });
        setStatus('success');
        setMessage('Email verified successfully! You can now use all features.');
      } catch (err: any) {
        setStatus('error');
        setMessage(
          err.response?.data?.message || 'Failed to verify email. The link may have expired.'
        );
      }
    };

    verifyEmail();
  }, [token]);

  const gradientColors = BaseAccentGradients[accentColor];

  return (
    <LinearGradient
      colors={[gradientColors.gradientStart, gradientColors.gradientEnd]}
      style={{ flex: 1 }}>
      <View className="flex-1 items-center justify-center px-6">
        <View className="w-full max-w-[400px]">
          <View className="mb-8 items-center">
            <View
              className="mb-4 items-center justify-center rounded-full"
              style={{ width: 72, height: 72, backgroundColor: 'rgba(255,255,255,0.2)' }}>
              <Ionicons name="mail" size={32} color="white" />
            </View>
            <Text className="text-3xl font-bold tracking-tight text-white">Verify Email</Text>
          </View>

          <View
            className="rounded-2xl p-6 shadow-xl"
            style={{
              backgroundColor: isDarkColorScheme ? 'rgba(35,25,22,0.95)' : 'rgba(255,255,255,0.95)',
            }}>
            {status === 'verifying' && (
              <Text className="text-center text-sm" style={{ color: colors.grey }}>
                Checking your verification link...
              </Text>
            )}

            {status === 'success' ? (
              <View className="mb-4 rounded-xl bg-emerald-500/15 p-3">
                <Text className="text-center text-sm font-medium text-emerald-600">{message}</Text>
              </View>
            ) : null}

            {status === 'error' ? (
              <View className="mb-4 rounded-xl bg-rose-500/15 p-3">
                <Text className="text-center text-sm font-medium text-rose-500">{message}</Text>
              </View>
            ) : null}

            {status !== 'verifying' ? (
              <View className="mt-4 gap-3">
                {status === 'success' ? (
                  <TouchableOpacity
                    onPress={() => router.replace('/auth/sign-in')}
                    className="items-center rounded-xl bg-emerald-500 px-4 py-3">
                    <Text className="font-semibold text-white">Go to Sign In</Text>
                  </TouchableOpacity>
                ) : (
                  <>
                    <TouchableOpacity
                      onPress={() => router.push('/resend-verification')}
                      className="items-center rounded-xl bg-[#F4644E] px-4 py-3">
                      <Text className="font-semibold text-white">Resend Email</Text>
                    </TouchableOpacity>
                    <TouchableOpacity
                      onPress={() => router.back()}
                      className="items-center rounded-xl px-4 py-3"
                      style={{ backgroundColor: isDarkColorScheme ? '#3d2a22' : '#f0e0d8' }}>
                      <Text className="font-medium" style={{ color: colors.text }}>
                        Go Back
                      </Text>
                    </TouchableOpacity>
                  </>
                )}
              </View>
            ) : null}
          </View>
        </View>
      </View>
    </LinearGradient>
  );
};

export default VerifyWebScreen;
