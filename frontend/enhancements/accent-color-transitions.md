# Accent Color Transition System Implementation Plan

## Technical Foundations

- **react-native-reanimated v3.16.2** already installed (package.json line 55)
- Existing color system in `theme/colors.ts`
- Accent management via `useAccentColor` hook

## Core Components Requiring Animation

1. `AccentColorButton` (color selection)
2. `Slider` thumb (nativewindui/Slider.tsx)
3. `ProgressIndicator` (nativewindui/ProgressIndicator.tsx)
4. Form buttons (Login/Register screens)
5. Interactive controls (CounterControls)

## Animation Strategy

### 1. Transition Types

```typescript
// Color transitions
const colorStyle = useAnimatedStyle(() => ({
  backgroundColor: withTiming(accentSet.value.base, { duration: 300 }),
}));

// Scale effects
const scaleStyle = useAnimatedStyle(() => ({
  transform: [{ scale: withSpring(isActive ? 1.1 : 1) }],
}));

// Border/shadow interpolations
const borderStyle = useAnimatedStyle(() => ({
  borderColor: withTiming(accentSet.value.border),
  shadowColor: withTiming(accentSet.value.shadow),
}));
```

### 2. Shared Utilities (Styles.tsx)

```typescript
export const accentTransitionPreset = {
  duration: 300,
  easing: Easing.inOut(Easing.ease),
};

export const createAnimatedAccentStyle = (accentValue: Animated.SharedValue<string>) =>
  useAnimatedStyle(() => ({
    backgroundColor: withTiming(accentValue.value, accentTransitionPreset),
  }));
```

## Implementation Phases

1. **Component Preparation**

   - Wrap target components with `Animated.View`
   - Create dedicated animation hooks
   - Establish baseline performance metrics

2. **Animation Integration**

   ```typescript
   // components/AccentColorButton.tsx
   const AnimatedButton = Animated.createAnimatedComponent(TouchableOpacity);

   const animatedStyle = useAnimatedStyle(() => ({
     backgroundColor: withTiming(accentSet.value.base),
     borderColor: withTiming(accentSet.value.border),
   }));
   ```

3. **Screen Coordination**
   - Sync animations across ControlsScreen components
   - Implement batch updates for color changes
   - Add debug mode toggle for animation inspection

## Performance Considerations

- Memoize animated styles with `useMemo`
- Use shared values from `useAccentColor` hook
- Platform-specific optimizations:
  ```typescript
  const duration = Platform.select({
    ios: 300,
    android: 400,
    default: 300,
  });
  ```

## Validation Checklist

- [ ] 60fps maintained during color transitions
- [ ] Memory usage < 5MB increase
- [ ] Android/iOS behavior parity
- [ ] Accessibility scaling support
- [ ] Dark mode transitions validated
