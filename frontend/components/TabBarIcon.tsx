import React from 'react';
import { View, Text, TouchableOpacity, StyleSheet } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import { Ionicons } from '@expo/vector-icons';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';

interface TabBarIconProps {
  focused: boolean;
  onPress: () => void;
}

const TabBarIcon: React.FC<TabBarIconProps> = ({ focused, onPress }) => {
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  return (
    <TouchableOpacity
      style={[styles.container, focused && { backgroundColor: accentSet.bgActive }]}
      onPress={onPress}>
      <Ionicons
        name="home-outline"
        size={24}
        style={styles.icon}
        color={focused ? accentSet.base : colors.text}
      />
    </TouchableOpacity>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 10,
  },
  icon: {
    fontSize: 24,
  },
});

export default TabBarIcon;
