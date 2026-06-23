import { useColorScheme as useNativewindColorScheme } from 'nativewind';
import { useEffect } from 'react';
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
    // Edge-to-edge handles navigation bar automatically on Android (SDK 55+)
  }, [colorScheme, accentSet, accentColor]);

  const setColorScheme = (colorScheme: 'light' | 'dark') => {
    setNativeWindColorScheme(colorScheme);
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

export { useColorScheme };
