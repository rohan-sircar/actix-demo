# Enhanced Sidebar Title Improvement Plan

## Current Issues Analysis

- **Inconsistent Navigation Titles**
  - Duplicate "Main" title appears for both home and controls pages
  - Composite "Auth + Login" title shows in authentication flow
  - Inconsistent title formatting and hierarchy
  - No clear separation between internal navigation names and display titles

## Architecture Improvements

### 1. Navigation Structure

#### Centralized Screen Configuration

```typescript
// types/navigation.ts
export const DRAWER_SCREEN_OPTIONS = {
  Home: {
    name: 'HomeNav',
    title: 'Home',
    icon: 'home',
  },
  Account: {
    name: 'AccountNav',
    title: 'Account',
    icon: 'person',
  },
  Settings: {
    name: 'SettingsNav',
    title: 'Settings',
    icon: 'cog',
  },
} as const;

export type DrawerScreenNames = keyof typeof DRAWER_SCREEN_OPTIONS;
```

#### Type-Safe Navigation Configuration

```typescript
// types/navigation.ts
export interface NavigationConfig {
  name: string;
  title: string;
  icon?: string;
}

export type DrawerScreenConfig = Record<DrawerScreenNames, NavigationConfig>;
```

### 2. Implementation Strategy

#### A. Drawer Navigation Updates

```tsx
// app/_layout.tsx
const DrawerScreens = () => {
  return (
    <>
      <Drawer.Screen
        name={DRAWER_SCREEN_OPTIONS.Home.name}
        component={HomeTabs}
        options={{
          title: DRAWER_SCREEN_OPTIONS.Home.title,
          headerShown: false,
        }}
      />
      <Drawer.Screen
        name={DRAWER_SCREEN_OPTIONS.Account.name}
        component={AuthStack}
        options={{
          title: DRAWER_SCREEN_OPTIONS.Account.title,
        }}
      />
    </>
  );
};
```

#### B. Tab Navigation with Dynamic Titles

```tsx
// components/HomeTabs.tsx
const TAB_ROUTE_TITLES: Record<string, string> = {
  Home: DRAWER_SCREEN_OPTIONS.Home.title,
  Controls: DRAWER_SCREEN_OPTIONS.Settings.title,
  Profile: 'Profile',
};

const HomeTabs = () => {
  const navigation = useNavigation();
  const state = useNavigationState((state) => state);

  useLayoutEffect(() => {
    if (!state?.routes || !state.index) return;
    const currentRoute = state.routes[state.index];
    navigation.setOptions({
      title: TAB_ROUTE_TITLES[currentRoute.name] || TAB_ROUTE_TITLES.Home,
    });
  }, [state?.index, navigation]);

  return <Tab.Navigator>{/* Tab screens */}</Tab.Navigator>;
};
```

#### C. Authentication Stack Refinement

```tsx
// components/AuthStack.tsx
const AuthStack = () => {
  return (
    <Stack.Navigator
      screenOptions={{
        headerTitle: DRAWER_SCREEN_OPTIONS.Account.title,
      }}>
      <Stack.Screen name="SignIn" component={LoginScreen} options={{ title: 'Sign In' }} />
      <Stack.Screen
        name="Register"
        component={RegisterScreen}
        options={{ title: 'Create Account' }}
      />
    </Stack.Navigator>
  );
};
```

## Testing Strategy

### 1. Navigation State Tests

```typescript
describe('Navigation State', () => {
  it('preserves navigation state during screen transitions', () => {
    // Test navigation state preservation
  });

  it('maintains correct screen titles after navigation', () => {
    // Test screen title consistency
  });
});
```

### 2. Deep Linking Tests

```typescript
describe('Deep Linking', () => {
  it('handles deep links with new navigation structure', () => {
    // Test deep linking functionality
  });
});
```

## Acceptance Criteria

### Functional Requirements

- [ ] All drawer entries display human-readable titles
- [ ] No duplicate screen titles in navigation
- [ ] Authentication screens show consistent "Account" section title
- [ ] Tab navigation properly updates parent drawer title
- [ ] All titles follow consistent capitalization rules

### Technical Requirements

- [ ] Type-safe navigation configuration
- [ ] Deep linking functionality preserved
- [ ] Navigation state properly maintained during transitions
- [ ] All titles support future localization
- [ ] No TypeScript compilation errors

## Migration Checklist

1. Create and validate navigation configuration types
2. Update drawer navigation implementation
3. Implement type-safe tab navigation with dynamic titles
4. Refactor authentication stack
5. Add navigation state tests
6. Verify deep linking behavior
7. Test across all supported devices and screen sizes
8. Document navigation structure for team reference

## Implementation Notes

### Phase 1: Types and Configuration

- Create navigation type definitions
- Set up centralized screen configuration
- Update TypeScript interfaces

### Phase 2: Navigation Updates

- Implement drawer navigation changes
- Add dynamic tab titles
- Refactor auth stack

### Phase 3: Testing and Validation

- Add navigation state tests
- Verify deep linking
- Cross-device testing

> **Estimated Timeline**:
>
> - Phase 1: 2-3 hours
> - Phase 2: 3-4 hours
> - Phase 3: 2-3 hours
>   Total: 7-10 hours including testing and validation

## Future Considerations

1. **Localization Support**

   - Prepare title strings for internationalization
   - Consider RTL language support for navigation

2. **Accessibility**

   - Ensure screen readers announce correct titles
   - Maintain navigation hierarchy in accessibility tree

3. **Analytics**
   - Update screen tracking to use new navigation names
   - Ensure analytics events capture correct screen titles

## Resources

- React Navigation Documentation: [Link to docs]
- TypeScript Navigation Guide: [Link to guide]
- Team Navigation Standards: [Link to internal docs]
