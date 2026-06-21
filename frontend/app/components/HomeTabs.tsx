import FontAwesome from '@expo/vector-icons/FontAwesome';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { useNavigation } from '@react-navigation/native';
import { Icon } from '@roninoss/icons';
import React from 'react';
import { View } from 'react-native';
import { LogoutButton } from '~/components/LogoutButton';
import { TabButton } from '~/components/TabButton';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import ControlsScreen from '../screens/ControlsScreen';
import DiscoverScreen from '../screens/DiscoverScreen';
import HomeScreen from '../screens/HomeScreen';
import PetProfileScreen from '../screens/PetProfileScreen';
import ProfileScreen from '../screens/ProfileScreen';
import SessionsScreen from '../screens/SessionsScreen';
import { useAuthStore } from '../stores/AuthStore';
import { NAVIGATION_CONFIG, TabParamList, TabStackParamList } from '~/types/navigation';
import { SettingsIcon } from './SettingsIcon';

const Tab = createBottomTabNavigator<TabParamList>();
const Stack = createNativeStackNavigator<TabStackParamList>();

const TabNavigator = () => {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  return (
    <Tab.Navigator
      initialRouteName={isAuthenticated ? 'Feed' : 'Discover'}
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: accentSet.base,
        tabBarInactiveTintColor: colors.grey,
        tabBarStyle: {
          backgroundColor: isWebPlatform() ? 'transparent' : colors.card,
          borderTopWidth: 0,
          paddingBottom: 8,
          paddingTop: 8,
          height: 80,
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
          name="Feed"
          component={HomeScreen}
          options={{
            tabBarIcon: ({ color }) => <Icon name={'home'} color={color} />,
            title: 'Home',
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
          name="Sessions"
          component={SessionsScreen}
          options={{
            title: 'Sessions',
            tabBarIcon: ({ color }) => <Icon name="monitor" color={color} />,
          }}
        />
      )}
    </Tab.Navigator>
  );
};

export const HomeTabs = () => {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);

  return (
    <Stack.Navigator
      initialRouteName="Tabs"
      screenOptions={{ headerShown: false }}>
      <Stack.Screen name="Tabs" component={TabNavigator} />
      {isAuthenticated && (
        <Stack.Screen
          name="PetProfile"
          component={PetProfileScreen}
          options={{ animation: 'slide_from_right' }}
        />
      )}
    </Stack.Navigator>
  );
};

const isWebPlatform = () => {
  try {
    return typeof window !== 'undefined';
  } catch {
    return false;
  }
};

export default HomeTabs;
