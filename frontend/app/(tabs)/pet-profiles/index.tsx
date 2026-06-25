import { useFocusEffect, useRouter } from 'expo-router';
import React, { useCallback, useEffect, useState } from 'react';
import { useAuthStore } from '../../stores/AuthStore';
import {
  View,
  Text,
  ActivityIndicator,
  TouchableOpacity,
  ScrollView,
  TextInput,
  Image,
  Alert,
  Platform,
} from 'react-native';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useActionSheet } from '@expo/react-native-action-sheet';
import * as ImagePicker from 'expo-image-picker';
import * as FileSystem from 'expo-file-system/legacy';
import { petImageApi } from '~/app/lib/api';
import api from '~/app/lib/api';
import PrivatePetCard from '~/app/components/PrivatePetCard';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { Pet } from '~/app/models/pets';
import { Ionicons } from '@expo/vector-icons';
import * as Style from '~/app/styles/Styles';

const guessMimeTypeFromUri = (uri: string): string => {
  const lower = uri.toLowerCase();
  if (lower.includes('.png')) return 'image/png';
  if (lower.includes('.webp')) return 'image/webp';
  if (lower.includes('.gif')) return 'image/gif';
  return 'image/jpeg';
};

const Home = () => {
  const router = useRouter();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  useEffect(() => {
    if (!isAuthenticated) {
      router.replace('/login');
    }
  }, [isAuthenticated, router]);
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const queryClient = useQueryClient();
  const { showActionSheetWithOptions } = useActionSheet();
  const [showAddForm, setShowAddForm] = useState(false);
  const [pendingDelete, setPendingDelete] = useState<{ pet_uuid: string; petName: string } | null>(null);

  const {
    data: pets,
    isLoading,
    error: queryError,
    refetch,
  } = useQuery({
    queryKey: ['pets'],
    queryFn: async () => {
      const res = await api.get<Pet[]>('/api/v1/private/user/pets');
      return res.data;
    },
  });

  const createPetMutation = useMutation({
    mutationFn: async (data: {
      name: string;
      species: string;
      breed?: string;
      date_of_birth?: string;
      gender?: string;
      weight?: number;
      color_markings?: string;
      description?: string;
    }) => {
      const res = await api.post('/api/v1/private/user/pets', data);
      return res.data;
    },
    onSuccess: () => {
      refetch();
      queryClient.invalidateQueries({ queryKey: ['pets-count'] });
      setShowAddForm(false);
      setFormData({
        name: '',
        species: '',
        breed: '',
        date_of_birth: '',
        gender: '',
        weight: '',
        color_markings: '',
        description: '',
      });
      setFormError('');
      setSelectedImage(null);
    },
  });

  const deletePetMutation = useMutation({
    mutationFn: async (pet_uuid: string) => {
      await api.delete(`/api/v1/private/user/pets/${pet_uuid}`);
    },
    onSuccess: () => {
      refetch();
      queryClient.invalidateQueries({ queryKey: ['pets-count'] });
      setPendingDelete(null);
    },
    onError: () => {
      Alert.alert('Error', 'Failed to delete pet. Please try again.');
    },
  });

  const handleDelete = (pet_uuid: string, petName: string) => {
    setPendingDelete({ pet_uuid, petName });
  };

  const handleActionSheetPress = (buttonIndex: number | undefined) => {
    if (buttonIndex === 1 && pendingDelete) {
      deletePetMutation.mutate(pendingDelete.pet_uuid);
    }
    setPendingDelete(null);
  };

  React.useEffect(() => {
    if (pendingDelete) {
      showActionSheetWithOptions(
        {
          options: ['Cancel', 'Delete'],
          destructiveButtonIndex: 1,
          cancelButtonIndex: 0,
        },
        handleActionSheetPress
      );
    }
  }, [pendingDelete, showActionSheetWithOptions]);

  useFocusEffect(
    useCallback(() => {
      refetch();
    }, [refetch])
  );

  const secondaryColor = colors.grey;

  const [formData, setFormData] = useState({
    name: '',
    species: '',
    breed: '',
    date_of_birth: '',
    gender: '',
    weight: '',
    color_markings: '',
    description: '',
  });
  const [formError, setFormError] = useState('');
  const [selectedImage, setSelectedImage] = useState<{ uri: string; mimeType: string } | null>(null);

  const handleCreatePet = async () => {
    if (!formData.name.trim() || !formData.species.trim()) {
      setFormError('Name and species are required.');
      return;
    }
    setFormError('');

    let petUuid: string | undefined;

    try {
      const result = await createPetMutation.mutateAsync({
        name: formData.name.trim(),
        species: formData.species.trim(),
        ...(formData.breed.trim() && { breed: formData.breed.trim() }),
        ...(formData.date_of_birth.trim() && { date_of_birth: formData.date_of_birth.trim() }),
        ...(formData.gender && { gender: formData.gender }),
        ...(formData.weight.trim() && { weight: parseFloat(formData.weight) }),
        ...(formData.color_markings.trim() && { color_markings: formData.color_markings.trim() }),
        ...(formData.description.trim() && { description: formData.description.trim() }),
      });
      petUuid = result.pet_uuid;
    } catch (err: any) {
      if (err.response?.data?.message) {
        setFormError(err.response.data.message);
      } else {
        setFormError('Failed to create pet. Please try again.');
      }
      return;
    }

    if (selectedImage && petUuid) {
      try {
        await petImageApi.upload(petUuid, selectedImage.uri, selectedImage.mimeType);
      } catch {
        // Image upload failed but pet was created successfully — non-critical
      }
    }
  };

  const handlePickImage = async () => {
    const result = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [1, 1],
      quality: 0.8,
    });

    if (!result.canceled && result.assets[0]) {
      const asset = result.assets[0];
      const uri = asset.uri;
      try {
        let dataUrl: string;
        let mimeType: string;
        if (Platform.OS === 'web') {
          const response = await fetch(uri);
          const blob = await response.blob();
          mimeType = blob.type || 'image/jpeg';
          dataUrl = await new Promise<string>((resolve) => {
            const reader = new FileReader();
            reader.onloadend = () => resolve(reader.result as string);
            reader.readAsDataURL(blob);
          });
        } else {
          mimeType = asset.type || guessMimeTypeFromUri(uri);
          const base64 = await FileSystem.readAsStringAsync(uri, { encoding: 'base64' });
          dataUrl = `data:${mimeType};base64,${base64}`;
        }
        setSelectedImage({ uri: dataUrl, mimeType });
      } catch (err) {
        console.error('Failed to load image:', err);
        setSelectedImage({ uri, mimeType: 'image/jpeg' });
      }
    }
  };

  const renderForm = () => (
    <ScrollView
      className="flex-1 px-4 pt-2"
      style={{ backgroundColor: colors.background }}
      contentContainerStyle={{ paddingBottom: 30 }}
      keyboardShouldPersistTaps="handled">
      <View className="mb-4 rounded-2xl p-5" style={{ backgroundColor: colors.card }}>
        <Text className="mb-4 text-lg font-bold" style={{ color: colors.text }}>
          Pet Details
        </Text>

        <TouchableOpacity
          onPress={handlePickImage}
          className="mb-4 items-center justify-center rounded-2xl border-2 border-dashed"
          style={{ height: 140, borderColor: selectedImage ? accentSet.base : secondaryColor }}>
          {selectedImage ? (
            <Image source={{ uri: selectedImage.uri }} className="h-full w-full rounded-2xl" resizeMode="cover" />
          ) : (
            <>
              <Ionicons name="camera" size={28} color={secondaryColor} />
              <Text className="mt-2 text-sm font-medium" style={{ color: secondaryColor }}>
                Add Photo
              </Text>
            </>
          )}
        </TouchableOpacity>

        {formError ? (
          <View className="mb-4 rounded-xl bg-rose-500/15 p-3">
            <Text className="text-center text-sm font-medium text-rose-500">{formError}</Text>
          </View>
        ) : null}

        <View className="gap-4">
          <View>
            <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
              Name *
            </Text>
            <TextInput
              placeholder="e.g. Buddy"
              value={formData.name}
              onChangeText={(v) => setFormData({ ...formData, name: v })}
              className="h-11 rounded-xl px-4 text-base"
              style={Style.inputStyle(isDarkColorScheme, accentSet)}
              placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
            />
          </View>

          <View>
            <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
              Species *
            </Text>
            <TextInput
              placeholder="e.g. dog, cat, bird"
              value={formData.species}
              onChangeText={(v) => setFormData({ ...formData, species: v })}
              className="h-11 rounded-xl px-4 text-base"
              style={Style.inputStyle(isDarkColorScheme, accentSet)}
              placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
            />
          </View>

          <View>
            <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
              Breed
            </Text>
            <TextInput
              placeholder="e.g. Golden Retriever"
              value={formData.breed}
              onChangeText={(v) => setFormData({ ...formData, breed: v })}
              className="h-11 rounded-xl px-4 text-base"
              style={Style.inputStyle(isDarkColorScheme, accentSet)}
              placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
            />
          </View>

          <View className="flex-row gap-3">
            <View className="flex-1">
              <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
                Gender
              </Text>
              <TextInput
                placeholder="male / female"
                value={formData.gender}
                onChangeText={(v) => setFormData({ ...formData, gender: v })}
                className="h-11 rounded-xl px-4 text-base"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
            </View>
            <View className="flex-1">
              <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
                Weight (kg)
              </Text>
              <TextInput
                placeholder="30.5"
                value={formData.weight}
                onChangeText={(v) => setFormData({ ...formData, weight: v })}
                keyboardType="decimal-pad"
                className="h-11 rounded-xl px-4 text-base"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
            </View>
          </View>

          <View>
            <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
              Date of Birth
            </Text>
            <TextInput
              placeholder="YYYY-MM-DD"
              value={formData.date_of_birth}
              onChangeText={(v) => setFormData({ ...formData, date_of_birth: v })}
              className="h-11 rounded-xl px-4 text-base"
              style={Style.inputStyle(isDarkColorScheme, accentSet)}
              placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
            />
          </View>

          <View>
            <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
              Color / Markings
            </Text>
            <TextInput
              placeholder="e.g. Golden with white chest"
              value={formData.color_markings}
              onChangeText={(v) => setFormData({ ...formData, color_markings: v })}
              className="h-11 rounded-xl px-4 text-base"
              style={Style.inputStyle(isDarkColorScheme, accentSet)}
              placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
            />
          </View>

          <View>
            <Text className="mb-1 text-sm font-medium" style={{ color: secondaryColor }}>
              Description
            </Text>
            <TextInput
              placeholder="Tell us about your pet..."
              value={formData.description}
              onChangeText={(v) => setFormData({ ...formData, description: v })}
              multiline
              numberOfLines={3}
              className="h-[80px] rounded-xl px-4 pt-3 text-base"
              style={Style.inputStyle(isDarkColorScheme, accentSet)}
              placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
            />
          </View>
        </View>

        <View className="mt-5 flex-row gap-3">
          <TouchableOpacity
            onPress={handleCreatePet}
            className="flex-1 items-center rounded-xl py-3"
            style={{ backgroundColor: accentSet.base }}>
            <Text className="font-semibold text-white">
              {createPetMutation.isPending ? 'Creating...' : 'Add Pet'}
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => {
              setShowAddForm(false);
              setFormData({
                name: '',
                species: '',
                breed: '',
                date_of_birth: '',
                gender: '',
                weight: '',
                color_markings: '',
                description: '',
              });
              setFormError('');
              setSelectedImage(null);
            }}
            className="flex-1 items-center rounded-xl py-3"
            style={{ backgroundColor: isDarkColorScheme ? '#3d2a22' : '#f0e0d8' }}>
            <Text className="font-semibold" style={{ color: colors.text }}>
              Cancel
            </Text>
          </TouchableOpacity>
        </View>
      </View>
    </ScrollView>
  );

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <ActivityIndicator size="large" color={accentSet.base} />
        <Text className="mt-4 text-sm font-medium" style={{ color: secondaryColor }}>
          Fetching your furry friends...
        </Text>
      </View>
    );
  }

  if (queryError && !showAddForm) {
    return (
      <View className="flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <Text className="text-center text-sm font-medium text-rose-500">
          Failed to load pets. Please try again.
        </Text>
      </View>
    );
  }

  if (showAddForm) {
    return renderForm();
  }

  return (
    <View className="flex-1" style={{ backgroundColor: colors.background }}>
      <View className="px-4 pb-2 pt-4">
        <View className="flex-row items-center justify-between">
          <View>
            <Text className="text-2xl font-bold" style={{ color: colors.text }}>
              My Pets 🐾
            </Text>
            <Text className="mt-0.5 text-sm" style={{ color: secondaryColor }}>
              Your furry family members
            </Text>
          </View>
          <TouchableOpacity
            onPress={() => setShowAddForm(true)}
            className="items-center justify-center rounded-full"
            style={{ width: 40, height: 40, backgroundColor: isDarkColorScheme ? colors.grey5 : accentSet.bgSubtle }}>
            <Ionicons name="add" size={20} color={accentSet.base} />
          </TouchableOpacity>
        </View>
      </View>

      {pets && pets.length > 0 ? (
        <ScrollView
          contentContainerStyle={{
            paddingHorizontal: 16,
            paddingTop: 8,
            paddingBottom: 30,
            flexDirection: 'row',
            flexWrap: 'wrap',
            gap: 12,
            justifyContent: 'flex-start',
          }}>
          {pets.map((item) => (
            <PrivatePetCard
              key={item.pet_uuid}
              pet_uuid={item.pet_uuid}
              name={item.name}
              species={item.species}
              breed={item.breed}
              primary_image={item.primary_image}
              onDelete={(uuid) => handleDelete(uuid, item.name)}
            />
          ))}
        </ScrollView>
      ) : (
        <View className="flex-1 items-center justify-center px-8">
          <View
            className="mb-4 items-center justify-center rounded-full"
            style={{ width: 64, height: 64, backgroundColor: isDarkColorScheme ? colors.grey5 : accentSet.bgSubtle }}>
            <Ionicons name="paw" size={32} color={accentSet.base} />
          </View>
          <Text className="mb-1 text-center text-base font-semibold" style={{ color: colors.text }}>
            No pets yet!
          </Text>
          <Text className="text-center text-sm" style={{ color: secondaryColor }}>
            Add your first pet to get started
          </Text>
        </View>
      )}


    </View>
  );
};

export default Home;
