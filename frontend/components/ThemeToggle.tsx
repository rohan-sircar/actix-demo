import { Icon } from '@roninoss/icons';
import React from 'react';
import { Pressable, View } from 'react-native';
import Animated, { LayoutAnimationConfig, ZoomInRotate } from 'react-native-reanimated';

import { cn } from '~/lib/cn';
import { useColorScheme } from '~/lib/useColorScheme';
import { COLORS } from '~/theme/colors';

const ThemeToggle: React.FC = () => {
  const { colorScheme, toggleColorScheme } = useColorScheme();
  return (
    <LayoutAnimationConfig skipEntering>
      <Animated.View
        className="items-center justify-center"
        key={`toggle-${colorScheme}`}
        entering={ZoomInRotate}>
        <Pressable
          onPress={toggleColorScheme}
          className="p-2 opacity-80" // Add padding for touch area
          hitSlop={{ top: 15, bottom: 15, left: 15, right: 15 }}
          android_ripple={{ borderless: true }}>
          {colorScheme === 'dark'
            ? ({ pressed }) => (
                <View className={cn('px-0.5', pressed && 'opacity-50')}>
                  <Icon namingScheme="sfSymbol" name="moon.stars" color={COLORS.white} size={24} />
                </View>
              )
            : ({ pressed }) => (
                <View className={cn('px-0.5', pressed && 'opacity-50')}>
                  <Icon namingScheme="sfSymbol" name="sun.min" color={COLORS.black} size={24} />
                </View>
              )}
        </Pressable>
      </Animated.View>
    </LayoutAnimationConfig>
  );
};

export default ThemeToggle;
