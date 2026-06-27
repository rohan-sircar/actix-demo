import React, { useCallback, useEffect, useState } from 'react';
import { Platform, View, ActivityIndicator, Text, StyleSheet } from 'react-native';
import { useRouter, useLocalSearchParams, useFocusEffect } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

import { petApi } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { PublicPet, PetImage } from '~/app/models/pets';
import { WebGallery } from '../pet-view/WebGallery';
import { NativeGallery } from '../pet-view/NativeGallery';

const isWeb = Platform.OS === 'web';

function LoadingState({ colors, accentSet }: { colors: any; accentSet: any }) {
  if (isWeb) {
    return (
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh', backgroundColor: colors.background }}>
        <div className="animate-spin rounded-full h-12 w-12 border-b-2" style={{ borderColor: accentSet.base, borderWidth: 3 }} />
      </div>
    );
  }
  return (
    <View style={[styles.loadingContainer, { backgroundColor: colors.background }]}>
      <ActivityIndicator size="large" color={accentSet.base} />
    </View>
  );
}

function ErrorState({ colors }: { colors: any }) {
  if (isWeb) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: 32, height: '100vh', backgroundColor: colors.background }}>
        <Ionicons name="alert-circle" size={48} color={colors.grey} />
        <h2 style={{ marginTop: 16, fontSize: 18, fontWeight: 600, color: colors.text }}>Pet not found</h2>
        <p style={{ marginTop: 4, textAlign: 'center', fontSize: 14, color: colors.grey }}>This pet may have been deleted</p>
      </div>
    );
  }
  return (
    <View style={[styles.errorContainer, { backgroundColor: colors.background }]}>
      <Ionicons name="alert-circle" size={48} color={colors.grey} />
      <Text style={[styles.errorTitle, { color: colors.text }]}>Pet not found</Text>
      <Text style={[styles.errorSubtitle, { color: colors.grey }]}>This pet may have been deleted</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  loadingContainer: { flex: 1, alignItems: 'center', justifyContent: 'center' },
  errorContainer: { flex: 1, alignItems: 'center', justifyContent: 'center', padding: 32 },
  errorTitle: { marginTop: 16, fontSize: 18, fontWeight: '600' },
  errorSubtitle: { marginTop: 4, textAlign: 'center', fontSize: 14 },
});

export default function PetPhotoGalleryPreviewScreen() {
  const { pet_uuid } = useLocalSearchParams<{ pet_uuid: string }>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const [pet, setPet] = useState<PublicPet | null>(null);
  const [images, setImages] = useState<PetImage[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const fetchPet = async () => {
    try {
      const res = await petApi.getPet(pet_uuid);
      setPet(res);
    } catch (err) {
      console.error('[Gallery Preview] Failed to fetch pet:', err);
    }
  };

  const fetchImages = async () => {
    try {
      const res = await petApi.getImages(pet_uuid);
      setImages(res.sort((a, b) => a.sort_order - b.sort_order));
    } catch (err) {
      console.error('[Gallery Preview] Failed to fetch images:', err);
    }
  };

  useEffect(() => {
    Promise.all([fetchPet(), fetchImages()]).finally(() => setIsLoading(false));
  }, [pet_uuid]);

  useFocusEffect(
    useCallback(() => {
      Promise.all([fetchPet(), fetchImages()]).finally(() => setIsLoading(false));
    }, [pet_uuid])
  );

  if (isLoading) {
    return <LoadingState colors={colors} accentSet={accentSet} />;
  }

  if (!pet) {
    return <ErrorState colors={colors} />;
  }

  const onFullProfile = () => {};

  if (isWeb) {
    return (
      <WebGallery
        pet={pet}
        images={images}
        colors={colors}
        accentSet={accentSet}
        isDarkColorScheme={isDarkColorScheme}
        onFullProfile={onFullProfile}
      />
    );
  }

  return (
    <NativeGallery
      pet={pet}
      images={images}
      colors={colors}
      accentSet={accentSet}
      isDarkColorScheme={isDarkColorScheme}
      onFullProfile={onFullProfile}
    />
  );
}
