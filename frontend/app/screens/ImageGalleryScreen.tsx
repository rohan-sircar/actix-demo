import React, { useCallback, useState } from 'react';
import {
  ActivityIndicator,
  View,
  Text,
  TouchableOpacity,
  Alert,
  Modal,
  Pressable,
} from 'react-native';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useNavigation, useRoute } from '@react-navigation/native';
import type { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { Ionicons } from '@expo/vector-icons';
import { FlashList } from '@shopify/flash-list';
import * as ImagePicker from 'expo-image-picker';

import api from '~/app/lib/api';
import { petImageApi } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { PetImage } from '~/app/models/pets';
import { TabStackParamList } from '~/types/navigation';
import { useFocusEffect } from '@react-navigation/native';

const API_BASE = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800/api/v1';

const getImageUrl = (imageUuid: string, variant: 'thumbnail' | 'medium' | 'original' = 'medium'): string => {
  return `${API_BASE}/pets/images/${imageUuid}/${variant}`;
};

type ImageGalleryScreenRoute = {
  params: { pet_uuid: string };
};

export default function ImageGalleryScreen() {
  const route = useRoute<any>();
  const navigation = useNavigation<NativeStackNavigationProp<TabStackParamList>>();
  const { pet_uuid } = route.params;
  const queryClient = useQueryClient();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);

  const [editMode, setEditMode] = useState(false);
  const [selectedImage, setSelectedImage] = useState<PetImage | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const { data: images, isLoading } = useQuery({
    queryKey: ['pet-images', pet_uuid],
    queryFn: async () => {
      const res = await api.get<PetImage[]>(`/api/v1/private/user/pets/${pet_uuid}/images`);
      return res.data.sort((a, b) => a.sort_order - b.sort_order);
    },
  });

  const setPrimaryMutation = useMutation({
    mutationFn: (imageUuid: string) => petImageApi.setPrimary(pet_uuid, imageUuid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pet-images', pet_uuid] });
      queryClient.invalidateQueries({ queryKey: ['pet', pet_uuid] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (imageUuid: string) => petImageApi.remove(pet_uuid, imageUuid),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pet-images', pet_uuid] });
      queryClient.invalidateQueries({ queryKey: ['pet', pet_uuid] });
    },
  });

  useFocusEffect(
    useCallback(() => {
      queryClient.invalidateQueries({ queryKey: ['pet-images', pet_uuid] });
    }, [queryClient, pet_uuid])
  );

  const handlePickImage = async () => {
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [1, 1],
      quality: 0.8,
    });

    if (!result.canceled && result.assets[0]) {
      try {
        const uri = result.assets[0].uri;
        const mimeType = result.assets[0].type || 'image/jpeg';
        console.log('[ImageGallery] Uploading image:', uri, mimeType);
        await petImageApi.upload(pet_uuid, uri, mimeType);
        queryClient.invalidateQueries({ queryKey: ['pet-images', pet_uuid] });
      } catch (err) {
        console.error('[ImageGallery] Image upload failed:', err);
        Alert.alert('Error', 'Failed to upload image.');
      }
    }
  };

  const handleSetPrimary = (image: PetImage) => {
    if (editMode) return;
    setPrimaryMutation.mutate(image.uuid);
    setSelectedImage(null);
  };

  const handleDelete = () => {
    if (selectedImage) {
      setShowDeleteConfirm(true);
    }
  };

  const confirmDelete = () => {
    if (selectedImage) {
      deleteMutation.mutate(selectedImage.uuid);
      setShowDeleteConfirm(false);
      setSelectedImage(null);
    }
  };

  const handleReorder = (fromIndex: number, toIndex: number) => {
    if (!images) return;
    const updated = [...images];
    const [moved] = updated.splice(fromIndex, 1);
    updated.splice(toIndex, 0, moved);
    const reordered = updated.map((img, idx) => ({ ...img, sort_order: idx }));
    queryClient.setQueryData(['pet-images', pet_uuid], reordered);
  };

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center">
        <ActivityIndicator size="large" color={accentSet.base} />
      </View>
    );
  }

  return (
    <View className="flex-1">
      <FlashList
        data={images}
        renderItem={({ item, index }) => (
          <TouchableOpacity
            onPress={() => setSelectedImage(selectedImage?.uuid === item.uuid ? null : item)}
            className="relative m-1.5 h-[100px] w-[calc(33.33%-8px)] overflow-hidden rounded-xl"
            style={{ borderWidth: item.is_primary && !editMode ? 2 : 0, borderColor: accentSet.base }}>
            <Ionicons
              name={getImageUrl(item.uuid).includes('thumbnail') ? 'image' : 'image'}
              size={24}
              color="#999"
            />
            {item.is_primary && !editMode && (
              <View className="absolute right-1 top-1 rounded-full px-1.5 py-0.5" style={{ backgroundColor: accentSet.base }}>
                <Ionicons name="checkmark" size={12} color="#fff" />
              </View>
            )}
            {editMode && (
              <>
                <TouchableOpacity
                  onPress={() => {
                    setSelectedImage(item);
                    setShowDeleteConfirm(true);
                  }}
                  className="absolute bottom-1 right-1 rounded-full p-1"
                  style={{ backgroundColor: 'rgba(0,0,0,0.5)' }}>
                  <Ionicons name="trash" size={14} color="#fff" />
                </TouchableOpacity>
                <View className="absolute left-1 top-1 rounded-full bg-black/50 p-1">
                  <Ionicons name="menu" size={14} color="#fff" />
                </View>
              </>
            )}
          </TouchableOpacity>
        )}
        keyExtractor={(item) => item.uuid}
        numColumns={3}
        estimatedItemSize={110}
        contentContainerStyle={{ paddingHorizontal: 4, paddingTop: 8, paddingBottom: 30 }}
        ListHeaderComponent={
          <View className="px-4 pt-2">
            <View className="mb-3 flex-row items-center justify-between">
              <TouchableOpacity
                onPress={() => setEditMode(!editMode)}
                className="flex-row items-center gap-1"
                style={{ backgroundColor: editMode ? accentSet.base : accentSet.bgSubtle, paddingHorizontal: 10, paddingVertical: 5, borderRadius: 6 }}>
                <Ionicons name={editMode ? 'checkmark' : 'shuffle'} size={16} color={editMode ? '#fff' : accentSet.base} />
                <Text className="text-sm font-semibold" style={{ color: editMode ? '#fff' : accentSet.base }}>
                  {editMode ? 'Done' : 'Edit'}
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={handlePickImage}
                className="flex-row items-center gap-1"
                style={{ backgroundColor: accentSet.bgSubtle, paddingHorizontal: 10, paddingVertical: 5, borderRadius: 6 }}>
                <Ionicons name="add" size={16} color={accentSet.base} />
                <Text className="text-sm font-semibold" style={{ color: accentSet.base }}>
                  Add
                </Text>
              </TouchableOpacity>
            </View>
          </View>
        }
      />

      <Modal visible={showDeleteConfirm} transparent animationType="fade">
        <Pressable className="flex-1 items-center justify-center bg-black/50" onPress={() => setShowDeleteConfirm(false)}>
          <View className="w-[80%] rounded-2xl p-6" style={{ backgroundColor: colors.card }}>
            <Text className="mb-2 text-lg font-bold" style={{ color: colors.text }}>
              Delete Photo?
            </Text>
            <Text className="mb-6 text-sm" style={{ color: colors.grey }}>
              This photo will be permanently removed.
            </Text>
            <View className="flex-row gap-3">
              <TouchableOpacity
                onPress={() => setShowDeleteConfirm(false)}
                className="flex-1 items-center rounded-xl py-3"
                style={{ backgroundColor: isDarkColorScheme ? '#3d2a22' : '#f0e0d8' }}>
                <Text className="font-semibold" style={{ color: colors.text }}>
                  Cancel
                </Text>
              </TouchableOpacity>
              <TouchableOpacity
                onPress={confirmDelete}
                className="flex-1 items-center rounded-xl py-3 bg-rose-500">
                <Text className="font-semibold text-white">
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
