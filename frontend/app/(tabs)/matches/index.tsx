import { Ionicons } from '@expo/vector-icons';
import React from 'react';
import { Image, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';

import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { getImageUrl, likesApi } from '~/app/lib/api';
import { useQuery } from '@tanstack/react-query';
import type { LikeWithPet } from '~/app/models/pets';

function MatchCard({ like, colors, isDarkColorScheme, accentSet }: {
  like: LikeWithPet;
  colors: any;
  isDarkColorScheme: boolean;
  accentSet: any;
}) {
  const router = useRouter();

  return (
    <TouchableOpacity
      onPress={() => router.push(`/pet-view/${like.pet_uuid}`)}
      style={{ width: '48%', borderRadius: 16, backgroundColor: colors.card, borderWidth: 1, borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8', shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.2, shadowRadius: 4, elevation: 3 }}>
      <View className="items-center">
        {like.primary_image_uuid ? (
          <Image source={{ uri: getImageUrl(like.primary_image_uuid, 'medium') }} style={{ width: '100%', height: 140, borderRadius: 16, borderBottomLeftRadius: 0, borderBottomRightRadius: 0 }} resizeMode="cover" />
        ) : (
          <View style={{ width: '100%', height: 140, backgroundColor: accentSet.bgSubtle || '#f0e0d8', justifyContent: 'center', alignItems: 'center', borderRadius: 16, borderBottomLeftRadius: 0, borderBottomRightRadius: 0 }}>
            <Ionicons name="paw" size={32} color={accentSet.base} />
          </View>
        )}
      </View>
      <View className="px-3 py-3">
        <Text className="text-sm font-bold" style={{ color: colors.text }}>{like.pet_name}</Text>
        <Text className="text-xs" style={{ color: colors.grey }}>{like.species}</Text>
      </View>
    </TouchableOpacity>
  );
}

export default function MatchesScreen() {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const { data: matches } = useQuery({
    queryKey: ['matches'],
    queryFn: () => likesApi.listMatches(),
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
          <View className="flex-row flex-wrap gap-4">
            {matches?.map((like) => (
              <MatchCard key={like.pet_uuid} like={like} colors={colors} isDarkColorScheme={isDarkColorScheme} accentSet={accentSet} />
            ))}
          </View>
        )}
      </ScrollView>
    </View>
  );
}
