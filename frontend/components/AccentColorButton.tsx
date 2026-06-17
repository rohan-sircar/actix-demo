import React from 'react';
import { Pressable, View } from 'react-native';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useWebRipple } from '~/lib/useWebRipple';
import { BaseAccentColors } from '~/theme/colors';
import { AccentColorType } from '~/theme/colors';

interface AccentColorButtonProps {
  color: AccentColorType;
}

const AccentColorButton: React.FC<AccentColorButtonProps> = ({ color }) => {
  const { setAccentColor } = useAccentColor();
  const accentSet = getAccentSet(color);
  const webRippleProps = useWebRipple();

  return (
    <Pressable
      onPress={() => setAccentColor(color)}
      className="p-2 opacity-80"
      hitSlop={{ top: 15, bottom: 15, left: 15, right: 15 }}
      android_ripple={{ borderless: true }}>
      <View
        className="rounded-full"
        style={{ width: 30, height: 30, backgroundColor: accentSet.base }}></View>
    </Pressable>
  );
};

export default AccentColorButton;
