import { View, Text, TextInput, TouchableOpacity } from 'react-native';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import React, { useState } from 'react';
import { resetPasswordSchema, ResetPasswordFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import FormButton from '../components/FormButton';
import * as Style from '../styles/Styles';
import { DrawerNavigationProp } from '@react-navigation/drawer';
import { useNavigation } from '@react-navigation/native';
import { DrawerParamList, navigateWithTitle } from '~/types/navigation';
import { LinearGradient } from 'expo-linear-gradient';
import { Ionicons } from '@expo/vector-icons';
import { BaseAccentGradients } from '~/theme/colors';

type ResetPasswordScreenProps = {
  route: {
    params: {
      token?: string;
    };
  };
};

const ResetPasswordScreen = ({ route }: ResetPasswordScreenProps) => {
  const navigation = useNavigation<DrawerNavigationProp<DrawerParamList>>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState('');

  const token = route?.params?.token || '';

  const {
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<ResetPasswordFormData>({
    resolver: zodResolver(resetPasswordSchema),
    defaultValues: { new_password: '', confirm_password: '' },
  });

  const onSubmit = async (data: ResetPasswordFormData) => {
    setError('');
    if (!token) {
      setError('Invalid reset token');
      return;
    }
    try {
      await api.post('/api/v1/auth/password-reset-complete', {
        token,
        new_password: data.new_password,
      });
      setSuccess(true);
    } catch {
      setError('Failed to reset password. Please try again.');
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
              <Ionicons name="key" size={32} color="white" />
            </View>
            <Text className="text-3xl font-bold tracking-tight text-white">Set New Password</Text>
            <Text className="mt-1 text-base text-white/80">Almost there, just one more step</Text>
          </View>

          <View
            className="rounded-2xl p-6 shadow-xl"
            style={{
              backgroundColor: isDarkColorScheme ? 'rgba(35,25,22,0.95)' : 'rgba(255,255,255,0.95)',
            }}>
            {success ? (
              <View className="mb-4 rounded-xl bg-emerald-500/15 p-3">
                <Text className="text-center text-sm font-medium text-emerald-600">
                  Password reset successful! You can now sign in.
                </Text>
              </View>
            ) : null}

            {error ? (
              <View className="mb-4 rounded-xl bg-rose-500/15 p-3">
                <Text className="text-center text-sm font-medium text-rose-500">{error}</Text>
              </View>
            ) : null}

            {!success ? (
              <View className="mb-6 gap-4">
                <Controller
                  control={control}
                  name="new_password"
                  render={({ field: { onChange, onBlur, value } }) => (
                    <View>
                      <TextInput
                        placeholder="New Password"
                        secureTextEntry
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
                      {errors.new_password ? (
                        <Text className="mt-1 text-xs text-rose-500">
                          {errors.new_password.message}
                        </Text>
                      ) : null}
                    </View>
                  )}
                />

                <Controller
                  control={control}
                  name="confirm_password"
                  render={({ field: { onChange, onBlur, value } }) => (
                    <View>
                      <TextInput
                        placeholder="Confirm New Password"
                        secureTextEntry
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
                      {errors.confirm_password ? (
                        <Text className="mt-1 text-xs text-rose-500">
                          {errors.confirm_password.message}
                        </Text>
                      ) : null}
                    </View>
                  )}
                />

                <FormButton buttonText="Reset Password" onPress={handleSubmit(onSubmit)} />
              </View>
            ) : null}

            <TouchableOpacity
              onPress={() =>
                navigateWithTitle(
                  () => navigation.navigate('Account', { screen: 'SignIn' }),
                  'Sign In'
                )
              }
              className="mt-4 items-center">
              <Text className="text-sm font-medium text-[#F4644E]">Back to Sign In</Text>
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </LinearGradient>
  );
};

export default ResetPasswordScreen;
