import React, { useCallback, useRef, useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Modal,
  Pressable,
  Image as RNImage,
  ScrollView as ScrollViewNative,
  Dimensions,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { PublicPet, PetImage } from '~/app/models/pets';
import { getImageUrl, getAgeFromDob } from './gallery-utils';

interface Props {
  pet: PublicPet;
  images: PetImage[];
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
  onFullProfile: () => void;
}

export function NativeGallery({ pet, images, colors, accentSet, isDarkColorScheme, onFullProfile }: Props) {
  const scrollRef = useRef<ScrollViewNative>(null);
  const [currentPage, setCurrentPage] = useState(0);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const screenWidth = Dimensions.get('window').width;

  const handleMomentumEnd = useCallback((e: any) => {
    const contentOffsetX = e.nativeEvent.contentOffset.x;
    const page = Math.round(contentOffsetX / screenWidth);
    setCurrentPage(page);
  }, [screenWidth]);

  const fallbackImages: PetImage[] = pet.primary_image
    ? [{ id: 0, uuid: pet.primary_image.uuid, format: 'jpeg' as const, is_primary: true, sort_order: 0, created_at: '' }]
    : [];
  const allImages = images.length > 0 ? images : fallbackImages;

  return (
    <View style={{ flex: 1, backgroundColor: colors.background }}>
      {/* Gallery area */}
      <View style={{ flex: 1 }}>
        <ScrollViewNative
          ref={scrollRef}
          horizontal
          pagingEnabled
          showsHorizontalScrollIndicator={false}
          onMomentumScrollEnd={handleMomentumEnd}
          style={{ flex: 1 }}
          contentContainerStyle={{ flex: 0 }}
        >
          {allImages.map((image) => (
            <TouchableOpacity
              key={image.uuid}
              onPress={() => setPreviewUrl(getImageUrl(image.uuid, 'original'))}
              activeOpacity={0.9}
              style={{ width: screenWidth, height: '100%' }}
            >
              <RNImage
                source={{ uri: getImageUrl(image.uuid, 'medium') }}
                style={{ width: screenWidth, height: '100%' }}
                resizeMode="cover"
              />
            </TouchableOpacity>
          ))}
        </ScrollViewNative>

        {/* Page dots */}
        {allImages.length > 1 && (
          <View style={{ position: 'absolute', bottom: 16, left: 0, right: 0, flexDirection: 'row', alignItems: 'center', justifyContent: 'center' }}>
            {allImages.map((_, idx) => (
              <View
                key={idx}
                style={{
                  width: idx === currentPage ? 8 : 6,
                  height: idx === currentPage ? 8 : 6,
                  borderRadius: 4,
                  marginHorizontal: 3,
                  backgroundColor: idx === currentPage ? accentSet.base : `${colors.grey}80`,
                }}
              />
            ))}
          </View>
        )}
      </View>

      {/* Info section */}
      <View style={{ paddingHorizontal: 16, paddingBottom: 24, paddingTop: 16 }}>
        <View style={{ flexDirection: 'row', alignItems: 'center' }}>
          <Text style={{ fontSize: 24, fontWeight: 'bold', marginRight: 8, color: colors.text }}>
            {pet.name}
          </Text>
          {pet.date_of_birth && (
            <Text style={{ fontSize: 12, paddingHorizontal: 8, paddingVertical: 2, borderRadius: 999, color: colors.grey, backgroundColor: isDarkColorScheme ? 'rgba(128,128,128,0.2)' : 'rgba(128,128,128,0.15)' }}>
              {getAgeFromDob(pet.date_of_birth)}
            </Text>
          )}
        </View>
        <Text style={{ fontSize: 16, marginTop: 2, color: colors.grey }}>
          {pet.species}
          {pet.breed ? ` · ${pet.breed}` : ''}
        </Text>

        <TouchableOpacity
          onPress={onFullProfile}
          style={{
            marginTop: 16,
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'center',
            borderRadius: 16,
            paddingVertical: 14,
            backgroundColor: accentSet.base,
            shadowColor: accentSet.base,
            shadowOffset: { width: 0, height: 2 },
            shadowOpacity: 0.3,
            shadowRadius: 4,
            elevation: 4,
          }}
        >
          <Ionicons name="information-circle" size={20} color="#fff" />
          <Text style={{ marginLeft: 8, fontSize: 16, fontWeight: '600', color: '#fff' }}>
            Full Profile
          </Text>
        </TouchableOpacity>
      </View>

      {/* Preview modal */}
      <Modal visible={previewUrl !== null} transparent animationType="fade">
        <Pressable style={{ flex: 1, alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.9)' }} onPress={() => setPreviewUrl(null)}>
          <View style={{ width: '90%', aspectRatio: 1, borderRadius: 12, overflow: 'hidden' }}>
            <RNImage
              source={{ uri: previewUrl || '' }}
              style={{ width: '100%', height: '100%' }}
              resizeMode="contain"
            />
          </View>
        </Pressable>
      </Modal>
    </View>
  );
}
