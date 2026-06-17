# Sidebar Title Improvement Plan

## Current Issues Analysis

- **Duplicate "Main" title** caused by:
  - Drawer.Screen name="Main" (lines 173/209)
  - ControlsScreen nested under HomeTabs (line 123)
- **Composite "Auth + Login" title** from:
  - Parent AuthStack navigation (lines 43-50)
  - Nested LoginScreen route (line 46)
- **Missing title standardization** across navigation hierarchies

## Proposed Changes

### 1. Navigation Structure Renaming

```tsx
// Before (line 173)
<Drawer.Screen name="Main" component={HomeTabs} />

// After
<Drawer.Screen
  name="HomeNav"
  component={HomeTabs}
  options={{ title: "Home Dashboard" }}
/>
```

### 2. Title Hierarchy Strategy

| Navigation Level | Naming Convention | Example    |
| ---------------- | ----------------- | ---------- |
| Drawer           | Action + "Nav"    | AccountNav |
| Stack            | Feature + "Stack" | AuthStack  |
| Tab              | Section + "Tabs"  | MainTabs   |

### 3. Implementation Steps

1. **Drawer Navigation Updates** (lines 163-211)

```diff
- <Drawer.Screen name="Main" component={HomeTabs} />
+ <Drawer.Screen
+   name="HomeNav"
+   component={HomeTabs}
+   options={{ title: "Home", headerShown: false }}
+ />
```

2. **Auth Stack Improvements** (lines 43-50)

```tsx
<Stack.Navigator
  screenOptions={{
    headerTitle: 'Account Management',
  }}>
  <Stack.Screen name="SignIn" component={LoginScreen} options={{ title: 'Sign In' }} />
</Stack.Navigator>
```

3. **Dynamic Tab Titles** (HomeTabs component)

```tsx
useLayoutEffect(() => {
  navigation.setOptions({
    title: routeNames[state?.index || 0] || 'Home',
  });
}, [state?.index]);
```

## Acceptance Criteria

- [ ] All drawer entries show human-readable titles
- [ ] No duplicate "Main" references in sidebar
- [ ] Auth-related screens show "Account" instead of "Auth"
- [ ] Tab navigation updates parent drawer title
- [ ] Titles use consistent title case formatting

## Migration Checklist

1. Update all Drawer.Screen components (2 instances)
2. Modify AuthStack navigation options
3. Implement tab index tracking in HomeTabs
4. Verify title consistency across breakpoints
5. Update navigation type definitions (types/navigation.ts)

> **Estimated Effort**: 2-3 hours development + testing
