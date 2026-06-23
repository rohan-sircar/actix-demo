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

export type TabParamList = {
  PetProfiles: undefined;
  Discover: undefined;
  Profile: undefined;
  Settings: NavigatorScreenParams<SettingsStackParamList>;
  Login: undefined;
  Register: undefined;
};

export type FeedStackParamList = {
  HomeScreen: undefined;
  PetProfile: { pet_uuid: string };
  ImageGallery: { pet_uuid: string };
  EditPet: { pet_uuid: string };
};

export type SettingsStackParamList = {
  SettingsScreen: undefined;
  SessionsScreen: undefined;
};


export type AuthStackParamList = {
  SignIn: undefined;
  Register: undefined;
  ForgotPassword: undefined;
  ResetPassword: { token?: string };
};

export type RootStackParamList = {
  [key in NavigationConfigKey]: undefined;
} & {
  Profile: { userId: string };
};

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
