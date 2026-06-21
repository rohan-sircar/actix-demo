import React from 'react';
import { View, Text, Pressable, TouchableOpacity } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '../styles/Styles';
import type { Pet } from '~/app/models/pets';

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
  onDelete?: (pet_uuid: string) => void;
  onPress?: () => void;
}> = ({ name, species, breed, date_of_birth, gender, weight, description, traits, pet_uuid, onDelete, onPress }) => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

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

  const badgeBg = `${accentSet.bgSubtle}80`;
  const badgeColor = accentSet.base;
  const secondaryColor = colors.grey;

  return (
    <TouchableOpacity
      onPress={onPress}
      activeOpacity={0.7}
      className="mb-3 rounded-2xl px-4 py-3"
      style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
      <View className="flex-row items-start gap-3">
        <View
          className="items-center justify-center rounded-2xl"
          style={{
            backgroundColor: `${accentSet.bgSubtle}90`,
            width: 52,
            height: 52,
          }}>
          <Ionicons name={getSpeciesIcon(species)} size={24} color={accentSet.base} />
        </View>
        <View className="flex-1">
          <View className="flex-row items-center gap-2">
            <Text className="text-lg font-bold" style={{ color: colors.text }}>
              {name}
            </Text>
            {gender && (
              <Ionicons
                name={gender.toLowerCase() === 'male' ? 'male' : 'female'}
                size={16}
                color={gender.toLowerCase() === 'male' ? '#5B8DEF' : '#F472B6'}
              />
            )}
          </View>
          <Text className="text-sm font-medium" style={{ color: secondaryColor }}>
            {species}
            {breed ? ` · ${breed}` : ''}
          </Text>
          <View className="mt-1.5 flex-row flex-wrap gap-1.5">
            {date_of_birth && (
              <View className="rounded-full px-2.5 py-1" style={{ backgroundColor: badgeBg }}>
                <Text className="text-xs font-semibold" style={{ color: badgeColor }}>
                  {getAgeFromDob(date_of_birth)}
                </Text>
              </View>
            )}
            {weight && (
              <View className="rounded-full px-2.5 py-1" style={{ backgroundColor: badgeBg }}>
                <Text className="text-xs font-semibold" style={{ color: badgeColor }}>
                  {weight}kg
                </Text>
              </View>
            )}
            {traits?.map((trait) => (
              <View
                key={trait}
                className="rounded-full px-2.5 py-1"
                style={{ backgroundColor: badgeBg }}>
                <Text className="text-xs font-semibold" style={{ color: badgeColor }}>
                  {trait}
                </Text>
              </View>
            ))}
          </View>
          {description && (
            <Text
              className="mt-2 line-clamp-2 text-sm leading-relaxed"
              style={{ color: secondaryColor }}>
              {description}
            </Text>
          )}
        </View>
        {onDelete ? (
          <Pressable
            onPress={(e) => {
              e?.stopPropagation();
              onDelete(pet_uuid);
            }}
            className="items-center justify-center rounded-lg"
            style={({ pressed }) => ({ padding: 8, opacity: pressed ? 0.6 : 1 })}>
            <Ionicons name="trash" size={18} color="#E11D48" />
          </Pressable>
        ) : null}
      </View>
    </TouchableOpacity>
  );
};

export default PetCard;
