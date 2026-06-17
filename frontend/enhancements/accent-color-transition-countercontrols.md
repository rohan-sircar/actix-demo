# CounterControls Component Transition Implementation Plan

## Current State Analysis

The CounterControls component currently:

- Uses static accent colors for card and button styling
- Implements accent color selection through AccentColorButton
- Relies on cardStyle and formButton style functions

## Transition Implementation Plan

### 1. Component Modifications

```typescript
// Before: Static View
<View style={[styles.section, cardStyle(isDarkColorScheme, colors, accentSet)]}>

// After: Animated View with transitions
const Animated = require('react-native-reanimated');
const AnimatedView = Animated.createAnimatedComponent(View);

const cardAnimatedStyle = useAnimatedStyle(() => ({
  backgroundColor: withTiming(accentSet.base, { duration: 300 }),
  borderColor: withTiming(accentSet.border, { duration: 300 }),
  shadowColor: withTiming(accentSet.shadow, { duration: 300 })
}));

<AnimatedView style={[styles.section, cardAnimatedStyle]}>
2. Button Transitions
const buttonAnimatedStyle = useAnimatedStyle(() => ({
  backgroundColor: withTiming(accentSet.base, { duration: 300 }),
  borderColor: withTiming(accentSet.border, { duration: 300 })
}));

const buttonTextAnimatedStyle = useAnimatedStyle(() => ({
  color: withTiming(isDarkColorScheme ? '#fff' : accentSet.base, { duration: 300 })
}));
3. Implementation Steps
Convert static styles to animated:

Card background, border, and shadow
Button background and border
Text colors
Create shared transition utilities:

export const createAccentTransition = (value: string) =>
  withTiming(value, {
    duration: 300,
    easing: Easing.inOut(Easing.ease)
  });
Add press feedback animations:

const pressAnimatedStyle = useAnimatedStyle(() => ({
  transform: [{ scale: withSpring(pressed.value ? 0.98 : 1) }]
}));
4. Performance Optimizations
Memoize animated styles:

const animatedStyles = useMemo(() => ({
  card: cardAnimatedStyle,
  button: buttonAnimatedStyle,
  text: buttonTextAnimatedStyle
}), [accentSet.base, accentSet.border]);
Use worklets for animation calculations:

const calculateButtonStyle = worklet((accent: string) => {
  'worklet';
  return { backgroundColor: accent };
});
Share animated values when possible:

const sharedAccentValue = useSharedValue(accentSet.base);
5. Testing Strategy
Visual Testing:

Verify smooth transitions between all accent colors
Check for any flickering or jank
Test dark/light mode transitions
Performance Testing:

Monitor frame drops during transitions
Test on low-end Android devices
Verify memory usage stays stable
Edge Cases:

Rapid accent color changes
Component unmounting during transition
Different screen sizes
Changes Required
Add imports:
import Animated, {
  useAnimatedStyle,
  withTiming,
  withSpring,
  useSharedValue
} from 'react-native-reanimated';
Create animated component wrappers
Implement animated styles
Add performance optimizations
Update prop types for animated components
Validation Checklist
[ ] Smooth 60fps transitions between all accent colors
[ ] No visual artifacts during transitions
[ ] Proper cleanup on component unmount
[ ] Consistent behavior across iOS and Android
[ ] Performance metrics within acceptable range:
Frame rate remains at 60fps
No memory leaks
CPU usage spike < 10%
```
