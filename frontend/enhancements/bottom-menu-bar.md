# Bottom Menu Bar Implementation Plan

## Implementation Steps

1. Install required dependencies:

```bash
npx expo install @react-navigation/bottom-tabs
```

2. Create tab navigation component in `app/_layout.tsx`:
3. Migrate to react native router from expo router
   - Making the layout with RNR is more explicit whereas expo router is dynamic.
     This explicit nature makes it easier to work with for me.
   - Since routing is no longer dynamic, we can move each screen page to `screens`
     dir
4. Move screen from `app` to `app/screens`
5. Explicit router config looks like this:

```tsx
function HomeTabs() {
  return (
    <Tab.Navigator>
      <Tab.Screen name="Feed" component={FeedScreen} />
      <Tab.Screen name="Messages" component={MessagesScreen} />
    </Tab.Navigator>
  );
}

function RootStack() {
  return (
    <Stack.Navigator>
      <Stack.Screen name="Home" component={HomeTabs} options={{ headerShown: false }} />
      <Stack.Screen name="Profile" component={ProfileScreen} />
    </Stack.Navigator>
  );
}
```

3. Header should be preserved, no special actions are needed for this.

## Required Dependencies

- `@react-navigation/bottom-tabs`: Tab navigation implementation

## Next Steps

1. Install @react-navigation/bottom-tabs
2. Make code changes in app/\_layout.tsx
3. Test navigation flow with both top and bottom bars

## Validation

- [x] Test tab transitions
- [x] Verify icon visibility in both themes
- [x] Ensure top navigation bar works as before
- [x] Check bottom spacing on device
