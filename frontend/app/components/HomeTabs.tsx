import FontAwesome from '@expo/vector-icons/FontAwesome';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { Icon } from '@roninoss/icons';
import React from 'react';
import { TabButton } from '~/components/TabButton';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import DiscoverScreen from '../screens/DiscoverScreen';
import EditPetScreen from '../screens/EditPetScreen';
import HomeScreen from '../screens/HomeScreen';
import ImageGalleryScreen from '../screens/ImageGalleryScreen';
import LoginScreen from '../screens/LoginScreen';
import PetProfileEditScreen from '../screens/PetProfileEditScreen';
import ProfileScreen from '../screens/ProfileScreen';
import RegisterScreen from '../screens/RegisterScreen';
import SettingsScreen from '../screens/SettingsScreen';
import SessionsScreen from '../screens/SessionsScreen';
import { useAuthStore } from '../stores/AuthStore';
import { TabParamList, FeedStackParamList, SettingsStackParamList } from '~/types/navigation';

const Tab = createBottomTabNavigator<TabParamList>();
const FeedStack = createNativeStackNavigator<FeedStackParamList>();
const SettingsStack = createNativeStackNavigator<SettingsStackParamList>();

const FeedStackNavigator = () => {
  return (
    <FeedStack.Navigator screenOptions={{ headerShown: false }}>
      <FeedStack.Screen name="HomeScreen" component={HomeScreen} />
      <FeedStack.Screen name="PetProfile" component={PetProfileEditScreen} options={{ animation: 'slide_from_right' }} />
      <FeedStack.Screen name="ImageGallery" component={ImageGalleryScreen} options={{ animation: 'slide_from_right' }} />
      <FeedStack.Screen name="EditPet" component={EditPetScreen} options={{ animation: 'slide_from_right' }} />
    </FeedStack.Navigator>
  );
};

const SettingsStackNavigator = () => {
  return (
    <SettingsStack.Navigator screenOptions={{ headerShown: false }}>
      <SettingsStack.Screen name="SettingsScreen" component={SettingsScreen} />
      <SettingsStack.Screen name="SessionsScreen" component={SessionsScreen} options={{ animation: 'slide_from_right' }} />
    </SettingsStack.Navigator>
  );
};

const TabNavigator = () => {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const insets = useSafeAreaInsets();
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  return (
    <Tab.Navigator
      initialRouteName={isAuthenticated ? 'PetProfiles' : 'Login'}
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: accentSet.base,
        tabBarInactiveTintColor: colors.grey,
        tabBarStyle: {
          backgroundColor: isWebPlatform() ? 'transparent' : colors.card,
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
      {isAuthenticated && (
        <Tab.Screen
          name="PetProfiles"
          component={FeedStackNavigator}
          options={{
            tabBarIcon: ({ color }) => <Icon name={'home'} color={color} />,
            title: 'Pets',
          }}
        />
      )}
      <Tab.Screen
        name="Discover"
        component={DiscoverScreen}
        options={{
          tabBarIcon: ({ color }) => <FontAwesome name="compass" size={22} color={color} />,
          title: 'Discover',
        }}
      />
      {!isAuthenticated && (
        <Tab.Screen
          name="Login"
          component={LoginScreen}
          options={{
            tabBarIcon: ({ color }) => <FontAwesome name="sign-in" size={22} color={color} />,
            title: 'Login',
          }}
        />
      )}
      {!isAuthenticated && (
        <Tab.Screen
          name="Register"
          component={RegisterScreen}
          options={{
            tabBarIcon: ({ color }) => <FontAwesome name="user-plus" size={22} color={color} />,
            title: 'Register',
          }}
        />
      )}
      {isAuthenticated && (
        <Tab.Screen
          name="Profile"
          component={ProfileScreen}
          options={{
            title: 'Profile',
            tabBarIcon: ({ color }) => <Icon name="person" color={color} />,
          }}
        />
      )}

      {isAuthenticated && (
        <Tab.Screen
          name="Settings"
          component={SettingsStackNavigator}
          options={{
            title: 'Settings',
            tabBarIcon: ({ color }) => <FontAwesome name="cog" size={22} color={color} />,
          }}
        />
      )}
    </Tab.Navigator>
  );
};

export const HomeTabs = () => {
  return <TabNavigator />;
};

const isWebPlatform = () => {
  try {
    return typeof window !== 'undefined';
  } catch {
    return false;
  }
};

export default HomeTabs;
