import React, { useLayoutEffect } from 'react';
import { ActivityIndicator, View, Text, Image as RNImage, ScrollView } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useQuery } from '@tanstack/react-query';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';

import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '~/app/styles/Styles';
import type { Pet, PetImage } from '~/app/models/pets';
import { RootStackParamList } from '~/types/navigation';

const getImageUrl = (image: PetImage | null): string | undefined => {
  if (!image) return undefined;
  return `http://localhost:8800/api/v1/pets/images/${image.uuid}/thumbnail`;
};

const getAgeFromDob = (dob: string) => {
  const birth = new Date(dob);
  const today = new Date();
  const ageYears = today.getFullYear() - birth.getFullYear();
  const ageMonths = today.getMonth() - birth.getMonth();
  if (ageYears > 0) {
    return `${ageYears}y${ageMonths > 0 ? ` ${ageMonths}m` : ''}`;
  }
  const ageDays = Math.floor((today.getTime() - birth.getTime()) / (1000 * 60 * 60 * 24));
  if (ageDays > 30) {
    return `${Math.floor(ageDays / 30)}m`;
  }
  return `${ageDays}d`;
};

const getSpeciesIcon = (sp: string) => {
  const lower = sp.toLowerCase();
  if (lower.includes('dog')) return 'paw' as const;
  if (lower.includes('cat')) return 'paw' as const;
  if (lower.includes('bird')) return 'planet' as const;
  if (lower.includes('fish')) return 'water' as const;
  if (lower.includes('reptile') || lower.includes('snake') || lower.includes('lizard'))
    return 'leaf' as const;
  return 'paw' as const;
};

type PetProfileScreenProps = {
  route: { params: { pet_uuid: string } };
};

export default function PetProfileScreen({ route }: PetProfileScreenProps) {
  const { pet_uuid } = route.params;
  const navigation = useNavigation<NativeStackNavigationProp<RootStackParamList>>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const { data: pet, isLoading } = useQuery({
    queryKey: ['pet', pet_uuid],
    queryFn: async () => {
      const res = await api.get<Pet>(`/api/v1/private/user/pets/${pet_uuid}`);
      return res.data;
    },
  });

  useLayoutEffect(() => {
    navigation.setOptions({
      headerTitle: pet?.name || 'Pet Profile',
    });
  }, [navigation, pet?.name]);

  const badgeBg = `${accentSet.bgSubtle}80`;
  const badgeColor = accentSet.base;
  const secondaryColor = colors.grey;

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center">
        <ActivityIndicator size="large" color={accentSet.base} />
      </View>
    );
  }

  if (!pet) {
    return (
      <View className="flex-1 items-center justify-center px-8">
        <Ionicons name="alert-circle" size={48} color={colors.grey} />
        <Text className="mt-4 text-lg font-semibold" style={{ color: colors.text }}>
          Pet not found
        </Text>
        <Text className="mt-1 text-center text-sm" style={{ color: secondaryColor }}>
          This pet may have been deleted
        </Text>
      </View>
    );
  }

  const imageUrl = pet.primary_image
    ? `http://localhost:8800/api/v1/pets/images/${pet.primary_image.uuid}/thumbnail`
    : undefined;

  return (
    <ScrollView className="flex-1">
      <View className="items-center pt-6 pb-4">
        {imageUrl ? (
          <RNImage
            source={{ uri: imageUrl }}
            style={{
              width: 180,
              height: 180,
              borderRadius: 90,
              marginBottom: 16,
            }}
            resizeMode="cover"
          />
        ) : (
          <View
            className="mb-4 items-center justify-center rounded-full"
            style={{
              width: 180,
              height: 180,
              backgroundColor: `${accentSet.bgSubtle}90`,
            }}>
            <Ionicons name={getSpeciesIcon(pet.species)} size={72} color={accentSet.base} />
          </View>
        )}

        <Text className="text-2xl font-bold" style={{ color: colors.text }}>
          {pet.name}
        </Text>

        <Text className="mt-1 text-base font-medium" style={{ color: secondaryColor }}>
          {pet.species}
          {pet.breed ? ` · ${pet.breed}` : ''}
        </Text>
      </View>

      <View className="mx-4 gap-4 pb-8">
        <View
          className="rounded-2xl px-4 py-3"
          style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
          <Text className="mb-2 text-sm font-semibold uppercase tracking-wide" style={{ color: secondaryColor }}>
            Details
          </Text>

          <View className="flex-row flex-wrap gap-2">
            {pet.gender && (
              <View className="flex-row items-center gap-1.5 rounded-full px-3 py-1.5" style={{ backgroundColor: badgeBg }}>
                <Ionicons
                  name={pet.gender.toLowerCase() === 'male' ? 'male' : 'female'}
                  size={16}
                  color={pet.gender.toLowerCase() === 'male' ? '#5B8DEF' : '#F472B6'}
                />
                <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                  {pet.gender}
                </Text>
              </View>
            )}

            {pet.date_of_birth && (
              <View className="rounded-full px-3 py-1.5" style={{ backgroundColor: badgeBg }}>
                <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                  {getAgeFromDob(pet.date_of_birth)} old
                </Text>
              </View>
            )}

            {pet.weight && (
              <View className="rounded-full px-3 py-1.5" style={{ backgroundColor: badgeBg }}>
                <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                  {pet.weight} kg
                </Text>
              </View>
            )}
          </View>
        </View>

        {pet.color_markings && (
          <View
            className="rounded-2xl px-4 py-3"
            style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
            <Text className="mb-1.5 text-sm font-semibold uppercase tracking-wide" style={{ color: secondaryColor }}>
              Color & Markings
            </Text>
            <Text className="text-sm leading-relaxed" style={{ color: colors.text }}>
              {pet.color_markings}
            </Text>
          </View>
        )}

        {pet.description && (
          <View
            className="rounded-2xl px-4 py-3"
            style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
            <Text className="mb-1.5 text-sm font-semibold uppercase tracking-wide" style={{ color: secondaryColor }}>
              About
            </Text>
            <Text className="text-sm leading-relaxed" style={{ color: colors.text }}>
              {pet.description}
            </Text>
          </View>
        )}

        {pet.traits && pet.traits.length > 0 && (
          <View
            className="rounded-2xl px-4 py-3"
            style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
            <Text className="mb-2 text-sm font-semibold uppercase tracking-wide" style={{ color: secondaryColor }}>
              Traits
            </Text>
            <View className="flex-row flex-wrap gap-2">
              {pet.traits.map((trait) => (
                <View key={trait} className="rounded-full px-3 py-1.5" style={{ backgroundColor: badgeBg }}>
                  <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                    {trait}
                  </Text>
                </View>
              ))}
            </View>
          </View>
        )}
      </View>
    </ScrollView>
  );
}
