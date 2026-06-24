import { useRouter } from 'expo-router';
import React, { useEffect, useState } from 'react';
import {
  Text,
  TextInput,
  TouchableOpacity,
  View,
  Alert,
  Platform,
  Image,
  ScrollView,
  useWindowDimensions,
} from 'react-native';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { loginSchema, LoginFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useAuthStore, AuthUser, UserResponse } from '~/app/stores/AuthStore';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import FormButton from '../../components/FormButton';
import GithubButton from '../../components/GithubButton';
import GoogleButton from '../../components/GoogleButton';
import * as Style from '../../styles/Styles';
import { LinearGradient } from 'expo-linear-gradient';
import { Ionicons } from '@expo/vector-icons';
import { BaseAccentGradients } from '~/theme/colors';

const isWeb = Platform.OS === 'web';

const LoginScreen = () => {
  const router = useRouter();
  const { width } = useWindowDimensions();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const setCredentials = useAuthStore((s) => s.setCredentials);
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  useEffect(() => {
    if (isAuthenticated) {
      router.replace('/pet-profiles');
    }
  }, [isAuthenticated, router]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const {
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: 'testuser1', password: 'password2' },
  });

  const onSubmit = async (data: LoginFormData) => {
    setLoading(true);
    setError('');
    try {
      if (isWeb) {
        await api.post('/api/v1/auth/login', {
          ...data,
          device_name: 'Web',
        });
        const userRes = await api.get<UserResponse>('/api/v1/private/user');
        setCredentials('', userRes.data);
       } else {
         const res = await api.post('/api/v1/auth/exchange', {
           ...data,
           device_name: 'Mobile',
         });
          setCredentials(res.data.token, res.data.user);
        }
        Alert.alert('Debug', `Login successful: ${data.username}`);
        router.replace('/pet-profiles');
    } catch (err: any) {
      if (err.response?.status === 401) {
        setError('Invalid credentials');
      } else {
        setError('Login failed. Please try again.');
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
              Welcome back!
            </Text>
            <Text className="mt-1 text-base text-white/80">Let the tails wag again</Text>
          </View>

          <View
            className="rounded-2xl p-6 shadow-xl web:w-1/2"
            style={{
              backgroundColor: isDarkColorScheme ? 'rgba(35,25,22,0.95)' : 'rgba(255,255,255,0.95)',
              backdropFilter: 'blur(10px)',
            }}>
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

            <FormButton buttonText="Sign In" onPress={handleSubmit(onSubmit)} />

            <TouchableOpacity
              onPress={() => router.push('/auth/forgot-password')}
              className="mt-4 items-center">
              <Text className="text-sm font-medium text-[#F4644E]">Forgot your password?</Text>
            </TouchableOpacity>

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
                onPress={() => router.push('/register')}>
                <Text className="text-sm">
                  <Text className="text-[#8B7368]">New to PetMatch? </Text>
                  <Text className="font-semibold text-[#F4644E]">Create account</Text>
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </View>
      </ScrollView>
    </LinearGradient>
  );
};

export default LoginScreen;
