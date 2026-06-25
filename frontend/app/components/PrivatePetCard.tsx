import React from 'react';
import { Platform, View, Text, Pressable, TouchableOpacity, Image } from 'react-native';
import { useRouter } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { Pet } from '~/app/models/pets';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';

const PrivatePetCard: React.FC<{
  pet_uuid: string;
  name: Pet['name'];
  species: Pet['species'];
  breed?: Pet['breed'];
  primary_image?: Pet['primary_image'];
  onDelete?: (pet_uuid: string) => void;
}> = ({ name, species, breed, primary_image, pet_uuid, onDelete }) => {
  const router = useRouter();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const secondaryColor = colors.grey;

  const imageUrl = primary_image
    ? `${API_BASE}/api/v1/pets/images/${primary_image.uuid}/medium`
    : null;

  const label = breed ? `${species} · ${breed}` : species;

  return (
    <TouchableOpacity
      onPress={() => router.push(`/pet-profiles/${pet_uuid}`)}
      activeOpacity={0.7}
      className="rounded-2xl overflow-hidden"
      style={{
        width: Platform.OS === 'web' ? '32%' : '48%',
        backgroundColor: colors.card,
        borderWidth: 1,
        borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8',
        shadowColor: '#000',
        shadowOffset: { width: 0, height: 2 },
        shadowOpacity: 0.08,
        shadowRadius: 4,
        elevation: 2,
      }}>
      <View style={{ height: Platform.OS === 'web' ? 210 : 200, backgroundColor: isDarkColorScheme ? colors.grey5 : `${accentSet.bgSubtle}90` }}>
        {imageUrl ? (
          <Image
            source={{ uri: imageUrl }}
            className="h-full w-full"
            resizeMode="cover"
          />
        ) : (
          <View className="h-full w-full items-center justify-center">
            <Ionicons name="paw" size={40} color={accentSet.base} />
          </View>
        )}
        <View className="absolute top-2 right-2 flex-row gap-2">
          <Pressable
            onPress={(e) => {
              e?.stopPropagation();
              router.push(`/pet-profiles/${pet_uuid}`);
            }}
            className="items-center justify-center rounded-full bg-black/40"
            style={({ pressed }) => ({
              width: 30,
              height: 30,
              opacity: pressed ? 0.6 : 1,
            })}>
            <Ionicons name="create-outline" size={14} color="#fff" />
          </Pressable>
          {onDelete ? (
            <Pressable
              onPress={(e) => {
                e?.stopPropagation();
                onDelete(pet_uuid);
              }}
              className="items-center justify-center rounded-full bg-black/40"
              style={({ pressed }) => ({
                width: 30,
                height: 30,
                opacity: pressed ? 0.6 : 1,
              })}>
              <Ionicons name="trash" size={14} color="#fff" />
            </Pressable>
          ) : null}
        </View>
      </View>
      <View className="px-3 pb-3 pt-2">
        <Text
          className="text-base font-bold"
          style={{ color: colors.text }}
          numberOfLines={1}>
          {name}
        </Text>
        <Text className="mt-0.5 text-xs font-medium" style={{ color: secondaryColor }}>
          {label}
        </Text>
      </View>
    </TouchableOpacity>
  );
};

export default PrivatePetCard;
