import { View, Text, TextInput, TouchableOpacity, Platform } from 'react-native';
import { Button } from '~/components/nativewindui/Button';
import { useColorScheme } from '~/lib/useColorScheme';
import React, { useState } from 'react';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { registerSchema, RegisterFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import * as Style from '../styles/Styles';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import GithubButton from '../components/GithubButton';
import GoogleButton from '../components/GoogleButton';
import FormButton from '../components/FormButton';

const isWeb = Platform.OS === 'web';

const RegisterScreen = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const setCredentials = useAuthStore((s) => s.setCredentials);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState(false);

  const {
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<RegisterFormData>({
    resolver: zodResolver(registerSchema),
    defaultValues: { username: '', email: '', password: '' },
  });

  const onSubmit = async (data: RegisterFormData) => {
    setLoading(true);
    setError('');
    try {
      await api.post('/api/v1/registration', {
        username: data.username,
        email: data.email,
        password: data.password,
      });
      try {
        if (isWeb) {
          await api.post('/api/v1/login', {
            username: data.username,
            password: data.password,
            device_name: 'Web',
          });
          const userRes = await api.get<UserResponse>('/api/v1/user');
          setCredentials('', userRes.data);
        } else {
          const res = await api.post('/auth/exchange', {
            username: data.username,
            password: data.password,
            device_name: 'Mobile',
          });
          setCredentials(res.data.token, res.data.user);
        }
        setSuccess(true);
      } catch {
        setError('Registration successful but login failed. Please sign in manually.');
      }
    } catch (err: any) {
      if (err.response?.data?.message) {
        setError(err.response.data.message);
      } else {
        setError('Registration failed. Please try again.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <View className="flex-1 items-center justify-center px-4">
      <View
        className={`w-full max-w-[380px] rounded-xl p-6 shadow-lg`}
        style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
        <View>
          <Text
            className={`mb-6 text-center text-xl font-semibold ${
              isDarkColorScheme ? 'text-zinc-100' : 'text-zinc-800'
            }`}>
            Create a new account
          </Text>
        </View>

        {success ? (
          <View className="mb-4 rounded-lg bg-emerald-500/20 p-3">
            <Text className="text-center text-sm text-emerald-500">
              Account created successfully!
            </Text>
          </View>
        ) : null}

        {error ? (
          <View className="mb-4 rounded-lg bg-rose-500/20 p-3">
            <Text className="text-center text-sm text-rose-500">{error}</Text>
          </View>
        ) : null}

        <View className="mb-6 gap-4">
          <Controller
            control={control}
            name="username"
            render={({ field: { onChange, onBlur, value } }) => (
              <View>
                <TextInput
                  placeholder="Username"
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  className={`h-12 rounded-lg border px-4 text-base`}
                  style={Style.inputStyle(isDarkColorScheme, accentSet)}
                  placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
                />
                {errors.username ? (
                  <Text className="mt-1 text-xs text-rose-500">{errors.username.message}</Text>
                ) : null}
              </View>
            )}
          />

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
                  className={`h-12 rounded-lg border px-4 text-base`}
                  style={Style.inputStyle(isDarkColorScheme, accentSet)}
                  placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
                />
                {errors.email ? (
                  <Text className="mt-1 text-xs text-rose-500">{errors.email.message}</Text>
                ) : null}
              </View>
            )}
          />

          <Controller
            control={control}
            name="password"
            render={({ field: { onChange, onBlur, value } }) => (
              <View>
                <TextInput
                  placeholder="Password"
                  secureTextEntry
                  onBlur={onBlur}
                  onChangeText={onChange}
                  value={value}
                  className={`h-12 rounded-lg border px-4 text-base`}
                  style={Style.inputStyle(isDarkColorScheme, accentSet)}
                  placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
                />
                {errors.password ? (
                  <Text className="mt-1 text-xs text-rose-500">{errors.password.message}</Text>
                ) : null}
              </View>
            )}
          />
        </View>

        <View>
          <FormButton buttonText="Register" onPress={handleSubmit(onSubmit)} />
        </View>

        <View className="relative my-6">
          <View className="absolute inset-0 flex items-center justify-center">
            <View
              className={`h-[1px] w-full ${isDarkColorScheme ? 'bg-zinc-700' : 'bg-gray-200'}`}
            />
          </View>
          <View className="relative flex flex-row justify-center">
            <Text
              className={`bg-inherit px-4 text-sm`}
              style={{ backgroundColor: colors.card, color: colors.foreground }}>
              or continue with
            </Text>
          </View>
        </View>

        <View className="gap-4">
          <GoogleButton />
          <GithubButton />
        </View>
      </View>
    </View>
  );
};

export default RegisterScreen;
