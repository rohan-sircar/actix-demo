# Layout Validation Checklist

## Cross-Device Testing

- [ ] Mobile (<600px): Check padding/margin consistency
- [ ] Tablet (600-800px): Verify responsive breakpoints
- [ ] Desktop (>800px): Validate max-width constraint

## Theme Compatibility

- [ ] Dark mode contrast ratios
- [ ] Light mode color consistency
- [ ] Theme provider inheritance

## Functionality Checks

- [ ] Safe area insets on notched devices
- [ ] Navigation elements positioning
- [ ] Touch target sizes (min 48dp)

## Performance Metrics

- [ ] FPS ≥60 during screen transitions
- [ ] Layout recalculations <5ms
- [ ] Memory usage stable across screens
