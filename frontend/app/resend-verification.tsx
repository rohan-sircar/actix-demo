import { View, Text, TextInput, TouchableOpacity } from 'react-native';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import React, { useState } from 'react';
import { useRouter } from 'expo-router';
import { resendVerificationSchema, ResendVerificationFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import FormButton from '~/app/components/FormButton';
import * as Style from '~/app/styles/Styles';
import { LinearGradient } from 'expo-linear-gradient';
import { Ionicons } from '@expo/vector-icons';
import { BaseAccentGradients } from '~/theme/colors';

const ResendVerificationScreen = () => {
  const router = useRouter();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [sent, setSent] = useState(false);
  const [error, setError] = useState('');

  const {
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<ResendVerificationFormData>({
    resolver: zodResolver(resendVerificationSchema),
    defaultValues: { email: '' },
  });

  const onSubmit = async (data: ResendVerificationFormData) => {
    setError('');
    try {
      await api.post('/api/v1/auth/resend-verification-email', { email: data.email });
      setSent(true);
    } catch {
      setError('Failed to send verification email. Please try again.');
    }
  };

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
              <Ionicons name="repeat" size={32} color="white" />
            </View>
            <Text className="text-3xl font-bold tracking-tight text-white">
              Resend Verification
            </Text>
            <Text className="mt-1 text-center text-base text-white/80">
              We'll send a fresh link to your inbox
            </Text>
          </View>

          <View
            className="rounded-2xl p-6 shadow-xl"
            style={{
              backgroundColor: isDarkColorScheme ? 'rgba(35,25,22,0.95)' : 'rgba(255,255,255,0.95)',
            }}>
            {sent ? (
              <View className="mb-4 rounded-xl bg-emerald-500/15 p-3">
                <Text className="text-center text-sm font-medium text-emerald-600">
                  Check your email for the verification link!
                </Text>
              </View>
            ) : null}

            {error ? (
              <View className="mb-4 rounded-xl bg-rose-500/15 p-3">
                <Text className="text-center text-sm font-medium text-rose-500">{error}</Text>
              </View>
            ) : null}

            {!sent ? (
              <View className="mb-6 gap-4">
                <Controller
                  control={control}
                  name="email"
                  render={({ field: { onChange, onBlur, value } }) => (
                    <View>
                      <TextInput
                        placeholder="Email"
                        keyboardType="email-address"
                        autoCapitalize="none"
                        onBlur={onBlur}
                        onChangeText={onChange}
                        value={value}
                        className="h-12 rounded-xl px-4 text-base"
                        style={Style.inputStyle(isDarkColorScheme, accentSet)}
                        placeholderTextColor={Style.getPlaceholderColor(
                          isDarkColorScheme,
                          accentSet
                        )}
                      />
                      {errors.email ? (
                        <Text className="mt-1 text-xs text-rose-500">{errors.email.message}</Text>
                      ) : null}
                    </View>
                  )}
                />
                <FormButton buttonText="Send Verification Link" onPress={handleSubmit(onSubmit)} />
              </View>
            ) : null}

            <TouchableOpacity onPress={() => router.back()} className="mt-4 items-center">
              <Text className="text-sm font-medium text-white">Go back to Sign In</Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </LinearGradient>
  );
};

export default ResendVerificationScreen;
