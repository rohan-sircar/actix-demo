import React from 'react';
import { Switch, View, Text, StyleSheet } from 'react-native';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';

interface ToggleProps {
  value: boolean;
  onValueChange: (value: boolean) => void;
  label: string;
}

const Toggle: React.FC<ToggleProps> = ({ value, onValueChange, label }) => {
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  return (
    <View style={styles.container}>
      <Text style={[styles.label, { color: colors.text }]}>{label}</Text>
      <Switch
        value={value}
        onValueChange={onValueChange}
        trackColor={{ false: colors.grey4, true: accentSet.base }}
        thumbColor={value ? accentSet.active : colors.grey3}
      />
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: 16,
  },
  label: {
    fontSize: 16,
    marginRight: 16,
  },
});

export default Toggle;
