import { View, Text, TextInput, TouchableOpacity } from 'react-native';
import { useForm, Controller } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import React, { useState } from 'react';
import { useNavigation } from '@react-navigation/native';
import { resendVerificationSchema, ResendVerificationFormData } from '~/app/lib/schemas';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import FormButton from '~/app/components/FormButton';
import * as Style from '~/app/styles/Styles';
import { DrawerNavigationProp } from '@react-navigation/drawer';
import { DrawerParamList, navigateWithTitle } from '~/types/navigation';

const ResendVerificationScreen = () => {
  const navigation = useNavigation<DrawerNavigationProp<DrawerParamList>>();
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
      await api.post('/email/verify/resend', { email: data.email });
      setSent(true);
    } catch {
      setError('Failed to send verification email. Please try again.');
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
            Resend Verification Email
          </Text>
          <Text
            className={`mb-6 text-center text-sm ${Style.getSecondaryTextColor(isDarkColorScheme, accentSet)}`}>
            Enter your email address and we'll send you a new verification link
          </Text>
        </View>

        {sent ? (
          <View className="mb-4 rounded-lg bg-emerald-500/20 p-3">
            <Text className="text-center text-sm text-emerald-500">
              Check your email for the verification link.
            </Text>
          </View>
        ) : null}

        {error ? (
          <View className="mb-4 rounded-lg bg-rose-500/20 p-3">
            <Text className="text-center text-sm text-rose-500">{error}</Text>
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
            <FormButton buttonText="Send Verification Link" onPress={handleSubmit(onSubmit)} />
          </View>
        ) : null}

        <TouchableOpacity onPress={() => navigateWithTitle(() => navigation.navigate('Account', { screen: 'SignIn' }), 'Sign In')} className="mt-2 self-center">
          <Text className={`text-sm ${Style.getSecondaryTextColor(isDarkColorScheme, accentSet)}`}>
            Go back
          </Text>
        </TouchableOpacity>
      </View>
    </View>
  );
};

export default ResendVerificationScreen;
