import { Stack } from 'expo-router';
import React from 'react';
import { useColorScheme } from '~/lib/useColorScheme';

export default function SettingsLayout() {
  const { colors } = useColorScheme();
  return (
    <Stack
      screenOptions={{
        headerShown: true,
        headerStyle: { backgroundColor: colors.background },
        headerTintColor: colors.text,
      }}
    >
      <Stack.Screen name="index" options={{ headerShown: false }} />
      <Stack.Screen
        name="sessions"
        options={{
          title: 'Active Sessions',
          presentation: 'card',
        }}
      />
    </Stack>
  );
}
