import { Ionicons } from '@expo/vector-icons';
import React from 'react';
import { useState } from 'react';
import { Text, TextInput, TouchableOpacity, View } from 'react-native';

import Avatar from '../components/Avatar';
import StatTile from '../components/StatTile';
import * as Style from '~/app/styles/Styles';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import api from '~/app/lib/api';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Pet } from '~/app/models/pets';

export default function ProfileScreen() {
  const queryClient = useQueryClient();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [isEditing, setIsEditing] = useState(false);
  const [displayName, setDisplayName] = useState('');
  const [bio, setBio] = useState('');
  const [location, setLocation] = useState('');
  const [website, setWebsite] = useState('');

  const { data: user, isLoading } = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const res = await api.get<UserResponse>('/api/v1/private/user');
      return res.data;
    },
  });

  const { data: pets } = useQuery({
    queryKey: ['pets-count'],
    queryFn: async () => {
      const res = await api.get<Pet[]>('/api/v1/private/user/pets');
      return res.data;
    },
  });

  const updateProfileMutation = useMutation({
    mutationFn: async (data: {
      display_name?: string;
      bio?: string;
      location?: string;
      website?: string;
    }) => {
      await api.patch('/api/v1/private/user/profile', data);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user'] });
      setIsEditing(false);
    },
  });

  if (isLoading) {
    return (
      <View className="w-full flex-1 items-center justify-center">
        <Text style={{ color: colors.grey }}>Loading profile...</Text>
      </View>
    );
  }

  if (!user) return null;

  const profile = user.profile || {};

  const handleSave = () => {
    const updates: Record<string, string> = {};
    if (displayName) updates.display_name = displayName;
    if (bio) updates.bio = bio;
    if (location) updates.location = location;
    if (website) updates.website = website;
    if (Object.keys(updates).length > 0) {
      updateProfileMutation.mutate(updates);
    } else {
      setIsEditing(false);
    }
  };

  return (
    <View className="w-full flex-1">
      <View className="w-full flex-1 px-4 pb-4 pt-2">
        <View className="mb-4">
          <Text className="mb-4 text-2xl font-bold" style={{ color: colors.text }}>
            My Profile
          </Text>

          <View className="flex-row items-center">
            <Avatar userId={user.id} size={64} style={{ marginRight: 16 }} />
            <View className="flex-1">
              <Text className="text-lg font-bold" style={{ color: colors.text }}>
                {profile.display_name || user.username}
              </Text>
              <Text className="text-sm" style={{ color: colors.grey }}>
                @{user.username}
              </Text>
            </View>
          </View>
        </View>

        <View className="mb-4">
          {isEditing ? (
            <View className="gap-3">
              <TextInput
                placeholder="Display Name"
                value={displayName || profile.display_name || ''}
                onChangeText={setDisplayName}
                className="h-11 rounded-xl border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <TextInput
                placeholder="Bio"
                value={bio || profile.bio || ''}
                onChangeText={setBio}
                multiline
                className="min-h-[60px] rounded-xl border px-3 py-2"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <TextInput
                placeholder="Location"
                value={location || profile.location || ''}
                onChangeText={setLocation}
                className="h-11 rounded-xl border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <TextInput
                placeholder="Website"
                value={website || profile.website || ''}
                onChangeText={setWebsite}
                className="h-11 rounded-xl border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <View className="mt-2 flex-row gap-3">
                <TouchableOpacity
                  onPress={handleSave}
                  className="flex-1 items-center rounded-xl py-3"
                  style={{ backgroundColor: accentSet.base }}>
                  <Text className="font-semibold text-white">Save Changes</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  onPress={() => setIsEditing(false)}
                  className="flex-1 items-center rounded-xl py-3"
                  style={{ backgroundColor: isDarkColorScheme ? '#3d2a22' : '#f0e0d8' }}>
                  <Text className="font-semibold" style={{ color: colors.text }}>
                    Cancel
                  </Text>
                </TouchableOpacity>
              </View>
            </View>
          ) : (
            <TouchableOpacity
              onPress={() => setIsEditing(true)}
              className="items-center rounded-xl px-4 py-3"
              style={{ backgroundColor: accentSet.bgSubtle }}>
              <Text className="font-semibold" style={{ color: accentSet.base }}>
                Edit Profile
              </Text>
            </TouchableOpacity>
          )}
        </View>

        {profile.bio ? (
          <>
            <Text className="mb-1.5 font-semibold" style={{ color: colors.text }}>
              About me
            </Text>
            <Text className="mb-3 text-sm leading-relaxed" style={{ color: colors.grey }}>
              {profile.bio}
            </Text>
          </>
        ) : null}

        {(profile.location || profile.website) && (
          <View className="mb-4 flex-row flex-wrap gap-4">
            {profile.location && (
              <Text className="text-sm" style={{ color: colors.grey }}>
                <Ionicons name="location" size={16} color={colors.grey} /> {profile.location}
              </Text>
            )}
            {profile.website && (
              <Text className="text-sm" style={{ color: colors.grey }}>
                <Ionicons name="globe" size={16} color={colors.grey} /> {profile.website}
              </Text>
            )}
          </View>
        )}

        <View
          className="mt-2 flex-row justify-between rounded-xl border-x border-b border-t px-4 py-5"
          style={{ borderColor: colors.grey5 }}>
          <StatTile title="Pets" value={String(pets?.length ?? 0)} />
        </View>
      </View>
    </View>
  );
}
