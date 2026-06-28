import { Ionicons } from '@expo/vector-icons';
import React, { useCallback, useEffect, useState } from 'react';
import { ScrollView, Text, TextInput, TouchableOpacity, View } from 'react-native';
import { useFocusEffect, useRouter } from 'expo-router';

import Avatar from '../../components/Avatar';
import StatTile from '../../components/StatTile';
import * as Style from '~/app/styles/Styles';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore, UserResponse } from '~/app/stores/AuthStore';
import api, { profileApi } from '~/app/lib/api';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import type { Pet, UserProfile as UserProfileType } from '~/app/models/pets';

export default function ProfileScreen() {
  const router = useRouter();
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  useEffect(() => {
    if (!isAuthenticated) {
      router.replace('/login');
    }
  }, [isAuthenticated, router]);
  const queryClient = useQueryClient();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [isEditing, setIsEditing] = useState(false);
  const [displayName, setDisplayName] = useState('');
  const [bio, setBio] = useState('');
  const [location, setLocation] = useState('');
  const [website, setWebsite] = useState('');

  const { data: user, isLoading: userLoading } = useQuery({
    queryKey: ['user'],
    queryFn: async () => {
      const res = await api.get<UserResponse>('/api/v1/private/user');
      return res.data;
    },
  });

  const { data: userProfile, isLoading: profileLoading } = useQuery({
    queryKey: ['user-profile'],
    queryFn: async () => {
      const res = await profileApi.get();
      return res;
    },
  });

  const { data: pets, refetch: refetchPets } = useQuery({
    queryKey: ['pets-count'],
    queryFn: async () => {
      const res = await api.get<Pet[]>('/api/v1/private/user/pets');
      return res.data;
    },
  });

  useFocusEffect(
    useCallback(() => {
      refetchPets();
    }, [refetchPets])
  );

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
      queryClient.invalidateQueries({ queryKey: ['user-profile'] });
      setIsEditing(false);
    },
  });

  const isLoading = userLoading || profileLoading;

  if (isLoading) {
    return (
      <View className="w-full flex-1 items-center justify-center" style={{ backgroundColor: colors.background }}>
        <Text style={{ color: colors.grey }}>Loading profile...</Text>
      </View>
    );
  }

  if (!user) return null;

  const profile: UserProfileType = userProfile || { display_name: null, bio: null, location: null, website_url: null, social_github: null, social_twitter: null };

  const handleSave = () => {
    const updates: Record<string, string> = {};
    if (displayName) updates.display_name = displayName;
    if (bio) updates.bio = bio;
    if (location) updates.location = location;
    if (website) updates.website_url = website;
    if (Object.keys(updates).length > 0) {
      updateProfileMutation.mutate(updates);
    } else {
      setIsEditing(false);
    }
  };

  return (
    <View className="w-full flex-1" style={{ backgroundColor: colors.background }}>
      <ScrollView className="w-full" showsVerticalScrollIndicator={false} contentContainerStyle={{ padding: 16, paddingBottom: 80 }}>
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
                value={website || profile.website_url || ''}
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
              style={{ backgroundColor: isDarkColorScheme ? colors.grey5 : accentSet.bgSubtle }}>
              <Text className="font-semibold" style={{ color: accentSet.base }}>
                Edit Profile
              </Text>
            </TouchableOpacity>
          )}
        </View>

        {/* Row 1: About Me + Stats */}
        <View className="flex-row flex-wrap gap-4" style={{ width: '100%' }}>
          {(profile.bio || profile.location || profile.website_url) && (
            <View style={{ width: '48%', borderRadius: 16, backgroundColor: colors.card, borderWidth: 1, borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8', shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.2, shadowRadius: 4, elevation: 3 }}>
              <View className="px-5 py-4">
                {profile.bio && (
                  <>
                    <Text className="mb-2 text-xs font-bold uppercase tracking-wider" style={{ color: accentSet.base }}>
                      About Me
                    </Text>
                    <Text className="text-sm leading-relaxed" style={{ color: colors.grey }}>
                      {profile.bio}
                    </Text>
                  </>
                )}
                {(profile.location || profile.website_url) && (
                  <View className="mt-3 flex-row flex-wrap gap-2">
                    {profile.location && (
                      <View className="flex-row items-center gap-1.5 rounded-full bg-black/10 px-3.5 py-2" style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)' }}>
                        <Ionicons name="location" size={15} color={accentSet.base} />
                        <Text className="text-sm font-medium" style={{ color: colors.text }}>
                          {profile.location}
                        </Text>
                      </View>
                    )}
                    {profile.website_url && (
                      <View className="flex-row items-center gap-1.5 rounded-full bg-black/10 px-3.5 py-2" style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)' }}>
                        <Ionicons name="globe" size={15} color={accentSet.base} />
                        <Text className="text-sm font-medium" style={{ color: colors.text }}>
                          {profile.website_url}
                        </Text>
                      </View>
                    )}
                  </View>
                )}
              </View>
            </View>
          )}

          <View style={{ width: profile.bio || profile.location || profile.website_url ? '48%' : '100%', borderRadius: 16, backgroundColor: colors.card, borderWidth: 1, borderColor: isDarkColorScheme ? colors.grey4 : '#e8e8e8', shadowColor: '#000', shadowOffset: { width: 0, height: 2 }, shadowOpacity: 0.2, shadowRadius: 4, elevation: 3 }}>
            <View className="px-5 py-5">
              <Text className="mb-3 text-xs font-bold uppercase tracking-wider text-center" style={{ color: accentSet.base }}>
                My Stats
              </Text>
              <View className="flex-row justify-center">
                <StatTile title="Pets" value={String(pets?.length ?? 0)} />
              </View>
            </View>
          </View>
        </View>

        {/* Row 2: Likes & Matches navigation */}
        <View className="mt-4 flex-row gap-4">
          <TouchableOpacity
            onPress={() => router.push('../likes')}
            className="flex-1 items-center rounded-xl py-3.5"
            style={{ backgroundColor: accentSet.base }}>
            <View className="flex-row items-center gap-2">
              <Ionicons name="heart-outline" size={18} color="#fff" />
              <Text className="font-semibold text-white">View Likes</Text>
            </View>
          </TouchableOpacity>
          <TouchableOpacity
            onPress={() => router.push('../matches')}
            className="flex-1 items-center rounded-xl py-3.5"
            style={{ backgroundColor: isDarkColorScheme ? '#3d2a22' : accentSet.bgSubtle }}>
            <View className="flex-row items-center gap-2">
              <Ionicons name="heart" size={18} color={accentSet.base} />
              <Text className="font-semibold" style={{ color: colors.text }}>View Matches</Text>
            </View>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </View>
  );
}
