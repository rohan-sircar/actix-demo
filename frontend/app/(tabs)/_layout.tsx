import { Tabs } from 'expo-router';
import FontAwesome from '@expo/vector-icons/FontAwesome';
import { Icon } from '@roninoss/icons';
import React from 'react';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { TabButton } from '~/components/TabButton';
import { useAuthStore } from '../stores/AuthStore';
import { Platform, View, Text } from 'react-native';
import ThemeToggle from '~/components/ThemeToggle';
import { LogoutButton } from '~/components/LogoutButton';
import { usePathname } from 'expo-router';

const isWebPlatform = () => Platform.OS === 'web';

const HeaderBranding = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  return (
    <View className="flex-row items-center gap-2">
      <View
        className="items-center justify-center rounded-full"
        style={{ width: 28, height: 28, backgroundColor: isDarkColorScheme ? colors.grey5 : accentSet.bgSubtle }}>
        <FontAwesome name="paw" size={14} color={accentSet.base} />
      </View>
      <Text style={{ color: colors.text, fontSize: 18, fontWeight: 'bold' }}>
        PetMatch
      </Text>
    </View>
  );
};

const HeaderRightContent = () => {
  const { colors } = useColorScheme();
  const { isAuthenticated } = useAuthStore();
  const pathname = usePathname();
  const isProfile = pathname === '/profile';
  return (
    <View className="flex flex-row items-center gap-3 pr-2">
      <ThemeToggle />
      {isProfile ? (
        <View className="opacity-80" style={{ opacity: 0.8 }}>
          <Icon name="cog" color={colors.foreground} />
        </View>
      ) : (
        isAuthenticated && <LogoutButton />
      )}
    </View>
  );
};

const hiddenTabOptions = {
  tabBarItemStyle: { display: 'none', width: 0, minWidth: 0 } as const,
  tabBarLabel: undefined,
};

export default function TabsLayout() {
  const insets = useSafeAreaInsets();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);

  return (
    <Tabs
      screenOptions={{
        headerShown: true,
        headerStyle: { backgroundColor: colors.background },
        headerTintColor: colors.text,
        headerTitle: HeaderBranding,
        headerRight: HeaderRightContent,
        tabBarActiveTintColor: accentSet.base,
        tabBarInactiveTintColor: colors.grey as string,
        tabBarStyle: {
          backgroundColor: isWebPlatform() ? 'transparent' : (isDarkColorScheme ? colors.background : colors.card),
          borderTopWidth: 0,
          paddingBottom: insets.bottom + 16,
          paddingTop: isWebPlatform() ? 8 : 0,
          height: 72 + insets.bottom,
        },
        tabBarButton: (props) => (
          <TabButton
            active={props.accessibilityState?.selected}
            onPress={props.onPress}
            style={[props.style]}>
            {props.children}
          </TabButton>
        ),
      }}>
      <Tabs.Screen
        name="pet-profiles"
        options={{
          tabBarIcon: ({ color }) => <Icon name={'home'} color={color as string} />,
          title: 'Pets',
          ...(!isAuthenticated ? hiddenTabOptions : {}),
        }}
      />
      <Tabs.Screen
        name="discover"
        options={{
          tabBarIcon: ({ color }) => <FontAwesome name="compass" size={22} color={color} />,
          title: 'Discover',
        }}
      />
      <Tabs.Screen
        name="profile"
        options={{
          title: 'Profile',
          tabBarIcon: ({ color }) => <Icon name="person" color={color as string} />,
          ...(!isAuthenticated ? hiddenTabOptions : {}),
        }}
      />
      <Tabs.Screen
        name="settings"
        options={{
          title: 'Settings',
          tabBarIcon: ({ color }) => <FontAwesome name="cog" size={22} color={color} />,
          ...(!isAuthenticated ? hiddenTabOptions : {}),
        }}
      />
      <Tabs.Screen
        name="login"
        options={{
          tabBarIcon: ({ color }) => <FontAwesome name="sign-in" size={22} color={color} />,
          title: 'Login',
          ...(isAuthenticated ? hiddenTabOptions : {}),
        }}
      />
      <Tabs.Screen
        name="register"
        options={{
          tabBarIcon: ({ color }) => <FontAwesome name="user-plus" size={22} color={color} />,
          title: 'Register',
          ...(isAuthenticated ? hiddenTabOptions : {}),
        }}
      />
    </Tabs>
  );
}
