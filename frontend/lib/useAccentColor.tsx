import { create } from 'zustand';
import { createJSONStorage, persist } from 'zustand/middleware';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { AccentColors, AccentColorType } from '~/theme/colors';

export const accentColorTypeKeys: AccentColorType[] = Object.values(
  AccentColorType
) as AccentColorType[];

export const ACCENT_KEY = 'ACCENT_COLOR';

interface AccentColorState {
  readonly accentColor: AccentColorType;
  setAccentColor: (color: AccentColorType) => void;
}

export const useAccentColor = create<AccentColorState>()(
  persist(
    (set) => ({
      accentColor: AccentColorType.BLUE,
      setAccentColor: (color: AccentColorType) => {
        set({ accentColor: color });
      },
    }),
    {
      name: ACCENT_KEY,
      storage: createJSONStorage(() => AsyncStorage),
    }
  )
);

// Helper function to get an accent color set.
export const getAccentSet = (color: AccentColorType) => {
  return AccentColors.get(color)!;
};
