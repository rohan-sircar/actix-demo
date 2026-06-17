import { DrawerNavigationProp } from '@react-navigation/drawer';
import { BottomTabNavigationProp } from '@react-navigation/bottom-tabs';
import { NavigatorScreenParams } from '@react-navigation/native';
import { Platform } from 'react-native';

export const NAVIGATION_CONFIG = {
  Home: {
    name: 'Home',
    title: 'Home',
    icon: 'home',
  },
  Account: {
    name: 'Account',
    title: 'Account',
    icon: 'person',
  },
  Settings: {
    name: 'Settings',
    title: 'Settings',
    icon: 'cog',
  },
} as const;

export const AUTH_NAVIGATION_CONFIG = {
  SignIn: {
    name: 'Login',
    title: 'Login',
    icon: 'sign-in',
  },
  Register: {
    name: 'Register',
    title: 'Register',
    icon: 'chevron-up',
  },
} as const;

export type NavigationConfigKey = keyof typeof NAVIGATION_CONFIG;

export type NavigationConfig = {
  name: string;
  title: string;
  icon?: string;
};

// Route param types
export type TabParamList = {
  Feed: undefined;
  Profile: undefined;
  Sessions: undefined;
};

export type AuthStackParamList = {
  SignIn: undefined;
  Register: undefined;
  ForgotPassword: undefined;
  ResetPassword: { token?: string };
};

export type DrawerParamList = {
  [NAVIGATION_CONFIG.Home.name]: NavigatorScreenParams<TabParamList>;
  [NAVIGATION_CONFIG.Account.name]: NavigatorScreenParams<AuthStackParamList>;
  [NAVIGATION_CONFIG.Settings.name]: undefined;
};

export type RootStackParamList = {
  [key in NavigationConfigKey]: undefined;
} & {
  Profile: { userId: string };
};

// Type guard for checking if a navigation route belongs to DrawerParamList
export function isDrawerRoute(route: string): route is keyof DrawerParamList {
  return Object.values(NAVIGATION_CONFIG).some((config) => config.name === route);
}

// Helper to get screen title from navigation config
export function getScreenTitle(route: string): string {
  const config = Object.values(NAVIGATION_CONFIG).find((c) => c.name === route);
  return config?.title || route;
}

// Navigation helper that handles both navigation and title updates
export function navigateWithTitle(navigate: () => void, title?: string) {
  if (Platform.OS == 'web') { document.title = title ? `My Web App | ${title}` : 'My App'; }
  navigate();
}
