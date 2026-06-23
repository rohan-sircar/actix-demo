import { Stack } from 'expo-router';
import React from 'react';

export default function PetProfilesLayout() {
  return (
    <Stack
      screenOptions={{
        headerShown: true,
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
