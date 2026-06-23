import { Stack } from 'expo-router';
import React from 'react';
import { useColorScheme } from '~/lib/useColorScheme';

export default function PetProfilesLayout() {
  const { colors } = useColorScheme();
  return (
    <Stack
      screenOptions={{
        headerShown: true,
        headerStyle: { backgroundColor: colors.background },
        headerTintColor: colors.text,
      }}>
      <Stack.Screen name="index" options={{ headerShown: false }} />
      <Stack.Screen
        name="[pet_uuid]/index"
        options={{
          title: 'Pet Profile',
          presentation: 'card',
        }}
      />
      <Stack.Screen
        name="[pet_uuid]/images"
        options={{
          title: 'Photos',
          presentation: 'card',
        }}
      />
      <Stack.Screen
        name="[pet_uuid]/edit"
        options={{
          title: 'Edit Pet',
          presentation: 'card',
        }}
      />
    </Stack>
  );
}
