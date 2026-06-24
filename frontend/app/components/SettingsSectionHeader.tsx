import React from 'react';
import { Text } from 'react-native';

import { useColorScheme } from '~/lib/useColorScheme';

interface SettingsSectionHeaderProps {
  title: string;
}

const SettingsSectionHeader: React.FC<SettingsSectionHeaderProps> = ({ title }) => {
  const { colors } = useColorScheme();

  return (
    <Text
      style={{
        color: colors.grey,
        fontSize: 12,
        fontWeight: '600',
        textTransform: 'uppercase',
        letterSpacing: 0.8,
        marginBottom: 8,
        marginLeft: 4,
        marginTop: 8,
      }}>
      {title}
    </Text>
  );
};

export default SettingsSectionHeader;
