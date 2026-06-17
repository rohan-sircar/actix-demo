# Dynamic Accent Colors System Proposal

## Overview

This document outlines a comprehensive plan to implement a dynamic accent color system that provides user-selectable UI accents while maintaining accessibility and platform-specific behaviors. The system will manage accent colors across all interactive elements while preserving the existing light/dark mode functionality.

## Objectives

- Implement 5 selectable accent colors: blue (default), red, orange, yellow, and green
- Provide a complete color set for each accent including variations for different states and uses
- Ensure accessibility across all color combinations
- Maintain platform-specific color adjustments (iOS/Android)
- Enable smooth transitions between accent colors
- Support persistence of user preferences

## Color System Design

### 1. Extended Color Palette Structure

```typescript
interface AccentColorSet {
  // Primary colors
  base: string; // Main accent color
  hover: string; // Lighter shade for hover states
  active: string; // Darker shade for active/pressed states
  disabled: string; // Muted version for disabled states

  // Text variations
  textOnAccent: string; // Text color when on accent background
  textMuted: string; // Subdued text in accent color

  // UI element variations
  border: string; // Border color for outlined components
  focus: string; // Focus ring color
  shadow: string; // Box shadow color

  // Background variations
  bgSubtle: string; // Subtle background tint
  bgHover: string; // Background for hover states
  bgActive: string; // Background for active states
}

interface AccentColors {
  blue: AccentColorSet;
  red: AccentColorSet;
  orange: AccentColorSet;
  yellow: AccentColorSet;
  green: AccentColorSet;
}
```

### 2. Platform-Specific Considerations

```typescript
const ACCENT_COLORS = Platform.select({
  ios: IOS_ACCENT_COLORS,
  android: ANDROID_ACCENT_COLORS,
  default: IOS_ACCENT_COLORS,
});
```

## Component Coverage

### 1. Core Interactive Elements

- Buttons (filled, outlined, ghost variants)
- Links
- Form inputs (focus states)
- Checkboxes and radio buttons

### 2. Navigation Elements

- Tab indicators
- Selected states in navigation
- Focus rings
- Progress indicators
- Menu selection highlights

### 3. Status and Feedback

- Loading spinners
- Progress bars
- Success/error states
- Badges and indicators

### 4. Custom Components

- Sliders (`Slider.tsx`)
- Toggles (`Toggle.tsx`)
- Pickers
- Date selectors
- Sheet handles and indicators

## Technical Implementation

### 1. Theme Integration

#### Updates to colors.ts

```typescript
// Add accent colors object
const accent = {
  blue: generateAccentSet('#007bfe'),
  red: generateAccentSet('#ff0000'),
  orange: generateAccentSet('#ff8c00'),
  yellow: generateAccentSet('#ffd700'),
  green: generateAccentSet('#28a745'),
};

// Helper function to generate full color set
function generateAccentSet(baseColor: string): AccentColorSet {
  return {
    base: baseColor,
    hover: adjustColor(baseColor, { lightness: +10 }),
    active: adjustColor(baseColor, { lightness: -10 }),
    // ... generate all variants
  };
}
```

### 2. Accent Hook Implementation

```typescript
interface AccentColorHook {
  accentColor: keyof AccentColors;
  accentSet: AccentColorSet;
  setAccentColor: (color: keyof AccentColors) => void;
}

function useAccentColor(): AccentColorHook {
  // Implementation with persistence
}
```

### 3. Storage and Persistence

- Use AsyncStorage for persistent storage
- Include migration handling for future updates
- Sync with system where appropriate

### 4. Style Integration

Updates needed in Styles.tsx:

- Add accent-aware style generators
- Include transition handling
- Support platform-specific adjustments

## Accessibility Considerations

1. **Contrast Requirements**

- Maintain WCAG 2.1 AA standard (4.5:1 for normal text)
- Ensure readable text on accent backgrounds
- Provide high-contrast mode support

2. **Color Combinations**

- Test all accent colors in both light and dark modes
- Validate contrast ratios programmatically
- Include color blind safe variations

## Migration Plan

1. **Phase 1: Foundation**

- Implement basic accent color structure
- Add storage and persistence
- Create useAccentColor hook

2. **Phase 2: Component Updates**

- Update core components
- Add transition support
- Implement platform specific adjustments

3. **Phase 3: Extended Features**

- Add accessibility utilities
- Implement color blind modes
- Add animation support

## Implementation Tasks

1. **Core Updates**

- [x] Create AccentColorSet type definitions
- [x] Implement color generation utilities
- [x] Add persistent storage handling

2. **Component Updates**

- [x] Update Button.tsx (all variants)
- [x] Update nativewindui/Button.tsx
- [x] Add form input handling (Login/Register screens)
- [x] app/\_layout.tsx (Navigation container styling)
- [ ] Update TabBarIcon.tsx (navigation indicators)
- [ ] Implement ProgressIndicator.tsx accents
- [ ] Add Slider.tsx color handling
- [ ] Update Toggle.tsx states
- [ ] Modify DatePicker components
- [ ] Enhance Sheet.tsx handles

3. **Theme Integration**

- [x] colors.ts base implementation
- [ ] Platform-specific DatePicker.android.tsx
- [ ] Styles.tsx transition system
- [x] useAccentColor.tsx hook enhancements

4. **Platform Specifics**

- [ ] Android overrides (DatePicker.android.tsx)
- [ ] iOS color adjustments
- [ ] Platform.select in colors.ts

5. **Testing Requirements**

- [ ] Validate color combinations
- [ ] Test platform behaviors
- [ ] Verify accessibility compliance
- [ ] Check theme persistence

## Development Guidelines

1. **Color Management**

- Use CSS custom properties for web platform
- Implement color mixing utilities
- Support alpha channel management

2. **Performance Considerations**

- [x] Memoize color calculations
- Optimize transition handling
- Minimize runtime calculations

3. **Maintainability**

- Document color generation logic
- Create color testing utilities
- Maintain platform-specific overrides

## Conclusion

This enhanced accent color system provides a robust foundation for user-customizable UI elements while ensuring accessibility, performance, and maintainability. The system is designed to be extensible for future requirements while maintaining compatibility with existing theme functionality.
