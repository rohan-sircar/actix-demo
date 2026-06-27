import React, { useCallback, useEffect, useState } from 'react';
import {
  ActivityIndicator,
  View,
  Text,
  TouchableOpacity,
  Image as RNImage,
  ScrollView,
  Modal,
  Pressable,
  Alert,
} from 'react-native';
import { useRouter, useLocalSearchParams, useFocusEffect } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useMutation } from '@tanstack/react-query';

import { petApi, likesApi, getImageUrl } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { PublicPet, PetImage } from '~/app/models/pets';

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

export default function PetFullProfileScreen() {
  const router = useRouter();
  const { pet_uuid } = useLocalSearchParams<{ pet_uuid: string }>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const [pet, setPet] = useState<PublicPet | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [previewImage, setPreviewImage] = useState<string | null>(null);

  const fetchPet = async () => {
    try {
      const res = await petApi.getPet(pet_uuid);
      setPet(res);
    } catch (err) {
      console.error('[Profile] Failed to fetch pet:', err);
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

  const likeMutation = useMutation({
    mutationFn: (direction: 'like' | 'dislike') => likesApi.create(pet_uuid, direction),
    onSuccess: () => {
      // Pet is liked/disliked successfully
    },
  });

  const handleLike = () => {
    likeMutation.mutate('like');
    Alert.alert('Liked!', `${pet?.name} has been liked.`);
  };

  const handleDislike = () => {
    likeMutation.mutate('dislike');
  };

  const handleMessage = () => {
    Alert.alert('Message', 'Messaging is coming soon!');
  };

  const handleReport = () => {
    Alert.alert('Report', 'Report feature is coming soon!');
  };

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <ActivityIndicator size="large" color={accentSet.base} />
      </View>
    );
  }

  if (!pet) {
    return (
      <View className="flex-1 items-center justify-center px-8" style={{ backgroundColor: colors.background }}>
        <Ionicons name="alert-circle" size={48} color={colors.grey} />
        <Text className="mt-4 text-lg font-semibold" style={{ color: colors.text }}>
          Pet not found
        </Text>
        <Text className="mt-1 text-center text-sm" style={{ color: colors.grey }}>
          This pet may have been deleted
        </Text>
      </View>
    );
  }

  const heroUrl = pet.primary_image ? getImageUrl(pet.primary_image.uuid, 'medium') : undefined;
  const secondaryColor = colors.grey;
  const badgeBg = isDarkColorScheme ? colors.grey5 : `${accentSet.bgSubtle}80`;
  const badgeColor = accentSet.base;

  return (
    <ScrollView className="flex-1" style={{ backgroundColor: colors.background }}>
      {/* Hero Image */}
      <View style={{ position: 'relative' }}>
        {heroUrl ? (
          <TouchableOpacity onPress={() => setPreviewImage(heroUrl)} activeOpacity={0.9}>
            <RNImage
              source={{ uri: heroUrl }}
              style={{ width: '100%', aspectRatio: 1, maxHeight: 400 }}
              resizeMode="cover"
            />
          </TouchableOpacity>
        ) : (
          <View
            style={{
              width: '100%',
              aspectRatio: 1,
              maxHeight: 400,
              backgroundColor: isDarkColorScheme ? colors.grey5 : `${accentSet.bgSubtle}90`,
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Ionicons name={getSpeciesIcon(pet.species)} size={80} color={accentSet.base} />
          </View>
        )}

        {/* Back button */}
        <TouchableOpacity
          onPress={() => router.back()}
          className="absolute top-4 left-4 items-center justify-center rounded-full bg-black/40"
          style={{ width: 36, height: 36 }}
        >
          <Ionicons name="arrow-back" size={20} color="#fff" />
        </TouchableOpacity>
      </View>

      {/* Pet Name & Species */}
      <View className="px-4 pt-4 pb-2">
        <Text className="text-3xl font-bold" style={{ color: colors.text }}>
          {pet.name}
        </Text>
        <Text className="mt-1 text-base" style={{ color: secondaryColor }}>
          {pet.species}
          {pet.breed ? ` · ${pet.breed}` : ''}
        </Text>
      </View>

      {/* Details Section */}
      <View className="mx-4 mb-4">
        <View
          className="rounded-2xl px-5 py-4"
          style={{
            backgroundColor: isDarkColorScheme ? colors.card : colors.card,
            borderWidth: 1,
            borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
          }}
        >
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

      {/* About */}
      {pet.description && (
        <View className="mx-4 mb-4">
          <View
            className="rounded-2xl px-5 py-4"
            style={{
              backgroundColor: isDarkColorScheme ? colors.card : colors.card,
              borderWidth: 1,
              borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
            }}
          >
            <Text className="mb-2 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
              About
            </Text>
            <Text className="text-sm leading-relaxed" style={{ color: colors.text }}>
              {pet.description}
            </Text>
          </View>
        </View>
      )}

      {/* Traits */}
      {pet.traits && pet.traits.length > 0 && (
        <View className="mx-4 mb-4">
          <View
            className="rounded-2xl px-5 py-4"
            style={{
              backgroundColor: isDarkColorScheme ? colors.card : colors.card,
              borderWidth: 1,
              borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
            }}
          >
            <Text className="mb-3 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
              Traits
            </Text>
            <View className="flex-row flex-wrap gap-2">
              {pet.traits.map((trait) => (
                <View key={trait.id} className="rounded-full px-3.5 py-2" style={{ backgroundColor: badgeBg }}>
                  <Text className="text-sm font-semibold" style={{ color: badgeColor }}>
                    {trait.name}
                  </Text>
                </View>
              ))}
            </View>
          </View>
        </View>
      )}

      {/* Color & Markings */}
      {pet.color_markings && (
        <View className="mx-4 mb-4">
          <View
            className="rounded-2xl px-5 py-4"
            style={{
              backgroundColor: isDarkColorScheme ? colors.card : colors.card,
              borderWidth: 1,
              borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
            }}
          >
            <Text className="mb-2 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
              Color & Markings
            </Text>
            <Text className="text-sm leading-relaxed" style={{ color: colors.text }}>
              {pet.color_markings}
            </Text>
          </View>
        </View>
      )}

      {/* Owner Section */}
      <View className="mx-4 mb-4">
        <View
          className="rounded-2xl px-5 py-4"
          style={{
            backgroundColor: isDarkColorScheme ? colors.card : colors.card,
            borderWidth: 1,
            borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
          }}
        >
          <Text className="mb-3 text-sm font-bold uppercase tracking-wider" style={{ color: secondaryColor }}>
            Owner
          </Text>
          <View className="flex-row items-center">
            {/* Avatar placeholder */}
            <View
              className="rounded-full items-center justify-center"
              style={{
                width: 48,
                height: 48,
                backgroundColor: isDarkColorScheme ? colors.grey5 : `${accentSet.bgSubtle}90`,
              }}
            >
              <Ionicons name="person" size={24} color={accentSet.base} />
            </View>
            <View className="ml-3">
              <Text className="text-base font-semibold" style={{ color: colors.text }}>
                {pet.owner.display_name || 'Unknown Owner'}
              </Text>
              <Text className="text-sm" style={{ color: secondaryColor }}>
                {pet.owner.pets_owned} pet{pet.owner.pets_owned !== 1 ? 's' : ''} owned
              </Text>
            </View>
          </View>
        </View>
      </View>

      {/* Action Buttons */}
      <View className="mx-4 mb-8">
        <View className="flex-row gap-3">
          {/* Dislike */}
          <TouchableOpacity
            onPress={handleDislike}
            disabled={likeMutation.isPending}
            className="flex-1 items-center justify-center rounded-xl"
            style={{
              backgroundColor: isDarkColorScheme ? colors.grey5 : '#fee2e2',
              paddingVertical: 14,
              opacity: likeMutation.isPending ? 0.5 : 1,
            }}
          >
            <Ionicons name="close-circle" size={24} color="#ef4444" />
            <Text className="mt-1 text-xs font-semibold" style={{ color: '#ef4444' }}>
              Dislike
            </Text>
          </TouchableOpacity>

          {/* Like */}
          <TouchableOpacity
            onPress={handleLike}
            disabled={likeMutation.isPending}
            className="flex-1 items-center justify-center rounded-xl"
            style={{
              backgroundColor: accentSet.base,
              paddingVertical: 14,
              shadowColor: accentSet.base,
              shadowOffset: { width: 0, height: 2 },
              shadowOpacity: 0.3,
              shadowRadius: 4,
              elevation: 4,
              opacity: likeMutation.isPending ? 0.5 : 1,
            }}
          >
            <Ionicons name="heart" size={24} color="#fff" />
            <Text className="mt-1 text-xs font-semibold text-white">
              Like
            </Text>
          </TouchableOpacity>
        </View>

        <View className="mt-3 flex-row gap-3">
          {/* Message */}
          <TouchableOpacity
            onPress={handleMessage}
            className="flex-1 items-center justify-center rounded-xl"
            style={{
              backgroundColor: isDarkColorScheme ? colors.grey5 : accentSet.bgSubtle,
              paddingVertical: 12,
            }}
          >
            <Ionicons name="chatbubble" size={20} color={accentSet.base} />
            <Text className="mt-0.5 text-xs font-semibold" style={{ color: accentSet.base }}>
              Message
            </Text>
          </TouchableOpacity>

          {/* Report */}
          <TouchableOpacity
            onPress={handleReport}
            className="flex-1 items-center justify-center rounded-xl"
            style={{
              backgroundColor: isDarkColorScheme ? colors.grey5 : accentSet.bgSubtle,
              paddingVertical: 12,
            }}
          >
            <Ionicons name="flag" size={20} color={accentSet.base} />
            <Text className="mt-0.5 text-xs font-semibold" style={{ color: accentSet.base }}>
              Report
            </Text>
          </TouchableOpacity>
        </View>
      </View>

      {/* Image Preview Modal */}
      <Modal visible={previewImage !== null} transparent animationType="fade">
        <Pressable className="flex-1 items-center justify-center bg-black/90" onPress={() => setPreviewImage(null)}>
          <View style={{ width: '100%', height: '100%' }}>
            <RNImage
              source={{ uri: previewImage || '' }}
              style={{ width: '100%', height: '100%' }}
              resizeMode="contain"
            />
          </View>
        </Pressable>
      </Modal>
    </ScrollView>
  );
}
