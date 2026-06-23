import { View, Text, TextInput, Platform, ScrollView, useWindowDimensions } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import React, { useEffect, useState } from 'react';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { registerSchema, RegisterFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import * as Style from '../../styles/Styles';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import GithubButton from '../../components/GithubButton';
import GoogleButton from '../../components/GoogleButton';
import FormButton from '../../components/FormButton';
import { LinearGradient } from 'expo-linear-gradient';
import { Ionicons } from '@expo/vector-icons';
import { TouchableOpacity } from 'react-native';
import { useRouter } from 'expo-router';
import { BaseAccentGradients } from '~/theme/colors';

const isWeb = Platform.OS === 'web';

const RegisterScreen = () => {
  const router = useRouter();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const { width } = useWindowDimensions();
  const setCredentials = useAuthStore((s) => s.setCredentials);
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  useEffect(() => {
    if (isAuthenticated) {
      router.replace('/pet-profiles');
    }
  }, [isAuthenticated, router]);
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
      await api.post('/api/v1/auth/registration', {
        username: data.username,
        email: data.email,
        password: data.password,
      });
      try {
        if (isWeb) {
          await api.post('/api/v1/auth/login', {
            username: data.username,
            password: data.password,
            device_name: 'Web',
          });
          const userRes = await api.get<UserResponse>('/api/v1/private/user');
          setCredentials('', userRes.data);
        } else {
          const res = await api.post('/api/v1/auth/exchange', {
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

  const gradientColors = BaseAccentGradients[accentColor];
  const scale = Math.min(width / 400, 1.2);
  const iconSize = Math.round(20 * scale);
  const iconContainerSize = Math.round(56 * scale);
  const headingFontSize = Math.round(20 * scale);

  return (
    <LinearGradient
      colors={[gradientColors.gradientStart, gradientColors.gradientEnd]}
      style={{ flex: 1 }}>
      <ScrollView
        className="flex-1 px-6 web:px-12"
        contentContainerStyle={{ flexGrow: 1, justifyContent: 'center' }}
        keyboardShouldPersistTaps="handled">
        <View className="mx-auto w-full max-w-md web:max-w-4xl web:flex-row web:items-center web:gap-4 web:px-6">
          <View className="mb-6 items-center web:mb-0 web:w-1/2 web:items-center web:text-center">
            <View
              className="mb-3 items-center justify-center rounded-full"
              style={{
                width: iconContainerSize,
                height: iconContainerSize,
                backgroundColor: 'rgba(255,255,255,0.2)',
              }}>
              <Ionicons name="paw" size={iconSize} color="white" />
            </View>
            <Text
              style={{ fontSize: headingFontSize }}
              className="font-bold tracking-tight text-white">
              Join PetMatch
            </Text>
            <Text className="mt-1 text-base text-white/80">Let's set up your profile</Text>
          </View>

          <View
            className="rounded-2xl p-6 shadow-xl web:w-1/2"
            style={{
              backgroundColor: isDarkColorScheme ? 'rgba(35,25,22,0.95)' : 'rgba(255,255,255,0.95)',
            }}>
            {success ? (
              <View className="mb-4 rounded-xl bg-emerald-500/15 p-3">
                <Text className="text-center text-sm font-medium text-emerald-600">
                  Account created successfully! Welcome to the pack!
                </Text>
              </View>
            ) : null}

            {error ? (
              <View className="mb-4 rounded-xl bg-rose-500/15 p-3">
                <Text className="text-center text-sm font-medium text-rose-500">{error}</Text>
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
                      className="h-12 rounded-xl px-4 text-base"
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
                      className="h-12 rounded-xl px-4 text-base"
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
                      className="h-12 rounded-xl px-4 text-base"
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

            <FormButton buttonText="Create Account" onPress={handleSubmit(onSubmit)} />

            <View className="my-6 items-center">
              <Text
                className="px-4 text-sm font-medium"
                style={{
                  color: colors.grey,
                }}>
                or continue with
              </Text>
            </View>

            <View className="gap-3">
              <GoogleButton />
              <GithubButton />
            </View>

            <View className="mt-5 items-center">
              <TouchableOpacity
                onPress={() => router.replace('/auth/sign-in')}>
                <Text className="text-sm">
                  <Text className="text-[#8B7368]">Already have an account? </Text>
                  <Text className="font-semibold text-[#F4644E]">Sign in</Text>
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </ScrollView>
    </LinearGradient>
  );
};

export default RegisterScreen;
