import { Ionicons } from '@expo/vector-icons';
import React from 'react';
import { Platform, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';

import { useColorScheme } from '~/lib/useColorScheme';
import { likesApi } from '~/app/lib/api';
import { AuthenticatedImage } from '~/app/components/AuthenticatedImage';
import { useQuery } from '@tanstack/react-query';
import type { MatchWithPets } from '~/app/models/pets';
import MatchPetCard from '~/app/components/MatchPetCard';

function MatchRow({ match, colors, isDarkColorScheme }: {
  match: MatchWithPets;
  colors: any;
  isDarkColorScheme: boolean;
}) {
  const router = useRouter();

  return (
    <TouchableOpacity
      onPress={() => router.push(`/likes/user-profile/${match.other_user_uuid}`)}
      activeOpacity={0.8}
      style={{
        borderRadius: 16,
        backgroundColor: colors.card,
        borderWidth: 1,
        borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.08,
        shadowRadius: 4,
        elevation: 2,
        overflow: 'hidden',
      }}>
      <View className="px-4 pt-4 pb-3">
        <View className="flex-row items-center gap-2">
          {match.other_owner_avatar_url ? (
            <AuthenticatedImage imageUuid={match.other_owner_avatar_url} variant="thumbnail" style={{ width: 28, height: 28, borderRadius: 14 }} />
          ) : (
            <Ionicons name="person-circle" size={28} color={colors.grey} />
          )}
          <Text className="text-base font-bold" style={{ color: colors.text }}>
            {match.other_owner_name || 'Unknown'}
          </Text>
        </View>
      </View>

      <View className="flex-row items-center gap-3 px-4 pb-4">
        <MatchPetCard
          pet={match.liked_by_other_pet}
          onPress={(e) => {
            (e as any)?.stopPropagation?.();
            router.push(`/pet-view/${match.liked_by_other_pet.pet_uuid}`);
          }}
        />

        <Ionicons name="heart" size={24} color="#ef4444" />

        <MatchPetCard
          pet={match.liked_pet}
          onPress={(e) => {
            (e as any)?.stopPropagation?.();
            router.push(`/pet-view/${match.liked_pet.pet_uuid}`);
          }}
        />
      </View>

      <View className="px-4 pb-4">
        <TouchableOpacity
          onPress={() => router.push(`/likes/user-profile/${match.other_user_uuid}`)}
          style={{ backgroundColor: '#ef4444', borderRadius: 8, paddingVertical: 10, alignItems: 'center' }}>
          <Text style={{ color: '#fff', fontSize: 13, fontWeight: 600 }}>View Profile</Text>
        </TouchableOpacity>
      </View>
    </TouchableOpacity>
  );
}

export default function MatchesScreen() {
  const { colors, isDarkColorScheme } = useColorScheme();

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
          <View className="items-center gap-4">
            {matches?.map((match) => (
              <View key={match.liked_pet.pet_uuid} style={{ width: Platform.OS === 'web' ? '75%' : '100%' }}>
                <MatchRow match={match} colors={colors} isDarkColorScheme={isDarkColorScheme} />
              </View>
            ))}
          </View>
        )}
      </ScrollView>
    </View>
  );
}
