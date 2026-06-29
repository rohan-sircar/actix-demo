import React, { useCallback, useEffect, useState } from 'react';
import { Platform, View, ActivityIndicator, Text, StyleSheet, Alert, Image, TouchableOpacity, ScrollView } from 'react-native';
import { useRouter, useLocalSearchParams, useFocusEffect } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';

import { petApi, likesApi, getImageUrl } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useAuthStore } from '~/app/stores/AuthStore';
import type { PublicPet, PetImage, MatchPetInfo } from '~/app/models/pets';
import { WebGallery } from './WebGallery';
import { NativeGallery } from './NativeGallery';

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

export default function PetPhotoGalleryScreen() {
  const router = useRouter();
  const { pet_uuid } = useLocalSearchParams<{ pet_uuid: string }>();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const { user } = useAuthStore();
  const [pet, setPet] = useState<PublicPet | null>(null);
  const [images, setImages] = useState<PetImage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [interaction, setInteraction] = useState<{ interacted: boolean; direction: string | null } | null>(null);
  const [potentialMatches, setPotentialMatches] = useState<MatchPetInfo[]>([]);
  const [showMatchDialog, setShowMatchDialog] = useState(false);
  const [selectedReciprocalPet, setSelectedReciprocalPet] = useState<string | null>(null);
  const [isSubmittingLike, setIsSubmittingLike] = useState(false);

  const isOwnPet = pet?.owner.user_uuid === user?.user_uuid;

  const fetchPet = async () => {
    try {
      const res = await petApi.getPet(pet_uuid);
      setPet(res);
    } catch (err) {
      console.error('[Gallery] Failed to fetch pet:', err);
    }
  };

  const fetchImages = async () => {
    try {
      const res = await petApi.getImages(pet_uuid);
      setImages(res.sort((a, b) => a.sort_order - b.sort_order));
    } catch (err) {
      console.error('[Gallery] Failed to fetch images:', err);
    }
  };

  const fetchInteraction = async () => {
    try {
      const res = await likesApi.checkInteraction(pet_uuid);
      setInteraction({ interacted: res.interacted, direction: res.direction });
      setPotentialMatches(res.potential_matches ?? []);
    } catch (err) {
      console.error('[Gallery] Failed to fetch interaction:', err);
    }
  };

  useEffect(() => {
    Promise.all([fetchPet(), fetchImages(), fetchInteraction()]).finally(() => setIsLoading(false));
  }, [pet_uuid]);

  useFocusEffect(
    useCallback(() => {
      Promise.all([fetchPet(), fetchImages(), fetchInteraction()]).finally(() => setIsLoading(false));
    }, [pet_uuid])
  );

  if (isLoading) {
    return <LoadingState colors={colors} accentSet={accentSet} />;
  }

  if (!pet) {
    return <ErrorState colors={colors} />;
  }

  const onFullProfile = () => router.push(`/pet-view/profile/${pet_uuid}`);

  const submitLike = async (reciprocalPetUuid?: string) => {
    if (isSubmittingLike) return;
    setIsSubmittingLike(true);
    try {
      await likesApi.create(pet_uuid, 'like', reciprocalPetUuid);
      setInteraction({ interacted: true, direction: 'like' });
      setShowMatchDialog(false);
      if (reciprocalPetUuid) {
        Alert.alert('Match! 🎉', `You and ${pet?.owner.display_name || 'someone'} have a match!`);
      } else {
        Alert.alert('Liked!', `${pet?.name} has been liked.`);
      }
    } catch (err: any) {
      console.error('[Gallery] Like failed:', err);
      if (err?.response?.status === 409) {
        Alert.alert('Cannot Like', 'You cannot like your own pet.');
      } else if (err?.response?.status === 400) {
        Alert.alert('Error', 'Something went wrong with the match. Please try again.');
      }
    } finally {
      setIsSubmittingLike(false);
    }
  };

  const onLike = async () => {
    if (interaction?.interacted || isOwnPet) return;
    if (potentialMatches.length > 0) {
      setShowMatchDialog(true);
      return;
    }
    await submitLike();
  };

  const onDislike = async () => {
    if (interaction?.interacted || isOwnPet) return;
    try {
      await likesApi.create(pet_uuid, 'dislike');
      setInteraction({ interacted: true, direction: 'dislike' });
    } catch (err: any) {
      console.error('[Gallery] Dislike failed:', err);
      if (err?.response?.status === 409) {
        Alert.alert('Cannot Dislike', 'You cannot dislike your own pet.');
      }
    }
  };

  if (isWeb) {
    return (
      <>
        <WebGallery
          pet={pet}
          images={images}
          colors={colors}
          accentSet={accentSet}
          isDarkColorScheme={isDarkColorScheme}
          onFullProfile={onFullProfile}
          onLike={onLike}
          onDislike={onDislike}
          interactionState={interaction || undefined}
          isOwnPet={isOwnPet}
        />
        {showMatchDialog && (
          <div style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0, zIndex: 100 }}>
            <div className="fixed inset-0 bg-black/50" onClick={() => !isSubmittingLike && setShowMatchDialog(false)} />
            <div className="fixed bottom-0 left-0 right-0 rounded-t-2xl p-5 max-h-[60%]" style={{ backgroundColor: colors.background }}>
              <h3 className="text-lg font-bold mb-1" style={{ color: colors.text }}>You have a match! 🎉</h3>
              <p className="text-sm mb-4" style={{ color: colors.grey }}>
                {pet?.owner.display_name || 'Someone'} liked one of your pets. Choose which pet to match with:
              </p>
              <div className="overflow-y-auto max-h-60 mb-4">
                {potentialMatches.map((match) => (
                  <div
                    key={match.pet_uuid}
                    onClick={() => setSelectedReciprocalPet(match.pet_uuid)}
                    className="flex items-center p-3 rounded-xl mb-2 cursor-pointer"
                    style={{
                      backgroundColor: selectedReciprocalPet === match.pet_uuid
                        ? accentSet.base : (isDarkColorScheme ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.03)'),
                    }}
                  >
                    {match.primary_image_uuid ? (
                      <img src={getImageUrl(match.primary_image_uuid, 'thumbnail')} alt=""
                        className="w-12 h-12 rounded-lg mr-3 object-cover" />
                    ) : (
                      <div className="w-12 h-12 rounded-lg mr-3 flex items-center justify-center"
                        style={{ backgroundColor: accentSet.bgSubtle || '#f0e0d8' }}>
                        <Ionicons name="paw" size={20} color={accentSet.base} />
                      </div>
                    )}
                    <div className="flex-1">
                      <div className="font-semibold text-sm" style={{ color: colors.text }}>{match.pet_name}</div>
                      <div className="text-xs" style={{ color: colors.grey }}>{match.species}</div>
                    </div>
                    <Ionicons
                      name={selectedReciprocalPet === match.pet_uuid ? 'checkmark-circle' : 'ellipse-outline'}
                      size={24}
                      color={selectedReciprocalPet === match.pet_uuid ? accentSet.base : colors.grey}
                    />
                  </div>
                ))}
              </div>
              <div className="flex gap-3">
                <button onClick={() => setShowMatchDialog(false)}
                  className="flex-1 py-3 rounded-xl font-semibold text-sm"
                  style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)', color: colors.grey }}>
                  Skip
                </button>
                <button onClick={() => submitLike(selectedReciprocalPet ?? undefined)} disabled={!selectedReciprocalPet || isSubmittingLike}
                  className="flex-1 py-3 rounded-xl font-semibold text-sm text-white"
                  style={{
                    backgroundColor: selectedReciprocalPet && !isSubmittingLike ? accentSet.base : (accentSet.base + '40'),
                    opacity: selectedReciprocalPet && !isSubmittingLike ? 1 : 0.5,
                  }}>
                  {isSubmittingLike ? '...' : 'Confirm Match'}
                </button>
              </div>
            </div>
          </div>
        )}
      </>
    );
  }

  return (
    <>
      <NativeGallery
        pet={pet}
        images={images}
        colors={colors}
        accentSet={accentSet}
        isDarkColorScheme={isDarkColorScheme}
        onFullProfile={onFullProfile}
        onLike={onLike}
        onDislike={onDislike}
        interactionState={interaction || undefined}
        isOwnPet={isOwnPet}
      />
      {showMatchDialog && (
        <View style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, zIndex: 100 }}>
          <TouchableOpacity
            style={{ flex: 1, backgroundColor: 'rgba(0,0,0,0.5)' }}
            activeOpacity={1}
            onPress={() => !isSubmittingLike && setShowMatchDialog(false)}
          />
          <View style={{
            position: 'absolute', bottom: 0, left: 0, right: 0,
            backgroundColor: colors.background,
            borderTopLeftRadius: 20, borderTopRightRadius: 20,
            padding: 20, maxHeight: '60%',
          }}>
            <Text style={{ fontSize: 18, fontWeight: 700, color: colors.text, marginBottom: 4 }}>
              You have a match! 🎉
            </Text>
            <Text style={{ fontSize: 14, color: colors.grey, marginBottom: 16 }}>
              {pet?.owner.display_name || 'Someone'} liked one of your pets. Choose which pet to match with:
            </Text>
            <ScrollView showsVerticalScrollIndicator={false} style={{ maxHeight: 240 }}>
              {potentialMatches.map((match) => (
                <TouchableOpacity
                  key={match.pet_uuid}
                  onPress={() => setSelectedReciprocalPet(match.pet_uuid)}
                  style={{
                    flexDirection: 'row', alignItems: 'center', padding: 12, borderRadius: 12,
                    marginBottom: 8,
                    backgroundColor: selectedReciprocalPet === match.pet_uuid
                      ? accentSet.base : (isDarkColorScheme ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.03)'),
                  }}
                >
                  {match.primary_image_uuid ? (
                    <Image source={{ uri: getImageUrl(match.primary_image_uuid, 'thumbnail') }}
                      style={{ width: 48, height: 48, borderRadius: 12, marginRight: 12 }} resizeMode="cover" />
                  ) : (
                    <View style={{ width: 48, height: 48, borderRadius: 12,
                      backgroundColor: accentSet.bgSubtle || '#f0e0d8',
                      justifyContent: 'center', alignItems: 'center', marginRight: 12 }}>
                      <Ionicons name="paw" size={20} color={accentSet.base} />
                    </View>
                  )}
                  <View style={{ flex: 1 }}>
                    <Text style={{ fontSize: 15, fontWeight: 600, color: colors.text }}>{match.pet_name}</Text>
                    <Text style={{ fontSize: 13, color: colors.grey }}>{match.species}</Text>
                  </View>
                  <Ionicons
                    name={selectedReciprocalPet === match.pet_uuid ? 'checkmark-circle' : 'ellipse-outline'}
                    size={24}
                    color={selectedReciprocalPet === match.pet_uuid ? accentSet.base : colors.grey}
                  />
                </TouchableOpacity>
              ))}
            </ScrollView>
            <View style={{ flexDirection: 'row', gap: 12, marginTop: 16 }}>
              <TouchableOpacity
                onPress={() => setShowMatchDialog(false)}
                style={{ flex: 1, paddingVertical: 14, borderRadius: 12,
                  backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)',
                  alignItems: 'center' }}>
                <Text style={{ fontSize: 15, fontWeight: 600, color: colors.grey }}>Skip</Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={() => submitLike(selectedReciprocalPet ?? undefined)}
                disabled={!selectedReciprocalPet || isSubmittingLike}
                style={{ flex: 1, paddingVertical: 14, borderRadius: 12,
                  backgroundColor: selectedReciprocalPet && !isSubmittingLike ? accentSet.base : (accentSet.base + '40'),
                  alignItems: 'center',
                  opacity: selectedReciprocalPet && !isSubmittingLike ? 1 : 0.5 }}>
                {isSubmittingLike ? (
                  <ActivityIndicator color="#fff" size="small" />
                ) : (
                  <Text style={{ fontSize: 15, fontWeight: 600, color: '#fff' }}>Confirm Match</Text>
                )}
              </TouchableOpacity>
            </View>
          </View>
        </View>
      )}
    </>
  );
}
