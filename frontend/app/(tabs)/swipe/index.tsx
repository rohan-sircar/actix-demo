import { useFocusEffect, useRouter } from 'expo-router';
import React, { useCallback } from 'react';
import { Platform, View } from 'react-native';
import { useAuthStore } from '../../stores/AuthStore';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import MOCK_PETS from '~/data/pets';
import { SwipeDeck } from '~/app/components/SwipeDeck';
import { SwipeDeckWeb } from '~/app/components/SwipeDeckWeb';

const isWeb = Platform.OS === 'web';

const SwipeScreen = () => {
  const router = useRouter();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  useFocusEffect(
    useCallback(() => {
      if (!isAuthenticated) {
        router.replace('/login');
      }
    }, [isAuthenticated, router])
  );

  if (isWeb) {
    return (
      <div style={{ backgroundColor: colors.background, height: '100vh', display: 'flex', flexDirection: 'column' }}>
        <SwipeDeckWeb
          pets={MOCK_PETS}
          colors={colors}
          accentSet={accentSet}
          isDarkColorScheme={isDarkColorScheme}
        />
      </div>
    );
  }

  return (
    <View style={{ backgroundColor: colors.background, flex: 1 }}>
      <SwipeDeck
        pets={MOCK_PETS}
        colors={colors}
        accentSet={accentSet}
        isDarkColorScheme={isDarkColorScheme}
      />
    </View>
  );
};

export default SwipeScreen;
