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
      await api.post('/password-reset/complete', {
        token,
        new_password: data.new_password,
      });
      setSuccess(true);
    } catch {
      setError('Failed to reset password. Please try again.');
    }
  };

  return (
    <View className="flex-1 items-center justify-center px-4">
      <View
        className={`w-full max-w-[380px] rounded-xl p-6 shadow-lg`}
        style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
        <View>
          <Text
            className={`mb-2 text-center text-xl font-semibold ${Style.getHeadingTextColor(isDarkColorScheme, accentSet)}`}>
            Reset Password
          </Text>
          <Text
            className={`mb-6 text-center text-sm ${Style.getSecondaryTextColor(isDarkColorScheme, accentSet)}`}>
            Enter your new password
          </Text>
        </View>

        {success ? (
          <View className="mb-4 rounded-lg bg-emerald-500/20 p-3">
            <Text className="text-center text-sm text-emerald-500">
              Password reset successful! You can now sign in.
            </Text>
          </View>
        ) : null}

        {error ? (
          <View className="mb-4 rounded-lg bg-rose-500/20 p-3">
            <Text className="text-center text-sm text-rose-500">{error}</Text>
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
                    className={`h-12 rounded-lg border px-4 text-base`}
                    style={Style.inputStyle(isDarkColorScheme, accentSet)}
                    placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
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
                    className={`h-12 rounded-lg border px-4 text-base`}
                    style={Style.inputStyle(isDarkColorScheme, accentSet)}
                    placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
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

        <TouchableOpacity onPress={() => navigateWithTitle(() => navigation.navigate('Account', { screen: 'SignIn' }), 'Sign In')} className="mt-2 self-center">
          <Text className={`text-sm ${Style.getSecondaryTextColor(isDarkColorScheme, accentSet)}`}>
            Back to Sign In
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );
};

export default ResetPasswordScreen;
