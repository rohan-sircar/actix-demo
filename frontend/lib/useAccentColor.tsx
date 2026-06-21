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
      accentColor: AccentColorType.CORAL,
      setAccentColor: (color: AccentColorType) => {
        set({ accentColor: color });
      },
    }),
    {
      name: ACCENT_KEY,
      storage: createJSONStorage(() => AsyncStorage),
      version: 2,
      migrate: (_persistedState: any) => {
        const state = _persistedState as { accentColor?: string };
        if (
          state &&
          state.accentColor &&
          accentColorTypeKeys.includes(state.accentColor as AccentColorType)
        ) {
          return state;
        }
        return { accentColor: AccentColorType.CORAL };
      },
    }
  )
);

// Helper function to get an accent color set.
export const getAccentSet = (color: AccentColorType) => {
  return AccentColors.get(color) ?? AccentColors.get(AccentColorType.CORAL)!;
};
