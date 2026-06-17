import React from 'react';
import { StyleSheet, Text, View } from 'react-native';

import { useColorScheme } from '~/lib/useColorScheme';

interface StatTileProps {
  value: string;
  title: string;
}

const StatTile: React.FC<StatTileProps> = ({ value, title }) => {
  const { isDarkColorScheme } = useColorScheme();
  return (
    <View style={styles.container}>
      <Text style={[styles.value, { color: isDarkColorScheme ? '#E4E4E7' : '#333' }]}>{value}</Text>
      <Text style={[styles.title, { color: isDarkColorScheme ? '#E4E4E7' : '#333' }]}>{title}</Text>
    </View>
  );
};

const styles = StyleSheet.create({
  container: {
    minWidth: 100,
    paddingHorizontal: 16,
  },
  value: {
    fontSize: 20,
    fontWeight: '600',
    textAlign: 'center',
    marginBottom: 6,
  },
  title: {
    color: '#888',
    textAlign: 'center',
  },
});

export default StatTile;
