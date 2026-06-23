import { DrawerContentComponentProps } from '@react-navigation/drawer';
import { Platform, View, Text, ScrollView } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useNavigation } from '@react-navigation/native';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useAuthStore } from '../stores/AuthStore';
import { Icon } from '@roninoss/icons';
import ThemeToggle from '~/components/ThemeToggle';
import React from 'react';
import type { DrawerNavigationProp } from '@react-navigation/drawer';
import { FontAwesome } from '@expo/vector-icons';
import MenuButton from './MenuButton';
import {
  NAVIGATION_CONFIG,
  DrawerParamList,
  navigateWithTitle,
  AUTH_NAVIGATION_CONFIG,
} from '~/types/navigation';

interface NavigationActions {
  home: () => void;
  signIn: () => void;
  register: () => void;
  profile: () => void;
  sessions: () => void;
  controls: () => void;
}

export const DrawerContent = (_props: DrawerContentComponentProps) => {
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const { isAuthenticated, user } = useAuthStore();
  const navigation = useNavigation<DrawerNavigationProp<DrawerParamList>>();
  const insets = useSafeAreaInsets();

  const navigateTo: NavigationActions = {
    home: () =>
      navigateWithTitle(
        () => navigation.navigate(NAVIGATION_CONFIG.Home.name, { screen: 'PetProfiles' }),
        NAVIGATION_CONFIG.Home.title
      ),
    signIn: () =>
      navigateWithTitle(
        () => navigation.navigate(NAVIGATION_CONFIG.Account.name, { screen: 'SignIn' }),
        AUTH_NAVIGATION_CONFIG.SignIn.title
      ),
    register: () =>
      navigateWithTitle(
        () => navigation.navigate(NAVIGATION_CONFIG.Account.name, { screen: 'Register' }),
        AUTH_NAVIGATION_CONFIG.Register.title
      ),
    profile: () =>
      navigateWithTitle(() => navigation.navigate('Home', { screen: 'Profile' }), 'Profile'),
    sessions: () =>
      navigateWithTitle(() => navigation.navigate('Home', { screen: 'Sessions' }), 'Sessions'),
    controls: () =>
      navigateWithTitle(
        () => navigation.navigate(NAVIGATION_CONFIG.Settings.name),
        NAVIGATION_CONFIG.Settings.title
      ),
  };

  return (
    <View
      style={{
        flex: 1,
        backgroundColor: colors.background,
        paddingTop: insets.top,
      }}>
      <ScrollView contentContainerStyle={{ paddingBottom: 40 }}>
        <View
          style={{
            padding: 20,
          }}>
          <View
            style={{
              flexDirection: 'row',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: 8,
            }}>
            <View className="flex-row items-center gap-2">
              <View
                className="items-center justify-center rounded-full"
                style={{ width: 36, height: 36, backgroundColor: accentSet.bgSubtle }}>
                <FontAwesome name="paw" size={16} color={accentSet.base} />
              </View>
              <Text style={{ color: colors.foreground, fontSize: 22, fontWeight: 'bold' }}>
                PetMatch
              </Text>
            </View>
            {Platform.OS !== 'web' && <ThemeToggle />}
          </View>

          {isAuthenticated && user ? (
            <Text style={{ color: colors.grey, fontSize: 13, marginTop: 4 }}>
              Signed in as @{user.username}
            </Text>
          ) : null}
        </View>

        <View style={{ paddingHorizontal: 12, gap: 4 }}>
          <MenuButton
            onPress={navigateTo.home}
            icon={
              <Icon
                name={NAVIGATION_CONFIG.Home.icon || 'home'}
                color={colors.foreground}
                size={14}
              />
            }
            label={NAVIGATION_CONFIG.Home.title}
          />

          {isAuthenticated && user && (
            <View style={{ gap: 4 }}>
              <MenuButton
                onPress={navigateTo.profile}
                icon={<Icon name="person" color={colors.foreground} size={14} />}
                label="Profile"
              />
              <MenuButton
                onPress={navigateTo.sessions}
                icon={<FontAwesome name="desktop" size={14} color={colors.foreground} />}
                label="Sessions"
              />
            </View>
          )}

          {!isAuthenticated && (
            <View style={{ gap: 4 }}>
              <MenuButton
                onPress={navigateTo.register}
                icon={<FontAwesome name="chevron-up" size={14} color={colors.foreground} />}
                label="Register"
              />
              <MenuButton
                onPress={navigateTo.signIn}
                icon={<FontAwesome name="sign-in" size={14} color={colors.foreground} />}
                label="Sign In"
              />
            </View>
          )}

          <View style={{ marginVertical: 8, borderTopWidth: 1, borderTopColor: colors.grey5 }} />

          <MenuButton
            onPress={navigateTo.controls}
            icon={<FontAwesome name="gear" size={14} color={colors.foreground} />}
            label={NAVIGATION_CONFIG.Settings.title}
          />
        </View>
      </ScrollView>
    </View>
  );
};

export default DrawerContent;
