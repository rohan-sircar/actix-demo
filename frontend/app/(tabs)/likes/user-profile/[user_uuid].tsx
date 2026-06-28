import React from 'react';
import { ScrollView, Text, View, ActivityIndicator, TouchableOpacity } from 'react-native';
import { useRouter, useLocalSearchParams } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useQuery } from '@tanstack/react-query';

import Avatar from '~/app/components/Avatar';
import StatTile from '~/app/components/StatTile';
import { usersApi, getImageUrl } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { PublicPet, UserProfile } from '~/app/models/pets';
import PetCard from '~/app/components/PetCard';

export default function PublicUserProfileScreen() {
  const router = useRouter();
  const { user_uuid } = useLocalSearchParams<{ user_uuid: string }>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const { data: profile, isLoading: profileLoading } = useQuery({
    queryKey: ['public-profile', user_uuid],
    queryFn: () => usersApi.getPublicProfile(user_uuid),
    enabled: !!user_uuid,
  });

  const { data: pets, isLoading: petsLoading } = useQuery({
    queryKey: ['user-pets', user_uuid],
    queryFn: () => usersApi.listPublicPets(user_uuid),
    enabled: !!user_uuid,
  });

  const isLoading = profileLoading || petsLoading;

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <ActivityIndicator size="large" color={accentSet.base} />
      </View>
    );
  }

  if (!profile) {
    return (
      <View className="flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <Ionicons name="person-outline" size={48} color={colors.grey} />
        <Text className="mt-3 text-sm" style={{ color: colors.grey }}>
          User not found
        </Text>
      </View>
    );
  }

  return (
    <View className="flex-1" style={{ backgroundColor: colors.background }}>
      <ScrollView
        showsVerticalScrollIndicator={false}
        contentContainerStyle={{ padding: 16, paddingBottom: 80 }}>
        {/* Header with user info */}
        <View className="mb-4">
          <Text className="mb-4 text-2xl font-bold" style={{ color: colors.text }}>
            Profile
          </Text>

          <View className="flex-row items-center">
            <Avatar userId={null} avatarUrl={null} size={64} style={{ marginRight: 16 }} />
            <View className="flex-1">
              <Text className="text-lg font-bold" style={{ color: colors.text }}>
                {profile.display_name || 'Pet Owner'}
              </Text>
              <Text className="text-sm" style={{ color: colors.grey }}>
                @{profile.display_name?.toLowerCase().replace(/\s+/g, '_') || 'user'}
              </Text>
            </View>
          </View>
        </View>

        {/* About Me + Stats */}
        <View className="flex-row flex-wrap gap-4" style={{ width: '100%' }}>
          {(profile.bio || profile.location || profile.website_url) && (
            <View style={{ width: '48%', borderRadius: 16, backgroundColor: colors.card, borderWidth: 1, borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8', shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.2, shadowRadius: 4, elevation: 3 }}>
              <View className="px-5 py-4">
                {profile.bio && (
                  <>
                    <Text className="mb-2 text-xs font-bold uppercase tracking-wider" style={{ color: accentSet.base }}>
                      About Me
                    </Text>
                    <Text className="text-sm leading-relaxed" style={{ color: colors.grey }}>
                      {profile.bio}
                    </Text>
                  </>
                )}
                {(profile.location || profile.website_url) && (
                  <View className="mt-3 flex-row flex-wrap gap-2">
                    {profile.location && (
                      <View className="flex-row items-center gap-1.5 rounded-full px-3.5 py-2" style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)' }}>
                        <Ionicons name="location" size={15} color={accentSet.base} />
                        <Text className="text-sm font-medium" style={{ color: colors.text }}>
                          {profile.location}
                        </Text>
                      </View>
                    )}
                    {profile.website_url && (
                      <View className="flex-row items-center gap-1.5 rounded-full px-3.5 py-2" style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)' }}>
                        <Ionicons name="globe" size={15} color={accentSet.base} />
                        <Text className="text-sm font-medium" style={{ color: colors.text }}>
                          {profile.website_url}
                        </Text>
                      </View>
                    )}
                  </View>
                )}
              </View>
            </View>
          )}

          <View style={{ width: profile.bio || profile.location || profile.website_url ? '48%' : '100%', borderRadius: 16, backgroundColor: colors.card, borderWidth: 1, borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8', shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.2, shadowRadius: 4, elevation: 3 }}>
            <View className="px-5 py-5">
              <Text className="mb-3 text-xs font-bold uppercase tracking-wider text-center" style={{ color: accentSet.base }}>
                Stats
              </Text>
              <View className="flex-row justify-center">
                <StatTile title="Pets" value={String(pets?.length ?? 0)} />
              </View>
            </View>
          </View>
        </View>

        {/* Pet grid */}
        <View className="mt-4">
          {(!pets || pets.length === 0) ? (
            <View className="items-center py-12 rounded-xl" style={{ backgroundColor: colors.card }}>
              <Ionicons name="paw-outline" size={48} color={colors.grey} />
              <Text className="mt-3 text-sm" style={{ color: colors.grey }}>
                No public pets to show
              </Text>
            </View>
          ) : (
            <View style={{ flexDirection: 'row', flexWrap: 'wrap', justifyContent: 'space-between', gap: 12 }}>
              {pets.map((pet) => (
                <PetCard
                  key={pet.pet_uuid}
                  pet_uuid={pet.pet_uuid}
                  name={pet.name}
                  species={pet.species}
                  breed={pet.breed}
                  date_of_birth={pet.date_of_birth}
                  gender={pet.gender}
                  weight={pet.weight}
                  description={pet.description}
                  traits={pet.traits}
                  primary_image={pet.primary_image}
                  onPress={() => router.push(`/pet-view/${pet.pet_uuid}`)}
                />
              ))}
            </View>
          )}
        </View>
      </ScrollView>
    </View>
  );
}
