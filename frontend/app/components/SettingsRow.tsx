import React from 'react';
import { Pressable, Text, View, ViewStyle } from 'react-native';

import FontAwesome from '@expo/vector-icons/FontAwesome';
import { Icon } from '@roninoss/icons';

import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';

interface SettingsRowProps {
  icon: string;
  faIcon?: string;
  title: string;
  subtitle?: string;
  onPress?: () => void;
  rightContent?: React.ReactNode;
  destructive?: boolean;
  style?: ViewStyle;
}

const SettingsRow: React.FC<SettingsRowProps> = ({
  icon,
  faIcon,
  title,
  subtitle,
  onPress,
  rightContent,
  destructive,
  style,
}) => {
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const textColor = destructive ? '#E11D48' : colors.text;
  const subtitleColor = subtitle ? colors.grey : undefined;

  const row = (
    <View
      style={[
        {
          flexDirection: 'row',
          alignItems: 'center',
          paddingVertical: 12,
          borderBottomWidth: 1,
          borderBottomColor: colors.grey5,
        },
        style,
      ]}>
      <View
        style={{
          width: 36,
          height: 36,
          borderRadius: 10,
          backgroundColor: accentSet.bgSubtle,
          alignItems: 'center',
          justifyContent: 'center',
          marginRight: 14,
        }}>
        {faIcon ? (
          <FontAwesome name={faIcon as any} size={16} color={destructive ? '#E11D48' : accentSet.base} />
        ) : (
          <Icon name={icon as any} size={16} color={destructive ? '#E11D48' : accentSet.base} />
        )}
      </View>
      <View style={{ flex: 1 }}>
        <Text
          style={{
            color: textColor,
            fontSize: 15,
            fontWeight: '500',
          }}>
          {title}
        </Text>
        {subtitle && (
          <Text
            style={{
              color: subtitleColor,
              fontSize: 12,
              marginTop: 2,
            }}>
            {subtitle}
          </Text>
        )}
      </View>
      {rightContent}
    </View>
  );

  if (onPress) {
    return (
      <Pressable
        onPress={onPress}
        style={{ width: '100%' }}>
        {row}
      </Pressable>
    );
  }

  return row;
};

export default SettingsRow;
