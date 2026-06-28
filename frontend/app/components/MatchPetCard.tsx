import React from 'react';
import { Platform, TouchableOpacity, View, Text, Image } from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { MatchPetInfo } from '~/app/models/pets';
import { getImageUrl } from '~/app/lib/api';

const MatchPetCard: React.FC<{
  pet: MatchPetInfo;
  onPress: (e?: any) => void;
}> = ({ pet, onPress }) => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const imageUrl = pet.primary_image_uuid
    ? getImageUrl(pet.primary_image_uuid, 'medium')
    : null;

  return (
    <TouchableOpacity
      onPress={onPress}
      activeOpacity={0.7}
      className="rounded-2xl overflow-hidden"
      style={{
        flex: 1,
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
      </View>
      <View className="px-3 pb-3 pt-2">
        <Text
          className="text-base font-bold"
          style={{ color: colors.text }}
          numberOfLines={1}>
          {pet.pet_name}
        </Text>
        <Text className="mt-0.5 text-xs font-medium" style={{ color: colors.grey }}>
          {pet.species}
        </Text>
      </View>
    </TouchableOpacity>
  );
};

export default MatchPetCard;
