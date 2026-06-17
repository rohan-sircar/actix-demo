import React from 'react';
import { View, Text } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '../styles/Styles';
import type { Pet } from '~/app/models/pets';

const PetCard: React.FC<
  Pick<
    Pet,
    | 'id'
    | 'name'
    | 'species'
    | 'breed'
    | 'date_of_birth'
    | 'gender'
    | 'weight'
    | 'description'
    | 'traits'
  >
> = ({ name, species, breed, date_of_birth, gender, weight, description, traits }) => {
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

  const badgeBg = `${accentSet.bgSubtle}40`;
  const badgeColor = accentSet.textOnAccent;
  const secondaryColor = colors.grey;

  return (
    <View
      className="mb-3 rounded-xl p-4"
      style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
      <View className="flex-row items-start gap-3">
        <View
          className="flex-row items-center justify-center rounded-full"
          style={{ backgroundColor: `${accentSet.bgSubtle}60`, width: 48, height: 48 }}>
          <Ionicons name={getSpeciesIcon(species)} size={24} color={accentSet.base} />
        </View>
        <View className="flex-1">
          <View className="flex-row items-center gap-2">
            <Text className="text-lg font-semibold" style={{ color: colors.text }}>
              {name}
            </Text>
            {gender && (
              <Ionicons
                name={gender.toLowerCase() === 'male' ? 'male' : 'female'}
                size={16}
                color={gender.toLowerCase() === 'male' ? '#3b82f6' : '#ec4899'}
              />
            )}
          </View>
          <Text className="text-sm" style={{ color: secondaryColor }}>
            {species}
            {breed ? ` · ${breed}` : ''}
          </Text>
          <View className="mt-1 flex-row flex-wrap gap-1">
            {date_of_birth && (
              <View className="rounded-full px-2 py-0.5" style={{ backgroundColor: badgeBg }}>
                <Text className="text-xs" style={{ color: badgeColor }}>
                  {getAgeFromDob(date_of_birth)}
                </Text>
              </View>
            )}
            {weight && (
              <View className="rounded-full px-2 py-0.5" style={{ backgroundColor: badgeBg }}>
                <Text className="text-xs" style={{ color: badgeColor }}>
                  {weight}kg
                </Text>
              </View>
            )}
            {traits?.map((trait) => (
              <View
                key={trait}
                className="rounded-full px-2 py-0.5"
                style={{ backgroundColor: badgeBg }}>
                <Text className="text-xs" style={{ color: badgeColor }}>
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
      </View>
    </View>
  );
};

export default PetCard;
