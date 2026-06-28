import { Ionicons } from '@expo/vector-icons';
import React from 'react';
import { Image, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';

import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { getImageUrl, likesApi } from '~/app/lib/api';
import { useQuery } from '@tanstack/react-query';
import type { MatchWithPets } from '~/app/models/pets';

function MatchCard({ match, colors, isDarkColorScheme, accentSet }: {
  match: MatchWithPets;
  colors: any;
  isDarkColorScheme: boolean;
  accentSet: any;
}) {
  const router = useRouter();

  return (
    <TouchableOpacity
      onPress={() => router.push(`/likes/user-profile/${match.other_user_uuid}`)}
      style={{
        borderRadius: 16,
        backgroundColor: colors.card,
        borderWidth: 1,
        borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.2,
        shadowRadius: 4,
        elevation: 3,
        overflow: 'hidden',
      }}>
      <View className="px-4 pt-4 pb-2">
        <View className="flex-row items-center gap-2">
          {match.other_owner_avatar_url ? (
            <Image source={{ uri: getImageUrl(match.other_owner_avatar_url, 'thumbnail') }} style={{ width: 28, height: 28, borderRadius: 14 }} resizeMode="cover" />
          ) : (
            <Ionicons name="person-circle" size={28} color={colors.grey} />
          )}
          <Text className="text-base font-bold" style={{ color: colors.text }}>
            {match.other_owner_name || 'Unknown'}
          </Text>
        </View>
      </View>

      <View className="flex-row items-center justify-center gap-3 px-4 py-2">
        <TouchableOpacity
          onPress={(e) => {
            e.stopPropagation();
            router.push(`/pet-view/${match.liked_by_other_pet.pet_uuid}`);
          }}
          style={{ flex: 1, alignItems: 'center' }}>
          {match.liked_by_other_pet.primary_image_uuid ? (
            <Image source={{ uri: getImageUrl(match.liked_by_other_pet.primary_image_uuid, 'medium') }} style={{ width: '100%', height: 120, borderRadius: 12 }} resizeMode="cover" />
          ) : (
            <View style={{ width: '100%', height: 120, backgroundColor: accentSet.bgSubtle || '#f0e0d8', justifyContent: 'center', alignItems: 'center', borderRadius: 12 }}>
              <Ionicons name="paw" size={28} color={accentSet.base} />
            </View>
          )}
          <Text className="text-sm font-bold mt-1" style={{ color: colors.text }}>
            {match.liked_by_other_pet.pet_name}
          </Text>
          <Text className="text-xs" style={{ color: colors.grey }}>
            {match.liked_by_other_pet.species}
          </Text>
        </TouchableOpacity>

        <Ionicons name="heart" size={24} color="#ef4444" />

        <TouchableOpacity
          onPress={(e) => {
            e.stopPropagation();
            router.push(`/pet-view/${match.liked_pet.pet_uuid}`);
          }}
          style={{ flex: 1, alignItems: 'center' }}>
          {match.liked_pet.primary_image_uuid ? (
            <Image source={{ uri: getImageUrl(match.liked_pet.primary_image_uuid, 'medium') }} style={{ width: '100%', height: 120, borderRadius: 12 }} resizeMode="cover" />
          ) : (
            <View style={{ width: '100%', height: 120, backgroundColor: accentSet.bgSubtle || '#f0e0d8', justifyContent: 'center', alignItems: 'center', borderRadius: 12 }}>
              <Ionicons name="paw" size={28} color={accentSet.base} />
            </View>
          )}
          <Text className="text-sm font-bold mt-1" style={{ color: colors.text }}>
            {match.liked_pet.pet_name}
          </Text>
          <Text className="text-xs" style={{ color: colors.grey }}>
            {match.liked_pet.species}
          </Text>
        </TouchableOpacity>
      </View>

      <View className="px-4 pb-3">
        <TouchableOpacity
          onPress={(e) => {
            e.stopPropagation();
            router.push(`/likes/user-profile/${match.other_user_uuid}`);
          }}
          style={{ backgroundColor: accentSet.base, borderRadius: 8, paddingVertical: 8, alignItems: 'center' }}>
          <Text style={{ color: '#fff', fontSize: 13, fontWeight: 600 }}>View Profile</Text>
        </TouchableOpacity>
      </View>
    </TouchableOpacity>
  );
}

export default function MatchesScreen() {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const { data: matches } = useQuery({
    queryKey: ['matches-with-pets'],
    queryFn: () => likesApi.listMatchesWithPets(),
  });

  return (
    <View className="w-full flex-1" style={{ backgroundColor: colors.background }}>
      <View className="px-4 pt-4">
        <Text className="text-2xl font-bold mb-1" style={{ color: colors.text }}>
          Matches
        </Text>
        <Text className="text-sm mb-4" style={{ color: colors.grey }}>
          {matches?.length ?? 0} mutual match{matches?.length !== 1 ? 'es' : ''}
        </Text>
      </View>

      <ScrollView showsVerticalScrollIndicator={false} contentContainerStyle={{ paddingLeft: 16, paddingRight: 16, paddingBottom: 80 }}>
        {matches?.length === 0 ? (
          <View className="items-center justify-center py-20">
            <Ionicons name="heart-dislike-outline" size={48} color={colors.grey} />
            <Text className="text-base font-semibold mt-4" style={{ color: colors.text }}>
              No matches yet
            </Text>
            <Text className="text-sm text-center mt-1 px-8" style={{ color: colors.grey }}>
              Keep swiping to find your pet's best friend!
            </Text>
          </View>
        ) : (
          <View className="gap-4">
            {matches?.map((match) => (
              <MatchCard key={match.liked_pet.pet_uuid} match={match} colors={colors} isDarkColorScheme={isDarkColorScheme} accentSet={accentSet} />
            ))}
          </View>
        )}
      </ScrollView>
    </View>
  );
}
