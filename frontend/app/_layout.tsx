import { ActionSheetProvider } from '@expo/react-native-action-sheet';
import { BottomSheetModalProvider } from '@gorhom/bottom-sheet';
import {
  createNativeStackNavigator,
  NativeStackNavigationOptions,
} from '@react-navigation/native-stack';
import { ThemeProvider as NavThemeProvider } from '@react-navigation/native';
import { Slot, usePathname, useSegments } from 'expo-router';
import 'expo-dev-client';
import { StatusBar } from 'expo-status-bar';
import React, { useEffect } from 'react';
import { Platform, Text, View } from 'react-native';
import { GestureHandlerRootView } from 'react-native-gesture-handler';
import { SafeAreaProvider } from 'react-native-safe-area-context';
import '../global.css';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import ThemeToggle from '~/components/ThemeToggle';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import { useResponsiveLayout } from '~/lib/useResponsiveLayout';
import { NAV_THEME } from '~/theme';

import FontAwesome from '@expo/vector-icons/FontAwesome';
import { LogoutButton } from '~/components/LogoutButton';
import { SystemColors } from '~/theme/colors';
import { useNavigationState } from '@react-navigation/native';
import { useAuthStore } from './stores/AuthStore';
import AuthStack from './components/AuthStack';
import HomeTabs from './components/HomeTabs';
import { SettingsIcon } from './components/SettingsIcon';

const HeaderBranding = () => {
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  return (
    <View className="flex-row items-center gap-2">
      <View
        className="items-center justify-center rounded-full"
        style={{ width: 28, height: 28, backgroundColor: accentSet.bgSubtle }}>
        <FontAwesome name="paw" size={14} color={accentSet.base} />
      </View>
      <Text style={{ color: colors.text, fontSize: 18, fontWeight: 'bold' }}>
        PetMatch
      </Text>
    </View>
  );
};

const HeaderRightContent = () => {
  const { isAuthenticated } = useAuthStore();
  const isProfile = useNavigationState((state) => {
    const homeRoute = state?.routes?.[0];
    const stackRoute = homeRoute?.state?.routes?.[homeRoute.state?.index || 0];
    const tabRoute = stackRoute?.state?.routes?.[stackRoute.state?.index || 0];
    return tabRoute?.name === 'Profile';
  });
  return (
    <View className="flex flex-row items-center gap-3 pr-2">
      <ThemeToggle />
      {isProfile ? (
        <SettingsIcon />
      ) : (
        isAuthenticated && <LogoutButton />
      )}
    </View>
  );
};

const Stack = createNativeStackNavigator();

const STANDALONE_ROUTES = ['verify', 'resend-verification'];

export default function RootLayout() {
  const { colorScheme, isDarkColorScheme, colors } = useColorScheme();
  const queryClient = new QueryClient();
  const { isAuthenticated, isLoading, hydrate } = useAuthStore();
  const { accentColor } = useAccentColor();
  const { isDesktop } = useResponsiveLayout();
  const pathname = usePathname();
  const isStandaloneRoute = STANDALONE_ROUTES.some(
    (route) => pathname === `/${route}` || pathname.startsWith(`/${route}/`)
  );

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
        {!isStandaloneRoute ? (
          <GestureHandlerRootView style={{ flex: 1 }}>
            <BottomSheetModalProvider>
              <ActionSheetProvider>
                <NavThemeProvider value={NAV_THEME[colorScheme]}>
                  <QueryClientProvider client={queryClient}>
                    <View className="flex-1" style={{ backgroundColor: colors.background }}>
                      <View className="mx-auto w-full max-w-[800px] flex-1">
                          <Stack.Navigator>
                            <Stack.Screen
                              name="HomeTabs"
                              component={HomeTabs}
                              options={{
                                headerShown: true,
                                headerTitle: HeaderBranding,
                                headerRight: HeaderRightContent,
                                contentStyle: { paddingBottom: 0 },
                              }}
                            />
                            <Stack.Screen
                              name="AuthStack"
                              component={AuthStack}
                              options={{
                                headerShown: false,
                              }}
                            />
                        </Stack.Navigator>
                      </View>
                    </View>
                  </QueryClientProvider>
                </NavThemeProvider>
              </ActionSheetProvider>
            </BottomSheetModalProvider>
          </GestureHandlerRootView>
        ) : (
          <GestureHandlerRootView style={{ flex: 1 }}>
            <BottomSheetModalProvider>
              <ActionSheetProvider>
                <NavThemeProvider value={NAV_THEME[colorScheme]}>
                  <QueryClientProvider client={queryClient}>
                    <View className="flex-1" style={{ backgroundColor: colors.background }}>
                      <View className="mx-auto w-full max-w-[800px] flex-1">
                        <Slot />
                      </View>
                    </View>
                  </QueryClientProvider>
                </NavThemeProvider>
              </ActionSheetProvider>
            </BottomSheetModalProvider>
          </GestureHandlerRootView>
        )}
      </>
    </SafeAreaProvider>
  );
}
