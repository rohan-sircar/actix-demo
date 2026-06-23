import { View, Text, TextInput, TouchableOpacity } from 'react-native';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import React, { useState } from 'react';
import { forgotPasswordSchema, ForgotPasswordFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import FormButton from '../components/FormButton';
import * as Style from '../styles/Styles';
import { useRouter } from 'expo-router';
import { LinearGradient } from 'expo-linear-gradient';
import { Ionicons } from '@expo/vector-icons';
import { BaseAccentGradients } from '~/theme/colors';

const ForgotPasswordScreen = () => {
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
  } = useForm<ForgotPasswordFormData>({
    resolver: zodResolver(forgotPasswordSchema),
    defaultValues: { email: '' },
  });

  const onSubmit = async (data: ForgotPasswordFormData) => {
    setError('');
    try {
      await api.post('/api/v1/auth/password-reset-request', { email: data.email });
      setSent(true);
    } catch {
      setError('Failed to send reset email. Please try again.');
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
              <Ionicons name="lock-closed" size={32} color="white" />
            </View>
            <Text className="text-3xl font-bold tracking-tight text-white">Forgot Password?</Text>
            <Text className="mt-1 text-center text-base text-white/80">
              No worries, we'll help you reset it
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
                  Check your email for the reset link!
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
                <Text className="mb-2 text-center text-sm" style={{ color: colors.grey }}>
                  Enter your email and we'll send you a reset link
                </Text>
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
                <FormButton buttonText="Send Reset Link" onPress={handleSubmit(onSubmit)} />
              </View>
            ) : null}

            <TouchableOpacity
              onPress={() => router.replace('/auth/sign-in')}
              className="mt-4 items-center">
              <Text className="text-sm font-medium text-[#F4644E]">
                Remember your password? Sign in
              </Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </LinearGradient>
  );
};

export default ForgotPasswordScreen;
