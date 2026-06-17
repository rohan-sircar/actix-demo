import React from 'react';
import { GestureResponderEvent, Platform, Pressable, View } from 'react-native';
import { useWebRipple } from '~/lib/useWebRipple';
import { cn } from '../lib/cn';
import { useColorScheme } from '~/lib/useColorScheme';
import { useHover } from '~/lib/useHover';

export type TabButtonProps = {
  children: React.ReactNode;
  active?: boolean;
  onPress?: (e: GestureResponderEvent) => void;
  style?: any;
};

export function TabButton({ children, active, onPress, style }: TabButtonProps) {
  const { isDarkColorScheme, colors } = useColorScheme();
  const { hoverStyle, hoverProps } = useHover(colors.grey5, undefined);
  const webRippleProps = useWebRipple({
    // color: isDarkColorScheme ? 'rgba(255, 255, 255, 0.2)' : 'rgba(0, 0, 0, 0.1)',
    // duration: 500,
  });

  return (
    <Pressable
      onPress={onPress}
      style={[style, hoverStyle]}
      className="web:web-ripple flex-1 flex-row items-center justify-center py-1"
      {...(Platform.OS === 'web' ? { ...webRippleProps, ...hoverProps } : {})}>
      <View className="items-center">{children}</View>
    </Pressable>
  );
}
