import { FlashList } from '@shopify/flash-list';
import { useFocusEffect } from '@react-navigation/native';
import React, { useCallback } from 'react';
import { View, Text, ActivityIndicator, TouchableOpacity } from 'react-native';
import { useQuery } from '@tanstack/react-query';
import api from '~/app/lib/api';
import PetCard from '~/app/components/PetCard';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { Pet } from '~/app/models/pets';
import { Ionicons } from '@expo/vector-icons';

const Home = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const {
    data: pets,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['pets'],
    queryFn: async () => {
      const res = await api.get<Pet[]>('/api/v1/private/user/pets');
      return res.data;
    },
  });

  useFocusEffect(
    useCallback(() => {
      refetch();
    }, [refetch])
  );

  const secondaryColor = colors.grey;

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center">
        <ActivityIndicator size="large" color={accentSet.base} />
        <Text className="mt-4 text-sm font-medium" style={{ color: secondaryColor }}>
          Fetching your furry friends...
        </Text>
      </View>
    );
  }

  if (error) {
    return (
      <View className="flex-1 items-center justify-center">
        <Text className="text-center text-sm font-medium text-rose-500">
          Failed to load pets. Please try again.
        </Text>
      </View>
    );
  }

  return (
    <View className="flex-1">
      <View className="px-4 pb-2 pt-4">
        <View className="flex-row items-center justify-between">
          <View>
            <Text className="text-2xl font-bold" style={{ color: colors.text }}>
              My Pets 🐾
            </Text>
            <Text className="mt-0.5 text-sm" style={{ color: secondaryColor }}>
              Your furry family members
            </Text>
          </View>
          <TouchableOpacity
            className="items-center justify-center rounded-full"
            style={{ width: 40, height: 40, backgroundColor: accentSet.bgSubtle }}>
            <Ionicons name="add" size={20} color={accentSet.base} />
          </TouchableOpacity>
        </View>
      </View>

      {pets && pets.length > 0 ? (
        <FlashList
          data={pets}
          renderItem={({ item }) => (
            <PetCard
              id={item.id}
              name={item.name}
              species={item.species}
              breed={item.breed}
              date_of_birth={item.date_of_birth}
              gender={item.gender}
              weight={item.weight}
              description={item.description}
              traits={item.traits}
            />
          )}
          estimatedItemSize={150}
          contentContainerStyle={{ paddingHorizontal: 16, paddingTop: 8, paddingBottom: 30 }}
        />
      ) : (
        <View className="flex-1 items-center justify-center px-8">
          <View
            className="mb-4 items-center justify-center rounded-full"
            style={{ width: 64, height: 64, backgroundColor: accentSet.bgSubtle }}>
            <Ionicons name="paw" size={32} color={accentSet.base} />
          </View>
          <Text className="mb-1 text-center text-base font-semibold" style={{ color: colors.text }}>
            No pets yet!
          </Text>
          <Text className="text-center text-sm" style={{ color: secondaryColor }}>
            Add your first pet to get started
          </Text>
        </View>
      )}
    </View>
  );
};

export default Home;
