import React from 'react';
import { View, Text, StyleSheet, ActivityIndicator } from 'react-native';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';

interface ProgressIndicatorProps {
  size?: number;
  color?: string;
  message?: string;
}

const ProgressIndicator: React.FC<ProgressIndicatorProps> = ({ size = 24, color, message }) => {
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const indicatorColor = color || accentSet.base;

  return (
    <View style={styles.container}>
      <ActivityIndicator size={size} color={indicatorColor} />
      {message && <Text style={[styles.message, { color: colors.text }]}>{message}</Text>}
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  message: {
    marginTop: 16,
    fontSize: 16,
  },
});

export default ProgressIndicator;
