import React, { useEffect, useState, useMemo } from 'react';
import { View, Text, ScrollView, TouchableOpacity, ActivityIndicator } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useAuthStore } from '../../stores/AuthStore';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import MOCK_PETS, { type MockPet } from '~/data/pets';
import PetCard from '~/app/components/PetCard';

const SPECIES_FILTERS = ['All', 'Dog', 'Cat'];

const DiscoverScreen = () => {
  const router = useRouter();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  useEffect(() => {
    if (!isAuthenticated) {
      router.replace('/login');
    }
  }, [isAuthenticated, router]);
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [selectedSpecies, setSelectedSpecies] = useState('All');
  const [loading, setLoading] = useState(true);

  React.useEffect(() => {
    const timer = setTimeout(() => setLoading(false), 600);
    return () => clearTimeout(timer);
  }, []);

  const filteredPets = useMemo(() => {
    if (selectedSpecies === 'All') return MOCK_PETS;
    return MOCK_PETS.filter((pet) => pet.species === selectedSpecies);
  }, [selectedSpecies]);

  if (loading) {
    return (
      <View className="flex-1 items-center justify-center">
        <ActivityIndicator size="large" color={accentSet.base} />
        <Text className="mt-4 text-sm font-medium" style={{ color: colors.grey }}>
          Discovering pets near you...
        </Text>
      </View>
    );
  }

  return (
    <View className="flex-1">
      <View className="px-4 pb-3 pt-4">
        <Text className="text-2xl font-bold" style={{ color: colors.text }}>
          Discover Pets 🎯
        </Text>
        <Text className="mt-0.5 text-sm" style={{ color: colors.grey }}>
          Find furry friends nearby
        </Text>

        <ScrollView
          horizontal
          showsHorizontalScrollIndicator={false}
          className="mt-3 gap-2"
          contentContainerStyle={{ paddingHorizontal: 0 }}>
          {SPECIES_FILTERS.map((filter) => {
            const isActive = selectedSpecies === filter;
            return (
              <TouchableOpacity
                key={filter}
                onPress={() => setSelectedSpecies(filter)}
                className="rounded-full px-4 py-2"
                style={{
                  backgroundColor: isActive ? accentSet.base : accentSet.bgSubtle,
                }}>
                <Text
                  className="text-sm font-semibold"
                  style={{ color: isActive ? 'white' : accentSet.base }}>
                  {filter === 'All' ? '🐾 All' : filter === 'Dog' ? '🐕 Dogs' : '🐈 Cats'}
                </Text>
              </TouchableOpacity>
            );
          })}
        </ScrollView>
      </View>

      <ScrollView
        className="flex-1"
        contentContainerStyle={{
          paddingHorizontal: 16,
          paddingBottom: 30,
          flexDirection: 'row',
          flexWrap: 'wrap',
          gap: 12,
          justifyContent: 'space-between',
        }}>
        {filteredPets.length > 0 ? (
          filteredPets.map((pet) => (
            <PetCard
              key={pet.id}
              pet_uuid={pet.id}
              name={pet.name}
              species={pet.species}
              breed={pet.breed}
              date_of_birth={
                new Date(Date.now() - Math.random() * 10 * 365 * 24 * 60 * 60 * 1000)
                  .toISOString()
                  .split('T')[0]
              }
              gender={pet.gender}
              weight={pet.weight ? Number(pet.weight) : undefined}
              description={pet.description}
              traits={pet.traits}
              onPress={() => {}}
            />
          ))
        ) : (
          <View className="items-center justify-center py-16">
            <Ionicons name="search" size={40} color={colors.grey} />
            <Text className="mt-3 text-base font-semibold" style={{ color: colors.text }}>
              No pets found
            </Text>
            <Text className="mt-1 px-8 text-center text-sm" style={{ color: colors.grey }}>
              Try adjusting your filters
            </Text>
          </View>
        )}
      </ScrollView>
    </View>
  );
};

export default DiscoverScreen;
