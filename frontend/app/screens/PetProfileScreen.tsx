import React, { useCallback, useLayoutEffect, useState } from 'react';
import { ActivityIndicator, View, Text, Image as RNImage, ScrollView, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useNavigation } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import * as ImagePicker from 'expo-image-picker';

import api from '~/app/lib/api';
import { petImageApi } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '~/app/styles/Styles';
import type { Pet, PetImage } from '~/app/models/pets';
import { TabStackParamList } from '~/types/navigation';
import { useFocusEffect } from '@react-navigation/native';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800/api/v1';

const detectMimeTypeFromUri = async (uri: string, fallback?: string): Promise<string> => {
  if (fallback) return fallback;
  const lower = uri.toLowerCase();
  if (lower.includes('.png')) return 'image/png';
  if (lower.includes('.webp')) return 'image/webp';
  if (lower.includes('.gif')) return 'image/gif';
  if (lower.startsWith('blob:')) {
    const res = await fetch(uri);
    const blob = await res.blob();
    return blob.type || 'image/jpeg';
  }
  return 'image/jpeg';
};

const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'thumbnail'): string => {
  return `${API_BASE}/api/v1/pets/images/${imageUuid}/${variant}`;
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
  const navigation = useNavigation<NativeStackNavigationProp<TabStackParamList>>();
  const queryClient = useQueryClient();
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

  useFocusEffect(
    useCallback(() => {
      queryClient.invalidateQueries({ queryKey: ['pet', pet_uuid] });
    }, [queryClient, pet_uuid])
  );

  const handlePickImage = async () => {
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [1, 1],
      quality: 0.8,
    });

    if (!result.canceled && result.assets[0] && pet) {
      try {
        const uri = result.assets[0].uri;
        const mimeType = await detectMimeTypeFromUri(uri, result.assets[0].type);
        console.log('[PetProfile] Uploading image:', uri, mimeType);
        await petImageApi.upload(pet.pet_uuid, uri, mimeType);
        queryClient.invalidateQueries({ queryKey: ['pet', pet_uuid] });
      } catch (err) {
        console.error('[PetProfile] Image upload failed:', err);
      }
    }
  };

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

  const imageUrl = pet.primary_image ? getImageUrl(pet.primary_image.uuid, 'thumbnail') : undefined;

  return (
    <ScrollView className="flex-1">
      <View className="items-center pt-6 pb-4">
        <View className="relative">
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

          <TouchableOpacity
            onPress={handlePickImage}
            className="absolute -bottom-1 -right-1 items-center justify-center rounded-full"
            style={{ width: 36, height: 36, backgroundColor: accentSet.base }}>
            <Ionicons name="add" size={20} color="#fff" />
          </TouchableOpacity>
        </View>

        <Text className="text-2xl font-bold" style={{ color: colors.text }}>
          {pet.name}
        </Text>

        <Text className="mt-1 text-base font-medium" style={{ color: secondaryColor }}>
          {pet.species}
          {pet.breed ? ` · ${pet.breed}` : ''}
        </Text>

        <TouchableOpacity
          onPress={() => navigation.navigate('ImageGallery', { pet_uuid })}
          className="mt-3 flex-row items-center gap-1.5"
          style={{ backgroundColor: accentSet.bgSubtle, paddingHorizontal: 12, paddingVertical: 6, borderRadius: 8 }}>
          <Ionicons name="images" size={16} color={accentSet.base} />
          <Text className="text-sm font-semibold" style={{ color: accentSet.base }}>
            Manage Photos
          </Text>
        </TouchableOpacity>
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
