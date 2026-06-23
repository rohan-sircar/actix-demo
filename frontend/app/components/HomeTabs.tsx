import FontAwesome from '@expo/vector-icons/FontAwesome';
import { createBottomTabNavigator } from '@react-navigation/bottom-tabs';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import { Icon } from '@roninoss/icons';
import React from 'react';
import { TabButton } from '~/components/TabButton';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import DiscoverScreen from '../screens/DiscoverScreen';
import EditPetScreen from '../screens/EditPetScreen';
import HomeScreen from '../screens/HomeScreen';
import ImageGalleryScreen from '../screens/ImageGalleryScreen';
import PetProfileEditScreen from '../screens/PetProfileEditScreen';
import ProfileScreen from '../screens/ProfileScreen';
import SessionsScreen from '../screens/SessionsScreen';
import { useAuthStore } from '../stores/AuthStore';
import { TabParamList, TabStackParamList, FeedStackParamList } from '~/types/navigation';

const Tab = createBottomTabNavigator<TabParamList>();
const Stack = createNativeStackNavigator<TabStackParamList>();
const FeedStack = createNativeStackNavigator<FeedStackParamList>();

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

const TabNavigator = () => {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  return (
    <Tab.Navigator
      initialRouteName={isAuthenticated ? 'PetProfiles' : 'Discover'}
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
  return (
    <Stack.Navigator
      initialRouteName="Tabs"
      screenOptions={{ headerShown: false }}>
      <Stack.Screen name="Tabs" component={TabNavigator} />
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
