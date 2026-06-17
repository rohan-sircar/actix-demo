import React from 'react';
import { View } from 'react-native';
import { Text } from 'react-native';
import { cardStyle } from '../styles/Styles';
import { useColorScheme } from '~/lib/useColorScheme';
import { Button } from '~/components/nativewindui/Button';
import * as Style from '../styles/Styles';
import { accentColorTypeKeys, getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import AccentColorButton from '~/components/AccentColorButton';
import FormButton from './FormButton';
import { useAuthStore } from '../stores/AuthStore';

const CounterControls = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const user = useAuthStore((state) => state.user);
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);

  return (
    <View className="my-4 p-4" style={cardStyle(isDarkColorScheme, colors, accentSet)}>
      <Text className="mb-4 text-2xl font-semibold tracking-wider" style={{ color: colors.text }}>
        Account Info
      </Text>
      {isAuthenticated && user ? (
        <>
          <Text className="mb-2 text-center text-base" style={{ color: colors.text }}>
            Logged in as: {user.username}
          </Text>
          <Text className="mb-3 text-center text-base" style={{ color: colors.text }}>
            Email: {user.email}
          </Text>
          <Text className="mb-3 text-center text-base" style={{ color: colors.text }}>
            User ID: {user.id}
          </Text>
        </>
      ) : (
        <Text className="mb-3 text-center text-base text-gray-500">Not logged in</Text>
      )}
      <View className="flex-col gap-4">
        <View className="flex-row justify-between">
          {accentColorTypeKeys.map((colorKey) => (
            <AccentColorButton key={colorKey} color={colorKey} />
          ))}
        </View>
      </View>
    </View>
  );
};

export default CounterControls;
