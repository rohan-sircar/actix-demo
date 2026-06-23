import React, { useCallback, useEffect, useLayoutEffect, useState } from 'react';
import { ActivityIndicator, View, Text, Image as RNImage, ScrollView, TouchableOpacity, Modal, Pressable } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import * as ImagePicker from 'expo-image-picker';
import { useRouter, useLocalSearchParams, useFocusEffect } from 'expo-router';

import api from '~/app/lib/api';
import { petImageApi } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '~/app/styles/Styles';
import type { Pet, PetImage } from '~/app/models/pets';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';

const detectMimeTypeFromUri = async (uri: string, _fallback?: string): Promise<string> => {
  const lower = uri.toLowerCase();
  if (lower.startsWith('blob:') || lower.startsWith('data:')) {
    if (lower.startsWith('data:')) {
      const match = lower.match(/^data:([^;]+)/);
      if (match) return match[1];
    }
    const res = await fetch(uri);
    const buf = await res.arrayBuffer();
    const bytes = new Uint8Array(buf);
    if (bytes[0] === 0x89 && bytes[1] === 0x50 && bytes[2] === 0x4E && bytes[3] === 0x47) return 'image/png';
    if (bytes[0] === 0xFF && bytes[1] === 0xD8 && bytes[2] === 0xFF) return 'image/jpeg';
    if (bytes[0] === 0x47 && bytes[1] === 0x49 && bytes[2] === 0x46) return 'image/gif';
    const str = String.fromCharCode(...bytes.slice(0, 12));
    if (str.includes('RIFF') && str.includes('WEBP')) return 'image/webp';
  }
  if (lower.includes('.png')) return 'image/png';
  if (lower.includes('.webp')) return 'image/webp';
  if (lower.includes('.gif')) return 'image/gif';
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

export default function PetProfileEditScreen() {
  const router = useRouter();
  const { pet_uuid } = useLocalSearchParams<{ pet_uuid: string }>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const [pet, setPet] = useState<Pet | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const fetchPet = async () => {
    try {
      const res = await api.get<Pet>(`/api/v1/private/user/pets/${pet_uuid}`);
      setPet(res.data);
    } catch (err) {
      console.error('[PetProfile] Failed to fetch pet:', err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchPet();
  }, [pet_uuid]);

  useFocusEffect(
    useCallback(() => {
      fetchPet();
    }, [pet_uuid])
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
        const mimeType = await detectMimeTypeFromUri(uri, result.assets[0].type ?? undefined);
        console.log('[PetProfile] Uploading image:', uri, mimeType);
        await petImageApi.upload(pet.pet_uuid, uri, mimeType);
        fetchPet();
      } catch (err) {
        console.error('[PetProfile] Image upload failed:', err);
      }
    }
  };

  const badgeBg = `${accentSet.bgSubtle}80`;
  const badgeColor = accentSet.base;
  const secondaryColor = colors.grey;
  const [previewImage, setPreviewImage] = useState<boolean>(false);

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

  const imageUrl = pet.primary_image ? getImageUrl(pet.primary_image.uuid, 'medium') : undefined;

  return (
    <ScrollView className="flex-1">
      {/* Profile Header Card */}
      <View className="items-center pt-6 pb-6">
        <View className="relative">
          <TouchableOpacity
            onPress={() => pet.primary_image && setPreviewImage(true)}
            disabled={!pet.primary_image}>
            {imageUrl ? (
              <RNImage
                source={{ uri: imageUrl }}
                style={{
                  width: 160,
                  height: 160,
                  borderRadius: 80,
                  marginBottom: 16,
                }}
                resizeMode="cover"
              />
            ) : (
              <View
                className="mb-4 items-center justify-center rounded-full"
                style={{
                  width: 160,
                  height: 160,
                  backgroundColor: `${accentSet.bgSubtle}90`,
                }}>
                <Ionicons name={getSpeciesIcon(pet.species)} size={64} color={accentSet.base} />
              </View>
            )}
          </TouchableOpacity>

        </View>

        <Text className="text-3xl font-bold" style={{ color: colors.text }}>
          {pet.name}
        </Text>
        <Text className="mt-1 text-base" style={{ color: secondaryColor }}>
          {pet.species}
          {pet.breed ? ` · ${pet.breed}` : ''}
        </Text>

        {/* Action Buttons */}
        <View className="mt-5 flex-row w-full px-8">
          <TouchableOpacity
            onPress={() => router.push(`/pet-profiles/${pet_uuid}/edit`)}
            className="flex-1 items-center justify-center rounded-xl"
            style={{ backgroundColor: accentSet.base, paddingVertical: 8, shadowColor: accentSet.base, shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.3, shadowRadius: 4, elevation: 4 }}>
            <Ionicons name="pencil" size={16} color="#fff" />
            <Text className="mt-0.5 text-xs font-semibold text-white">
              Edit
            </Text>
          </TouchableOpacity>
          <View className="w-3" />
          <TouchableOpacity
            onPress={() => router.push(`/pet-profiles/${pet_uuid}/images`)}
            className="flex-1 items-center justify-center rounded-xl"
            style={{ backgroundColor: accentSet.bgSubtle, paddingVertical: 8 }}>
            <Ionicons name="images" size={16} color={accentSet.base} />
            <Text className="mt-0.5 text-xs font-semibold" style={{ color: accentSet.base }}>
              Manage Photos
            </Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* About */}
      {pet.description && (
        <View className="mx-4 mb-4">
          <View
            className="rounded-2xl px-5 py-4"
            style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
            <Text className="mb-2 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
              About
            </Text>
            <Text className="text-sm leading-relaxed" style={{ color: colors.text }}>
              {pet.description}
            </Text>
          </View>
        </View>
      )}


      {/* Details Section */}
      <View className="mx-4 mb-4">
        <View
          className="rounded-2xl px-5 py-4"
          style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
          <Text className="mb-3 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
            Details
          </Text>

          <View className="flex-row flex-wrap gap-2">
            {pet.gender && (
              <View className="flex-row items-center gap-1.5 rounded-full px-3.5 py-2" style={{ backgroundColor: badgeBg }}>
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
              <View className="flex-row items-center gap-1.5 rounded-full px-3.5 py-2" style={{ backgroundColor: badgeBg }}>
                <Ionicons name="calendar" size={16} color={badgeColor} />
                <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                  {getAgeFromDob(pet.date_of_birth)} old
                </Text>
              </View>
            )}

            {pet.weight && (
              <View className="flex-row items-center gap-1.5 rounded-full px-3.5 py-2" style={{ backgroundColor: badgeBg }}>
                <Ionicons name="scale" size={16} color={badgeColor} />
                <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                  {pet.weight} kg
                </Text>
              </View>
            )}
          </View>
        </View>
      </View>

      {/* Color & Markings */}
      {pet.color_markings && (
        <View className="mx-4 mb-4">
          <View
            className="rounded-2xl px-5 py-4"
            style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
            <Text className="mb-2 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
              Color & Markings
            </Text>
            <Text className="text-sm leading-relaxed" style={{ color: colors.text }}>
              {pet.color_markings}
            </Text>
          </View>
        </View>
      )}

      {/* Traits */}
      {pet.traits && pet.traits.length > 0 && (
        <View className="mx-4 mb-8">
          <View
            className="rounded-2xl px-5 py-4"
            style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
            <Text className="mb-3 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
              Traits
            </Text>
            <View className="flex-row flex-wrap gap-2">
              {pet.traits.map((trait) => (
                <View key={trait} className="rounded-full px-3.5 py-2" style={{ backgroundColor: badgeBg }}>
                  <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                    {trait}
                  </Text>
                </View>
              ))}
            </View>
          </View>
        </View>
      )}

      {/* Image Preview Modal */}
      <Modal visible={previewImage} transparent animationType="fade">
        <Pressable className="flex-1 items-center justify-center bg-black/80" onPress={() => setPreviewImage(false)}>
          <View style={{ width: '85%', maxWidth: 400, borderRadius: 16, overflow: 'hidden', backgroundColor: colors.card }}>
            <RNImage
              source={{ uri: pet.primary_image ? getImageUrl(pet.primary_image.uuid, 'medium') : '' }}
              style={{ width: '100%', aspectRatio: 1 }}
              resizeMode="cover"
            />
          </View>
        </Pressable>
      </Modal>
    </ScrollView>
  );
}
