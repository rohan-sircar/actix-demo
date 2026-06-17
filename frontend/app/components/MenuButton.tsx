import { Pressable, Text } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import { useHover } from '~/lib/useHover';
import React from 'react';

type MenuButtonProps = {
  onPress: () => void;
  icon: React.ReactNode;
  label: string;
};

export const MenuButton = ({ onPress, icon, label }: MenuButtonProps) => {
  const { colors } = useColorScheme();
  const { hoverProps, hoverStyle } = useHover(colors.grey5);

  return (
    <Pressable
      onPress={onPress}
      {...hoverProps}
      style={({ pressed }) => [
        {
          paddingVertical: 12,
          paddingHorizontal: 16,
          flexDirection: 'row',
          alignItems: 'center',
          gap: 12,
          borderRadius: 8,
          backgroundColor: pressed ? colors.grey4 : 'transparent',
        },
        hoverStyle,
      ]}>
      {icon}
      <Text style={{ color: colors.foreground }}>{label}</Text>
    </Pressable>
  );
};

export default MenuButton;
