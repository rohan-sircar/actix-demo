import { View, Text, TouchableOpacity } from 'react-native';
import React, { useEffect, useState } from 'react';
import { useLocalSearchParams, useRouter } from 'expo-router';
import { useNavigation } from '@react-navigation/native';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet } from '~/lib/useAccentColor';
import * as Style from '~/app/styles/Styles';
import { DrawerNavigationProp } from '@react-navigation/drawer';
import { DrawerParamList, navigateWithTitle } from '~/types/navigation';

const VerifyWebScreen = () => {
  const navigation = useNavigation<DrawerNavigationProp<DrawerParamList>>();
  const router = useRouter();
  const { colors, isDarkColorScheme } = useColorScheme();
  const accentColor = (useColorScheme() as any).accentColor || 'ocean';
  const accentSet = getAccentSet(accentColor);
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
        await api.post('/api/v1/email/verify', { token });
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

  return (
    <View className="flex-1 items-center justify-center px-4">
      <View
        className={`w-full max-w-[380px] rounded-xl p-6 shadow-lg`}
        style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
        <View>
          <Text
            className={`mb-2 text-center text-xl font-semibold ${Style.getHeadingTextColor(isDarkColorScheme, accentSet)}`}>
            Verify Email
          </Text>
          <Text
            className={`mb-6 text-center text-sm ${Style.getSecondaryTextColor(isDarkColorScheme, accentSet)}`}>
            {status === 'verifying' && 'Checking your verification link...'}
          </Text>
        </View>

        {status === 'success' ? (
          <View className="mb-4 rounded-lg bg-emerald-500/20 p-3">
            <Text className="text-center text-sm text-emerald-500">{message}</Text>
          </View>
        ) : null}

        {status === 'error' ? (
          <View className="mb-4 rounded-lg bg-rose-500/20 p-3">
            <Text className="text-center text-sm text-rose-500">{message}</Text>
          </View>
        ) : null}

        {status !== 'verifying' ? (
          <View className="mt-4 gap-3">
            <TouchableOpacity
              onPress={() => {
                if (status === 'success') {
                  navigateWithTitle(() => navigation.navigate('Account', { screen: 'SignIn' }), 'Sign In');
                } else {
                  router.push('/resend-verification');
                }
              }}
              className={`rounded-lg px-4 py-3 items-center ${
                status === 'success'
                  ? 'bg-emerald-500'
                  : isDarkColorScheme
                    ? 'bg-zinc-700'
                    : 'bg-zinc-200'
              }`}>
              <Text
                className={`font-medium ${
                  status === 'success' ? 'text-white' : Style.getHeadingTextColor(isDarkColorScheme, accentSet)
                }`}>
                {status === 'success' ? 'Go to Sign In' : 'Try Again'}
              </Text>
            </TouchableOpacity>
          </View>
        ) : null}
      </View>
    </View>
  );
};

export default VerifyWebScreen;
