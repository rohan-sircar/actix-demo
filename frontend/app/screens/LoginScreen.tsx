import { DrawerNavigationProp } from '@react-navigation/drawer';
import { useNavigation } from '@react-navigation/native';
import React, { useState } from 'react';
import { Text, TextInput, TouchableOpacity, View, Alert, Platform } from 'react-native';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { loginSchema, LoginFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useAuthStore, AuthUser, UserResponse } from '~/app/stores/AuthStore';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import { DrawerParamList, navigateWithTitle } from '~/types/navigation';
import FormButton from '../components/FormButton';
import GithubButton from '../components/GithubButton';
import GoogleButton from '../components/GoogleButton';
import * as Style from '../styles/Styles';

const isWeb = Platform.OS === 'web';

const LoginScreen = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const navigation = useNavigation<DrawerNavigationProp<DrawerParamList>>();
  const setCredentials = useAuthStore((s) => s.setCredentials);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const {
    control,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
    defaultValues: { username: '', password: '' },
  });

  const onSubmit = async (data: LoginFormData) => {
    setLoading(true);
    setError('');
    try {
      if (isWeb) {
        await api.post('/api/v1/login', {
          ...data,
          device_name: 'Web',
        });
        const userRes = await api.get<UserResponse>('/api/v1/user');
        setCredentials('', userRes.data);
      } else {
        const res = await api.post('/auth/exchange', {
          ...data,
          device_name: 'Mobile',
        });
        setCredentials(res.data.token, res.data.user);
      }
      navigateWithTitle(() => navigation.navigate('Home', { screen: 'Feed' }), 'Home');
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

  return (
    <View className={`flex-1 items-center justify-center px-4`}>
      <View
        className={`w-full max-w-[380px] rounded-xl p-6 shadow-lg`}
        style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
        <View>
          <Text
            className={`mb-6 text-center text-xl font-semibold ${Style.getHeadingTextColor(isDarkColorScheme, accentSet)}`}>
            Sign in to your account
          </Text>
        </View>

        <View className="mb-4 gap-4">
          {error ? (
            <View className="rounded-lg bg-rose-500/20 p-3">
              <Text className="text-center text-sm text-rose-500">{error}</Text>
            </View>
          ) : null}

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
          <FormButton buttonText="Submit" onPress={handleSubmit(onSubmit)} />
          <TouchableOpacity
            onPress={() =>
              navigateWithTitle(
                () => navigation.navigate('Account', { screen: 'ForgotPassword' }),
                'Forgot Password'
              )
            }>
            <Text
              className={`mt-4 text-center text-sm ${Style.getSecondaryTextColor(isDarkColorScheme, accentSet)}`}>
              Forgot Password?
            </Text>
          </TouchableOpacity>
        </View>

        <View className="relative my-6">
          <View className="absolute inset-0 flex items-center justify-center">
            <View
              className={`h-[1px] w-full ${Style.getDividerColor(isDarkColorScheme, accentSet)}`}
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

export default LoginScreen;
