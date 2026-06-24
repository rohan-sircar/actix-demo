import { Icon } from '@roninoss/icons';
import React from 'react';
import { Pressable, View } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import { useRouter } from 'expo-router';

export const SettingsIcon = () => {
  const { colors } = useColorScheme();
  const router = useRouter();

  return (
    <Pressable
      className="opacity-80"
      onPress={() => {
        router.push('/settings');
      }}>
      <View className="opacity-90">
        <Icon name="cog" color={colors.foreground} />
      </View>
    </Pressable>
  );
};

export default SettingsIcon;
