import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { Icon } from '@roninoss/icons';
import React from 'react';
import { Pressable, View } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore } from '../stores/AuthStore';
import { RootStackParamList } from '~/types/navigation';

export const SettingsIcon = () => {
  const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
  const { colors } = useColorScheme();
  const navigation = useNavigation<NativeStackNavigationProp<RootStackParamList>>();

  return (
    <Pressable
      className="opacity-80"
      onPress={() => {
        navigation.navigate('Settings');
      }}>
      <View className="opacity-90">
        <Icon name="cog" color={colors.foreground} />
      </View>
    </Pressable>
  );
};

export default SettingsIcon;
