import React from 'react';
import { Stack } from 'expo-router';

export default function LikesLayout() {
  return (
    <Stack screenOptions={{ headerShown: false }}>
      <Stack.Screen name="index" />
      <Stack.Screen name="user-profile/[user_uuid]" options={{ title: 'Profile' }} />
    </Stack>
  );
}
