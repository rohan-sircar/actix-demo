# Responsive Layout Rules

## Breakpoints

- **Mobile**: <600px (from Home screen's `width > 600` check)
- **Tablet**: 600px - 800px
- **Desktop**: >800px (max-width constraint)

## Implementation Examples

```tsx
// From app/index.tsx
const dynamicStyles = StyleSheet.create({
  container: {
    paddingLeft: width > 600 ? 32 : 16,
    paddingRight: width > 600 ? 32 : 16,
  },
});

// Recommended pattern for new components:
function useResponsive() {
  const { width } = useWindowDimensions();
  return {
    isMobile: width < 600,
    isTablet: width >= 600 && width <= 800,
    isDesktop: width > 800,
  };
}
```
