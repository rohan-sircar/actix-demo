import React from 'react';
import { Platform, View, Text, Pressable, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '../styles/Styles';
import type { Pet } from '~/app/models/pets';
import { AuthenticatedImage } from './AuthenticatedImage';

const PetCard: React.FC<{
  pet_uuid: string;
  name: Pet['name'];
  species: Pet['species'];
  breed?: Pet['breed'];
  date_of_birth?: Pet['date_of_birth'];
  gender?: Pet['gender'];
  weight?: Pet['weight'];
  description?: Pet['description'];
  traits?: Pet['traits'];
  primary_image?: Pet['primary_image'];
  onDelete?: (pet_uuid: string) => void;
  onPress?: () => void;
}> = ({ name, species, breed, description, primary_image, pet_uuid, onDelete, onPress }) => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const secondaryColor = colors.grey;

  const label = breed ? `${species} · ${breed}` : species;

  return (
    <TouchableOpacity
      onPress={onPress}
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
        {primary_image ? (
          <AuthenticatedImage
            imageUuid={primary_image.uuid}
            variant="medium"
            style={{ width: '100%', height: '100%' }}
          />
        ) : (
          <View className="h-full w-full items-center justify-center">
            <Ionicons name="paw" size={40} color={accentSet.base} />
          </View>
        )}
        {onDelete ? (
          <Pressable
            onPress={(e) => {
              e?.stopPropagation();
              onDelete(pet_uuid);
            }}
            className="absolute top-2 right-2 items-center justify-center rounded-full bg-black/40"
            style={({ pressed }) => ({
              width: 30,
              height: 30,
              opacity: pressed ? 0.6 : 1,
            })}>
            <Ionicons name="trash" size={14} color="#fff" />
          </Pressable>
        ) : null}
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
        {description ? (
          <Text
            className="mt-1 text-xs leading-relaxed"
            style={{ color: secondaryColor }}
            numberOfLines={2}>
            {description}
          </Text>
        ) : null}
      </View>
    </TouchableOpacity>
  );
};

export default PetCard;
