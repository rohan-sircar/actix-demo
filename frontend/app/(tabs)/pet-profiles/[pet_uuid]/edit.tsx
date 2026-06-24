import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  ActivityIndicator,
  TouchableOpacity,
  ScrollView,
  TextInput,
  Alert,
} from 'react-native';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { Ionicons } from '@expo/vector-icons';
import { useRouter, useLocalSearchParams } from 'expo-router';

import api from '~/app/lib/api';
import { petApi } from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import type { Pet } from '~/app/models/pets';
import * as Style from '~/app/styles/Styles';

export default function EditPetScreen() {
  const router = useRouter();
  const { pet_uuid } = useLocalSearchParams<{ pet_uuid: string }>();
  const queryClient = useQueryClient();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const secondaryColor = colors.grey;

  const { data: pet, isLoading } = useQuery({
    queryKey: ['pet', pet_uuid],
    queryFn: async () => {
      const res = await api.get<Pet>(`/api/v1/private/user/pets/${pet_uuid}`);
      return res.data;
    },
  });

  const updatePetMutation = useMutation({
    mutationFn: async (data: {
      name: string;
      species: string;
      breed?: string | null;
      date_of_birth?: string | null;
      gender?: string | null;
      weight?: number | null;
      color_markings?: string | null;
      description?: string | null;
    }) => {
      return await petApi.updatePet(pet_uuid, data);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['pet', pet_uuid] });
      queryClient.invalidateQueries({ queryKey: ['pets'] });
      router.back();
    },
    onError: (error: any) => {
      const message = error.response?.data?.message || 'Failed to update pet. Please try again.';
      Alert.alert('Error', message);
    },
  });

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

  useEffect(() => {
    if (pet) {
      setFormData({
        name: pet.name || '',
        species: pet.species || '',
        breed: pet.breed || '',
        date_of_birth: pet.date_of_birth || '',
        gender: pet.gender || '',
        weight: pet.weight?.toString() || '',
        color_markings: pet.color_markings || '',
        description: pet.description || '',
      });
    }
  }, [pet]);

  const handleSave = async () => {
    if (!formData.name.trim() || !formData.species.trim()) {
      setFormError('Name and species are required.');
      return;
    }
    setFormError('');

    const payload: {
      name: string;
      species: string;
      breed?: string | null;
      date_of_birth?: string | null;
      gender?: string | null;
      weight?: number | null;
      color_markings?: string | null;
      description?: string | null;
    } = {
      name: formData.name.trim(),
      species: formData.species.trim(),
    };

    if (formData.breed.trim()) {
      payload.breed = formData.breed.trim();
    } else {
      payload.breed = null;
    }

    if (formData.date_of_birth.trim()) {
      payload.date_of_birth = formData.date_of_birth.trim();
    } else {
      payload.date_of_birth = null;
    }

    if (formData.gender.trim()) {
      payload.gender = formData.gender.trim();
    } else {
      payload.gender = null;
    }

    if (formData.weight.trim()) {
      payload.weight = parseFloat(formData.weight);
    } else {
      payload.weight = null;
    }

    if (formData.color_markings.trim()) {
      payload.color_markings = formData.color_markings.trim();
    } else {
      payload.color_markings = null;
    }

    if (formData.description.trim()) {
      payload.description = formData.description.trim();
    } else {
      payload.description = null;
    }

    await updatePetMutation.mutateAsync(payload);
  };

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <ActivityIndicator size="large" color={accentSet.base} />
      </View>
    );
  }

  return (
    <ScrollView className="flex-1 px-4 pt-2" style={{ backgroundColor: colors.background }} contentContainerStyle={{ paddingBottom: 30 }} keyboardShouldPersistTaps="handled">
      <View className="mb-4 rounded-2xl p-5" style={{ backgroundColor: colors.card }}>
        <Text className="mb-4 text-lg font-bold" style={{ color: colors.text }}>
          Edit Pet Details
        </Text>

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
            onPress={handleSave}
            disabled={updatePetMutation.isPending}
            className="flex-1 items-center rounded-xl py-3"
            style={{ backgroundColor: accentSet.base, opacity: updatePetMutation.isPending ? 0.7 : 1 }}>
            <Text className="font-semibold text-white">
              {updatePetMutation.isPending ? 'Saving...' : 'Save Changes'}
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => router.back()}
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
}
