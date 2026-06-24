import React from 'react';
import { View, ViewProps } from 'react-native';

import { SystemColors } from '~/theme/colors';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';

interface SettingsCardProps extends ViewProps {
  children: React.ReactNode;
}

const SettingsCard: React.FC<SettingsCardProps> = ({ children, ...props }) => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  return (
    <View
      {...props}
      style={[
        {
          borderRadius: 16,
          backgroundColor: colors.card,
          borderWidth: 1,
          borderColor: colors.grey5,
          padding: 16,
          marginBottom: 20,
        },
        props.style,
      ]}>
      {children}
    </View>
  );
};

export default SettingsCard;
