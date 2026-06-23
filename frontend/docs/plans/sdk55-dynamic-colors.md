# SDK 55 — Navigation Bar Cleanup + Dynamic Colors

**Date:** 2026-06-23
**From SDK:** 54
**To SDK:** 55

## Goal
Replace deprecated `expo-navigation-bar` calls with the new SDK 55 Colors API to get native Material 3 dynamic colors on Android (colors adapt to user's wallpaper), while cleaning up all deprecated APIs.

## Scope
- Replace hardcoded Android system colors with Material 3 dynamic colors via `expo-router`'s `Color` API
- Remove all deprecated `NavigationBar` method calls
- Simplify `StatusBar` usage for edge-to-edge mode
- Bump all Expo packages to SDK 55 versioning scheme
- Remove `newArchEnabled` from app.json (mandatory in SDK 55)

## Files to modify (6 files)

### 1. `theme/colors.ts` — Android colors updated to Material 3 design language
- Android colors replaced with Material 3 static color roles (hex values)
- Colors match Material 3 spec: `#6750a4` primary, `#1c1b1f` onSurface text, etc.
- iOS keeps existing hardcoded colors
- Created `ANDROID_STATIC_MATERIAL_COLORS` export for ripple effect compatibility
- Mapping:
  - `surface` → background / root
  - `onSurface` → text / foreground
  - `primary` → primary (`#6750a4` light, `#d0bcff` dark)
  - `surfaceVariant` → card
  - `onSurfaceVariant` → grey variants (muted text, borders)
  - `error` → destructive

### 2. `lib/useColorScheme.tsx` — Remove deprecated NavigationBar calls
- Remove `import * as NavigationBar from 'expo-navigation-bar'`
- Remove `setNavigationBar()` function
- Remove `useInitialAndroidBarSync()` hook
- Clean up exports

### 3. `app/_layout.tsx` — Simplify for edge-to-edge
- Remove `useInitialAndroidBarSync()` call
- Remove `backgroundColor` prop from `<StatusBar>` (deprecated with edge-to-edge)

### 4. `app/modal.tsx` — Simplify StatusBar
- Remove `backgroundColor` prop from `<StatusBar>` (deprecated with edge-to-edge)

### 5. `app.json` — Remove deprecated config
- Remove `newArchEnabled: true` (config option removed in SDK 55, New Arch is mandatory)

### 6. `package.json` — Bump all Expo packages to SDK 55
Version bumps:
- `expo`: ~54.0.0 → ~55.0.0
- `expo-router`: ~6.0.24 → ~7.0.0
- `expo-constants`: ~18.0.13 → ~19.0.0
- `expo-dev-client`: ~6.0.21 → ~6.0.22 (or latest)
- `expo-device`: ~8.0.10 → ~9.0.0
- `expo-file-system`: ~19.0.23 → ~20.0.0
- `expo-font`: ~14.0.12 → ~14.0.12 (minor only)
- `expo-image-picker`: ~17.0.11 → ~18.0.0
- `expo-linear-gradient`: ~15.0.8 → ~16.0.0
- `expo-linking`: ~8.0.12 → ~9.0.0
- `expo-navigation-bar`: ~5.0.10 → ~6.0.0
- `expo-network`: ~8.0.8 → ~9.0.0
- `expo-secure-store`: ~15.0.8 → ~16.0.0
- `expo-status-bar`: ~3.0.9 → ~4.0.0
- `expo-store-review`: ~9.0.9 → ~10.0.0
- `expo-system-ui`: ~6.0.9 → ~7.0.0
- `expo-web-browser`: ~15.0.11 → ~16.0.0
- `react-native`: 0.81.5 → 0.83.x
- `react`: 19.1.0 → 19.2.x

## Post-edit steps
1. `npx expo install --fix` — align peer deps
2. `npx expo prebuild --clean` — regenerate native projects
3. `yarn install`
4. `npx tsc --noEmit` — TypeScript check

## Notes
- Android colors use Material 3 static color roles (hex values) matching the M3 spec
- True dynamic colors (`Color.android.dynamic.*`) return `OpaqueColorValue` which is incompatible with string-based APIs used throughout the app
- iOS is unaffected — keeps existing hardcoded system colors
- `StatusBar` `backgroundColor` prop is deprecated with edge-to-edge — system handles it automatically
- All `NavigationBar` methods are no-ops with mandatory edge-to-edge
