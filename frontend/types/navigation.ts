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
  Menu: {
    name: 'Menu',
    title: 'Menu',
    icon: 'bars',
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

export type TabParamList = {
  Feed: undefined;
  Discover: undefined;
  Profile: undefined;
  Sessions: undefined;
  PetProfile: { pet_uuid: string };
  ImageGallery: { pet_uuid: string };
};

export type TabStackParamList = {
  Tabs: undefined;
  PetProfile: { pet_uuid: string };
  ImageGallery: { pet_uuid: string };
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
  [NAVIGATION_CONFIG.Menu.name]: undefined;
};

export type StackParamList = {
  [NAVIGATION_CONFIG.Home.name]: NavigatorScreenParams<TabParamList>;
  [NAVIGATION_CONFIG.Account.name]: NavigatorScreenParams<AuthStackParamList>;
  [NAVIGATION_CONFIG.Settings.name]: undefined;
  [NAVIGATION_CONFIG.Menu.name]: undefined;
};

export type RootStackParamList = {
  [key in NavigationConfigKey]: undefined;
} & {
  Profile: { userId: string };
};

export function isDrawerRoute(route: string): route is keyof DrawerParamList {
  return Object.values(NAVIGATION_CONFIG).some((config) => config.name === route);
}

export function getScreenTitle(route: string): string {
  const config = Object.values(NAVIGATION_CONFIG).find((c) => c.name === route);
  return config?.title || route;
}

export function navigateWithTitle(navigate: () => void, title?: string) {
  if (Platform.OS == 'web') {
    document.title = title ? `PetMatch | ${title}` : 'PetMatch';
  }
  navigate();
}
