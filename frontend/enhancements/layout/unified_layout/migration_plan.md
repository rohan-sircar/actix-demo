# Unified Layout Migration Plan

## Phase 1: Base Layout Implementation

1. Create `app/_layout.tsx` with:
   - SafeAreaView wrapper
   - Responsive container (max-width: 800px)
   - Theme providers from old_layout.tsx
2. Update existing screens to use layout slots:
   ```tsx
   // Home screen example
   export default function Home() {
     return (
       <Layout>
         <FeedContent />
       </Layout>
     );
   }
   ```

## Phase 2: Responsive Rules Integration

1. Implement breakpoint handling:
   - Mobile: <600px
   - Tablet: 600-800px
   - Desktop: >800px
2. Create responsive hooks for shared logic

## Phase 3: Validation & Testing

1. Cross-screen layout consistency checks
2. Theme compatibility verification
3. Performance benchmarks
