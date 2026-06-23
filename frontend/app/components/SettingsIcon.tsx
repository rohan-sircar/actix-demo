import { useNavigation } from '@react-navigation/native';
import type { BottomTabNavigationProp } from '@react-navigation/bottom-tabs';
import { Icon } from '@roninoss/icons';
import React from 'react';
import { Pressable, View } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore } from '../stores/AuthStore';
import { TabParamList } from '~/types/navigation';

export const SettingsIcon = () => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const { colors } = useColorScheme();
  const navigation = useNavigation<BottomTabNavigationProp<TabParamList>>();

  return (
    <Pressable
      className="opacity-80"
      onPress={() => {
        navigation.navigate('Settings', { screen: 'SettingsScreen' });
      }}>
      <View className="opacity-90">
        <Icon name="cog" color={colors.foreground} />
      </View>
    </Pressable>
  );
};

export default SettingsIcon;
