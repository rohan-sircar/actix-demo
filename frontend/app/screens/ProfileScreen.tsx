import { Ionicons } from '@expo/vector-icons';
import React from 'react';
import { useState } from 'react';
import { StyleSheet, Text, TextInput, TouchableOpacity, View } from 'react-native';

import Avatar from '../components/Avatar';
import StatTile from '../components/StatTile';
import * as Style from '~/app/styles/Styles';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import api from '~/app/lib/api';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';

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
      const res = await api.get<UserResponse>('/api/v1/user');
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
      await api.patch('/api/v1/user/profile', data);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['user'] });
      setIsEditing(false);
    },
  });

  if (isLoading) {
    return (
      <View className="w-full flex-1 items-center justify-center">
        <Text>Loading...</Text>
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
    <View className="w-full flex-1 items-center">
      <View className="w-full flex-1 px-4 py-4 md:px-8">
        <View className="mb-1 flex-row items-center">
          <Avatar userId={user.id} style={styles.avatarContainer} size={64} />
          <View>
            <Text className="mb-1 text-lg font-semibold" style={{ color: colors.text }}>
              {profile.display_name || user.username}
            </Text>
            <Text style={{ color: colors.text }}>@{user.username}</Text>
          </View>
        </View>

        <View className="mb-4 pl-20">
          {isEditing ? (
            <View className="gap-3">
              <TextInput
                placeholder="Display Name"
                value={displayName || profile.display_name || ''}
                onChangeText={setDisplayName}
                className="h-10 rounded-lg border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <TextInput
                placeholder="Bio"
                value={bio || profile.bio || ''}
                onChangeText={setBio}
                multiline
                className="min-h-[60px] rounded-lg border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <TextInput
                placeholder="Location"
                value={location || profile.location || ''}
                onChangeText={setLocation}
                className="h-10 rounded-lg border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <TextInput
                placeholder="Website"
                value={website || profile.website || ''}
                onChangeText={setWebsite}
                className="h-10 rounded-lg border px-3"
                style={Style.inputStyle(isDarkColorScheme, accentSet)}
                placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
              />
              <View className="flex-row gap-3">
                <TouchableOpacity onPress={handleSave} className="rounded-lg bg-blue-500 px-4 py-2">
                  <Text className="text-white">Save</Text>
                </TouchableOpacity>
                <TouchableOpacity
                  onPress={() => setIsEditing(false)}
                  className="rounded-lg bg-gray-300 px-4 py-2 dark:bg-gray-700">
                  <Text>Cancel</Text>
                </TouchableOpacity>
              </View>
            </View>
          ) : (
            <TouchableOpacity onPress={() => setIsEditing(true)}>
              <Text className="text-blue-500">Edit Profile</Text>
            </TouchableOpacity>
          )}
        </View>

        {profile.bio ? (
          <>
            <Text className="mb-2 font-medium" style={{ color: colors.text }}>
              About me
            </Text>
            <Text className="pr-3 text-gray-500">{profile.bio}</Text>
          </>
        ) : null}

        {(profile.location || profile.website) && (
          <View className="mt-3 flex-row gap-4">
            {profile.location && (
              <Text style={{ color: colors.text }}>
                <Ionicons name="location" size={16} color={colors.text} /> {profile.location}
              </Text>
            )}
            {profile.website && (
              <Text style={{ color: colors.text }}>
                <Ionicons name="globe" size={16} color={colors.text} /> {profile.website}
              </Text>
            )}
          </View>
        )}

        <View className="mt-5 flex-row justify-between border-b border-t border-gray-200 py-5 dark:border-gray-700">
          <StatTile title="Pets" value="0" />
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  avatarContainer: {
    marginRight: 16,
  },
});
