import { ActionSheetProvider } from '@expo/react-native-action-sheet';
import { BottomSheetModalProvider } from '@gorhom/bottom-sheet';
import { Slot } from 'expo-router';
import 'expo-dev-client';
import { StatusBar } from 'expo-status-bar';
import React, { useEffect } from 'react';
import { View } from 'react-native';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import '../global.css';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore } from './stores/AuthStore';

export default function RootLayout() {
  const { isDarkColorScheme, colors } = useColorScheme();
  const queryClient = new QueryClient();
  const { isLoading, hydrate } = useAuthStore();

  useEffect(() => {
    hydrate();
  }, [hydrate]);

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center">
        <StatusBar style="light" />
      </View>
    );
  }

  return (
    <SafeAreaProvider>
      <>
        <StatusBar
          key={`root-status-bar-${isDarkColorScheme ? 'light' : 'dark'}`}
          style={isDarkColorScheme ? 'light' : 'dark'}
        />
        <GestureHandlerRootView style={{ flex: 1 }}>
          <BottomSheetModalProvider>
            <ActionSheetProvider>
              <QueryClientProvider client={queryClient}>
                <View className="flex-1" style={{ backgroundColor: colors.background }}>
                  <View className="mx-auto w-full max-w-[800px] flex-1">
                    <Slot />
                  </View>
                </View>
              </QueryClientProvider>
            </ActionSheetProvider>
          </BottomSheetModalProvider>
        </GestureHandlerRootView>
      </>
    </SafeAreaProvider>
  );
}
