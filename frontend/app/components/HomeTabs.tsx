import FontAwesome from '@expo/vector-icons/FontAwesome';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { useNavigation, useNavigationState } from '@react-navigation/native';
import { Icon } from '@roninoss/icons';
import React, { useEffect, useLayoutEffect } from 'react';
import { View } from 'react-native';
import { LogoutButton } from '~/components/LogoutButton';
import { TabButton } from '~/components/TabButton';
import { useColorScheme } from '~/lib/useColorScheme';
import ControlsScreen from '../screens/ControlsScreen';
import HomeScreen from '../screens/HomeScreen';
import ProfileScreen from '../screens/ProfileScreen';
import SessionsScreen from '../screens/SessionsScreen';
import { useAuthStore } from '../stores/AuthStore';
import { NAVIGATION_CONFIG, TabParamList } from '~/types/navigation';
import { SettingsIcon } from './SettingsIcon';

const Tab = createBottomTabNavigator<TabParamList>();

export const HomeTabs = () => {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const { colors } = useColorScheme();
  const navigation = useNavigation();
  const state = useNavigationState((state) => state);

  return (
    <Tab.Navigator
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: colors.text,
        tabBarActiveBackgroundColor: colors.grey4,
        tabBarStyle: {
          backgroundColor: colors.background,
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
      <Tab.Screen
        name="Feed"
        component={HomeScreen}
        options={{
          headerRight: () => (
            <View className="flex flex-row items-center gap-3 pr-2">
              {isAuthenticated && <SettingsIcon />}
              {isAuthenticated && <LogoutButton />}
            </View>
          ),
          tabBarIcon: () => <Icon name={'home'} color={colors.text} />,
          title: 'Feed',
        }}
      />
      {isAuthenticated && (
        <Tab.Screen
          name="Profile"
          component={ProfileScreen}
          options={{
            title: 'Profile',
            tabBarIcon: () => <Icon name="person" color={colors.text} />,
          }}
        />
      )}
      {isAuthenticated && (
        <Tab.Screen
          name="Sessions"
          component={SessionsScreen}
          options={{
            title: 'Sessions',
            tabBarIcon: () => <Icon name="monitor" color={colors.text} />,
          }}
        />
      )}
    </Tab.Navigator>
  );
};

export default HomeTabs;
