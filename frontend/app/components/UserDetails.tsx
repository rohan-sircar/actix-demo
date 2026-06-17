import { useQuery } from '@tanstack/react-query';
import React from 'react';
import { ActivityIndicator, View } from 'react-native';
import { StyleSheet } from 'react-native';
import { Text } from 'react-native';
import { cardStyle, subCardStyle } from '../styles/Styles';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useAuthStore, UserResponse } from '../stores/AuthStore';
import api from '~/app/lib/api';

const UserDetails = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const user = useAuthStore((state) => state.user);

  const { data, isLoading, isError, error } = useQuery<UserResponse, Error>({
    queryKey: ['user-details'],
    queryFn: async () => {
      const res = await api.get<UserResponse>('/api/v1/user');
      return res.data;
    },
    enabled: !!user,
  });

  return (
    <View style={[cardStyle(isDarkColorScheme, colors, accentSet)]}>
      <Text
        className="tracking-moderate mb-4 text-2xl font-semibold"
        style={{ color: colors.text }}>
        User Details
      </Text>
      {isLoading && <ActivityIndicator size="large" color={colors.primary} />}
      {isError && (
        <Text className="mt-2.5 rounded-lg bg-red-50 p-3 text-center text-base text-red-600">
          Error fetching user data: {error?.message}
        </Text>
      )}
      {data && (
        <View style={subCardStyle(isDarkColorScheme, accentSet)}>
          <Text className="my-1 text-lg tracking-tight" style={{ color: colors.text }}>
            Name: {data.username}
          </Text>
          <Text className="my-1 text-lg tracking-tight" style={{ color: colors.text }}>
            Email: {data.email}
          </Text>
        </View>
      )}
    </View>
  );
};

export default UserDetails;
