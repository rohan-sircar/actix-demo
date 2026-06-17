import { FlashList } from '@shopify/flash-list';
import { useFocusEffect } from '@react-navigation/native';
import React, { useCallback } from 'react';
import { View, Text, ActivityIndicator } from 'react-native';
import { useQuery } from '@tanstack/react-query';
import api from '~/app/lib/api';
import PetCard from '~/app/components/PetCard';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { Pet } from '~/app/models/pets';

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
      const res = await api.get<Pet[]>('/api/v1/user/pets');
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
        <Text className="mt-4 text-sm" style={{ color: secondaryColor }}>
          Loading your pets...
        </Text>
      </View>
    );
  }

  if (error) {
    return (
      <View className="flex-1 items-center justify-center">
        <Text className="text-center text-sm text-rose-500">
          Failed to load pets. Please try again.
        </Text>
      </View>
    );
  }

  return (
    <View className="flex-1 px-4 py-4">
      <Text className="mb-4 text-xl font-bold" style={{ color: colors.text }}>
        My Pets
      </Text>
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
          contentContainerStyle={{ paddingBottom: 20 }}
        />
      ) : (
        <View className="flex-1 items-center justify-center">
          <Text className="text-center text-sm" style={{ color: secondaryColor }}>
            No pets yet. Add your first pet from Settings!
          </Text>
        </View>
      )}
    </View>
  );
};

export default Home;
