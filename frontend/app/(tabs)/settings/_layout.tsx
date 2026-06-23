import { Stack } from 'expo-router';
import React from 'react';

export default function SettingsLayout() {
  return (
    <Stack screenOptions={{ headerShown: true }}>
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
