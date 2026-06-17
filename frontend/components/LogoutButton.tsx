import React from 'react';
import FontAwesome from '@expo/vector-icons/FontAwesome';
import { Pressable } from 'react-native';
import { useColorScheme } from '../lib/useColorScheme';
import { useAuthStore } from '~/app/stores/AuthStore';
import api from '~/app/lib/api';

export function LogoutButton() {
  const { colors, isDarkColorScheme } = useColorScheme();

  return (
    <Pressable
      onPress={async () => {
        try {
          await api.post('/api/v1/logout');
        } catch {
          // ignore logout errors - still clear local state
        }
        await useAuthStore.getState().clearCredentials();
      }}
      style={{ marginRight: 15 }}>
      {({ pressed }) => (
        <FontAwesome
          name="sign-out"
          size={24}
          style={[
            {
              opacity: pressed ? 0.5 : 1,
              color: colors.text,
            },
          ]}
        />
      )}
    </Pressable>
  );
}
