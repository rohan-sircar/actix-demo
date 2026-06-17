import * as NavigationBar from 'expo-navigation-bar';
import { useColorScheme as useNativewindColorScheme } from 'nativewind';
import { useState, useEffect } from 'react';
import { Platform } from 'react-native';
import { AccentColorSet, COLORS, AccentColors, SystemColors } from '~/theme/colors';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';

interface ColorSchemeHook {
  colorScheme: 'light' | 'dark';
  isDarkColorScheme: boolean;
  setColorScheme: (colorScheme: 'light' | 'dark') => void;
  toggleColorScheme: () => void;
  colors: SystemColors;
}

function useColorScheme(): ColorSchemeHook {
  const { colorScheme: nativeWindColorScheme, setColorScheme: setNativeWindColorScheme } =
    useNativewindColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const colorScheme = nativeWindColorScheme ?? 'light';

  useEffect(() => {
    if (Platform.OS === 'android') {
      setNavigationBar(colorScheme, accentSet);
    }
  }, [colorScheme, accentSet, accentColor]);

  const setColorScheme = (colorScheme: 'light' | 'dark') => {
    setNativeWindColorScheme(colorScheme);
    if (Platform.OS === 'android') {
      setNavigationBar(colorScheme, accentSet);
    }
  };

  const toggleColorScheme = () => {
    setColorScheme(colorScheme === 'light' ? 'dark' : 'light');
  };

  const colors: SystemColors = {
    ...COLORS[colorScheme],
  };

  return {
    colorScheme,
    isDarkColorScheme: colorScheme === 'dark',
    setColorScheme,
    toggleColorScheme,
    colors,
  };
}

function setNavigationBar(colorScheme: 'light' | 'dark', accentSet: AccentColorSet) {
  return Promise.all([
    NavigationBar.setButtonStyleAsync(colorScheme === 'dark' ? 'light' : 'dark'),
    NavigationBar.setPositionAsync('absolute'),
    NavigationBar.setBackgroundColorAsync(
      colorScheme === 'dark' ? accentSet.bgSubtle : '#ffffff80'
    ),
  ]);
}

function useInitialAndroidBarSync() {
  const { colorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  useEffect(() => {
    if (Platform.OS === 'android') {
      setNavigationBar(colorScheme, accentSet).catch((error) => {
        console.error('useColorScheme.tsx", "useInitialColorScheme', error);
      });
    }
  }, [colorScheme, accentSet, accentColor]);
}

export { useColorScheme, useInitialAndroidBarSync };
