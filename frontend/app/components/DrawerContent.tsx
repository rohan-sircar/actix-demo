import { DrawerContentComponentProps } from '@react-navigation/drawer';
import { View, Text } from 'react-native';
import { useNavigation, useNavigationState } from '@react-navigation/native';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useAuthStore } from '../stores/AuthStore';
import { Icon } from '@roninoss/icons';
import ThemeToggle from '~/components/ThemeToggle';
import React, { useLayoutEffect } from 'react';
import type { DrawerNavigationProp } from '@react-navigation/drawer';
import { FontAwesome } from '@expo/vector-icons';
import MenuButton from './MenuButton';
import {
  NAVIGATION_CONFIG,
  TabParamList,
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
  const state = useNavigationState((state) => state);

  const navigateTo: NavigationActions = {
    home: () =>
      navigateWithTitle(
        () => navigation.navigate(NAVIGATION_CONFIG.Home.name, { screen: 'Feed' }),
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
        padding: 16,
        borderRightWidth: 1,
        borderRightColor: colors.foreground,
      }}>
      <View
        style={{
          flexDirection: 'row',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 32,
        }}>
        <Text style={{ color: colors.foreground, fontSize: 24, fontWeight: 'bold' }}>Menu</Text>
        <ThemeToggle />
      </View>

      <MenuButton
        onPress={navigateTo.home}
        icon={
          <Icon name={NAVIGATION_CONFIG.Home.icon || 'home'} color={colors.foreground} size={14} />
        }
        label={NAVIGATION_CONFIG.Home.title}
      />

      {isAuthenticated && user && (
        <View>
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
        <View>
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

      <MenuButton
        onPress={navigateTo.controls}
        icon={<FontAwesome name="gear" size={14} color={colors.foreground} />}
        label={NAVIGATION_CONFIG.Settings.title}
      />
    </View>
  );
};

export default DrawerContent;
