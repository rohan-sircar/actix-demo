import React, { useCallback, useState, useEffect, useRef } from 'react';
import {
  ActivityIndicator,
  View,
  Text,
  TouchableOpacity,
  Alert,
  Modal,
  Pressable,
  Image as RNImage,
  ScrollView,
} from 'react-native';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRouter, useLocalSearchParams, useFocusEffect } from 'expo-router';
import { Ionicons } from '@expo/vector-icons';
import * as ImagePicker from 'expo-image-picker';

import api from '~/app/lib/api';
import { petImageApi } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { PetImage } from '~/app/models/pets';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';
const MAX_IMAGES = 10;
const GAP = 12;
const COLUMNS = 3;

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

const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'medium'): string => {
  return `${API_BASE}/api/v1/pets/images/${imageUuid}/${variant}`;
};

export default function ImageGalleryScreen() {
  const router = useRouter();
  const { pet_uuid } = useLocalSearchParams<{ pet_uuid: string }>();
  const queryClient = useQueryClient();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const [editMode, setEditMode] = useState(false);
  const [deletingImage, setDeletingImage] = useState<PetImage | null>(null);
  const [previewImage, setPreviewImage] = useState<PetImage | null>(null);
  const [images, setImages] = useState<PetImage[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isUploading, setIsUploading] = useState(false);
  const contentRef = useRef<View>(null);
  const [layoutWidth, setLayoutWidth] = useState(0);

  const imageCount = images.length;
  const padding = 12;
  const availableWidth = Math.max(layoutWidth, 300);
  const cellSize = (availableWidth - padding * 2 - GAP * (COLUMNS - 1)) / COLUMNS;

  const fetchImages = async () => {
    try {
      const res = await api.get<PetImage[]>(`/api/v1/private/user/pets/${pet_uuid}/images`);
      setImages(res.data.sort((a, b) => a.sort_order - b.sort_order));
    } catch (err) {
      console.error('[Gallery] Failed to fetch images:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const setPrimaryMutation = useMutation({
    mutationFn: (imageUuid: string) => petImageApi.setPrimary(pet_uuid, imageUuid),
    onSuccess: () => {
      fetchImages();
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (imageUuid: string) => petImageApi.remove(pet_uuid, imageUuid),
    onSuccess: () => {
      fetchImages();
    },
  });

  useEffect(() => {
    fetchImages();
  }, [pet_uuid]);

  useFocusEffect(
    useCallback(() => {
      fetchImages();
    }, [pet_uuid])
  );

  const handlePickImage = async () => {
    if (imageCount >= MAX_IMAGES) {
      Alert.alert('Limit Reached', `You can upload up to ${MAX_IMAGES} images.`);
      return;
    }
    let result;
    try {
      const pickerPromise = ImagePicker.launchImageLibraryAsync({
        mediaTypes: ImagePicker.MediaTypeOptions.Images,
        allowsEditing: false,
        quality: 0.8,
      });
      result = await pickerPromise;
    } catch (err: any) {
      Alert.alert('Error', `Picker failed: ${err?.message || err}`);
      return;
    }

    if (!result.canceled && result.assets[0]) {
      setIsUploading(true);
      try {
        const uri = result.assets[0].uri;
        const mimeType = await detectMimeTypeFromUri(uri, result.assets[0].type ?? undefined);
        await petImageApi.upload(pet_uuid, uri, mimeType);
        fetchImages();
      } catch (err) {
        console.error('[ImageGallery] Image upload failed:', err);
        Alert.alert('Error', 'Failed to upload image.');
      } finally {
        setIsUploading(false);
      }
    }
  };

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center">
        <ActivityIndicator size="large" color={accentSet.base} />
      </View>
    );
  }

  const imageRows: PetImage[][] = [];
  (images ?? []).forEach((img, i) => {
    const rowIdx = Math.floor(i / COLUMNS);
    if (!imageRows[rowIdx]) imageRows[rowIdx] = [];
    imageRows[rowIdx].push(img);
  });

  return (
    <View className="flex-1">
      <ScrollView contentContainerStyle={{ alignItems: 'center', paddingVertical: padding, paddingBottom: 40 }}>
        {/* Toolbar */}
        <View style={{ flexDirection: 'row', alignItems: 'center', justifyContent: 'center', gap: 8, marginBottom: 12 }}>
          <TouchableOpacity
            onPress={() => setEditMode(!editMode)}
            style={{
              flexDirection: 'row',
              alignItems: 'center',
              gap: 4,
              backgroundColor: editMode ? accentSet.base : accentSet.bgSubtle,
              paddingHorizontal: 10,
              paddingVertical: 6,
              borderRadius: 6,
            }}>
            <Ionicons name={editMode ? 'checkmark' : 'pencil'} size={16} color={editMode ? '#fff' : accentSet.base} />
            <Text style={{ fontSize: 13, fontWeight: '600', color: editMode ? '#fff' : accentSet.base }}>
              {editMode ? 'Done' : 'Edit'}
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={handlePickImage}
            disabled={isUploading}
            style={{
              flexDirection: 'row',
              alignItems: 'center',
              gap: 4,
              backgroundColor: accentSet.bgSubtle,
              paddingHorizontal: 10,
              paddingVertical: 6,
              borderRadius: 6,
              opacity: isUploading ? 0.5 : 1,
            }}>
            {isUploading ? (
              <ActivityIndicator size="small" color={accentSet.base} />
            ) : (
              <Ionicons name="add" size={16} color={accentSet.base} />
            )}
            <Text style={{ fontSize: 13, fontWeight: '600', color: accentSet.base }}>
              {isUploading ? 'Uploading...' : `Add (${imageCount}/${MAX_IMAGES})`}
            </Text>
          </TouchableOpacity>
        </View>

        {/* Grid */}
        <View ref={contentRef} onLayout={(e) => setLayoutWidth(e.nativeEvent.layout.width)} style={{ width: '100%', paddingHorizontal: padding }}>
          {imageRows.length === 0 ? (
          <View style={{ alignItems: 'center', paddingVertical: 60 }}>
            <Ionicons name="images-outline" size={48} color={colors.grey} />
            <Text style={{ marginTop: 12, fontSize: 15, fontWeight: '500', color: colors.grey }}>
              No photos yet
            </Text>
            <Text style={{ marginTop: 4, fontSize: 13, color: colors.grey }}>
              Tap + Add to upload photos
            </Text>
          </View>
        ) : (
          imageRows.map((row, rowIdx) => (
            <View key={rowIdx} style={{ flexDirection: 'row', marginBottom: GAP, gap: GAP }}>
              {row.map((image) => (
                <TouchableOpacity
                  key={image.uuid}
                  onPress={() => {
                    if (!editMode) {
                      setPreviewImage(image);
                    }
                  }}
                  activeOpacity={0.7}
                  style={{
                    width: cellSize,
                    height: cellSize,
                    borderRadius: 12,
                    overflow: 'hidden',
                    borderWidth: image.is_primary && !editMode ? 2 : 0,
                    borderColor: image.is_primary && !editMode ? accentSet.base : 'transparent',
                    backgroundColor: colors.grey + '33',
                  }}>
                    <RNImage
                      source={{ uri: getImageUrl(image.uuid, 'medium') }}
                      style={{ width: '100%', height: '100%' }}
                      resizeMode="cover"
                    />
                  {image.is_primary && !editMode && (
                    <View
                      style={{
                        position: 'absolute',
                        top: 4,
                        left: 4,
                        backgroundColor: accentSet.base,
                        borderRadius: 10,
                        paddingHorizontal: 6,
                        paddingVertical: 2,
                        alignItems: 'center',
                      }}>
                      <Ionicons name="checkmark" size={12} color="#fff" />
                    </View>
                  )}
                  {editMode && !image.is_primary && (
                    <TouchableOpacity
                      onPress={(e) => {
                        e.stopPropagation();
                        setDeletingImage(image);
                      }}
                      style={{
                        position: 'absolute',
                        bottom: 4,
                        right: 4,
                        backgroundColor: 'rgba(0,0,0,0.6)',
                        borderRadius: 12,
                        padding: 4,
                      }}>
                      <Ionicons name="trash" size={14} color="#fff" />
                    </TouchableOpacity>
                  )}
                </TouchableOpacity>
              ))}
            </View>
          ))
        )}
        </View>
      </ScrollView>

      {/* Preview Modal */}
      <Modal visible={previewImage !== null} transparent animationType="fade">
        <Pressable className="flex-1 items-center justify-center bg-black/80" onPress={() => setPreviewImage(null)}>
          <View style={{ width: '85%', maxWidth: 400, borderRadius: 16, overflow: 'hidden', backgroundColor: colors.card }}>
            <RNImage
              source={{ uri: previewImage ? getImageUrl(previewImage.uuid, 'medium') : '' }}
              style={{ width: '100%', aspectRatio: 1 }}
              resizeMode="cover"
            />
            {previewImage && !previewImage.is_primary && !editMode && (
              <TouchableOpacity
                onPress={() => {
                  setPrimaryMutation.mutate(previewImage.uuid);
                  setPreviewImage(null);
                }}
                disabled={setPrimaryMutation.isPending}
                style={{
                  alignItems: 'center',
                  paddingVertical: 14,
                  backgroundColor: setPrimaryMutation.isPending ? accentSet.base + '88' : accentSet.base,
                }}>
                {setPrimaryMutation.isPending ? (
                  <ActivityIndicator size="small" color="#fff" />
                ) : (
                  <Text style={{ fontSize: 14, fontWeight: '600', color: '#fff' }}>
                    Set as Primary Photo
                  </Text>
                )}
              </TouchableOpacity>
            )}
          </View>
        </Pressable>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal visible={deletingImage !== null} transparent animationType="fade">
        <Pressable className="flex-1 items-center justify-center bg-black/50" onPress={() => setDeletingImage(null)}>
          <View style={{ width: '80%', borderRadius: 16, padding: 24, backgroundColor: colors.card }}>
            <Text style={{ fontSize: 17, fontWeight: '700', color: colors.text, marginBottom: 8 }}>
              Delete Photo?
            </Text>
            <Text style={{ fontSize: 13, color: colors.grey, marginBottom: 20 }}>
              This photo will be permanently removed.
            </Text>
            <View style={{ flexDirection: 'row', gap: 12 }}>
              <TouchableOpacity
                onPress={() => setDeletingImage(null)}
                style={{
                  flex: 1,
                  alignItems: 'center',
                  borderRadius: 12,
                  paddingVertical: 12,
                  backgroundColor: isDarkColorScheme ? '#3d2a22' : '#f0e0d8',
                }}>
                <Text style={{ fontWeight: '600', color: colors.text }}>Cancel</Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={() => {
                  if (deletingImage) {
                    deleteMutation.mutate(deletingImage.uuid);
                    setDeletingImage(null);
                  }
                }}
                style={{
                  flex: 1,
                  alignItems: 'center',
                  borderRadius: 12,
                  paddingVertical: 12,
                  backgroundColor: '#f43f5e',
                }}>
                <Text style={{ fontWeight: '600', color: '#fff' }}>
                  {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        </Pressable>
      </Modal>
    </View>
  );
}
