import { Ionicons } from '@expo/vector-icons';
import React, { useEffect } from 'react';
import { Alert, Pressable, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';

import FontAwesome from '@expo/vector-icons/FontAwesome';
import { Icon } from '@roninoss/icons';

import SettingsCard from '~/app/components/SettingsCard';
import SettingsRow from '~/app/components/SettingsRow';
import SettingsSectionHeader from '~/app/components/SettingsSectionHeader';
import AccentColorButton from '~/components/AccentColorButton';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAuthStore } from '../../stores/AuthStore';
import { useQuery } from '@tanstack/react-query';
import api from '~/app/lib/api';
import type { UserResponse } from '../../stores/AuthStore';

export default function SettingsScreen() {
  const router = useRouter();
  const { colors, isDarkColorScheme, toggleColorScheme, colorScheme } = useColorScheme();
  const { accentColor, setAccentColor } = useAccentColor();
  const { isAuthenticated, user, clearCredentials } = useAuthStore();

  useEffect(() => {
    if (!isAuthenticated) {
      router.replace('/login');
    }
  }, [isAuthenticated, router]);

  const { data: userData } = useQuery<UserResponse, Error>({
    queryKey: ['user'],
    queryFn: async () => {
      const res = await api.get<UserResponse>('/api/v1/private/user');
      return res.data;
    },
    enabled: !!isAuthenticated,
  });

  const { data: sessionsMap } = useQuery({
    queryKey: ['sessions'],
    queryFn: async () => {
      const res = await api.get<Record<string, { session_id: string }>>('/api/v1/private/sessions');
      return res.data;
    },
    enabled: !!isAuthenticated,
  });

  const accentSet = getAccentSet(accentColor);
  const profile = userData?.profile || {};
  const sessionCount = Object.keys(sessionsMap || {}).length;

  const handleLogout = () => {
    Alert.alert('Log Out', 'Are you sure you want to log out?', [
      { text: 'Cancel', style: 'cancel' },
      {
        text: 'Log Out',
        style: 'destructive',
        onPress: async () => {
          await clearCredentials();
        },
      },
    ]);
  };

  const handleDeleteAccount = () => {
    Alert.alert(
      'Delete Account',
      'This action is permanent and cannot be undone. All your data including pets, sessions, and profile will be removed.',
      [
        { text: 'Cancel', style: 'cancel' },
        {
          text: 'Delete',
          style: 'destructive',
          onPress: () => {
            Alert.alert('Coming Soon', 'Account deletion is not yet available.');
          },
        },
      ]
    );
  };

  const navigateToProfile = () => {
    router.push('/profile');
  };

  const navigateToSessions = () => {
    router.push('/settings/sessions');
  };

  const navigateToMenu = (action: string) => {
    Alert.alert('Coming Soon', 'Help & Support is not yet available.');
  };

  const themeLabel = colorScheme === 'dark' ? 'Dark' : 'Light';

  return (
    <ScrollView
      className="flex-1 px-4 md:px-8"
      contentContainerStyle={{ paddingTop: 16, paddingBottom: 40 }}
      keyboardShouldPersistTaps="handled">
      {/* Account Info Card */}
      <SettingsCard>
        <View className="flex-row items-center mb-3 mt-2 pt-2">
          <View
            style={{
              width: 56,
              height: 56,
              borderRadius: 28,
              backgroundColor: accentSet.bgSubtle,
              alignItems: 'center',
              justifyContent: 'center',
            }}>
            <FontAwesome name="user" size={26} color={accentSet.base} />
          </View>
          <View className="flex-1 ml-4">
            <Text className="text-lg font-bold" style={{ color: colors.text }}>
              {profile.display_name || user?.username || 'User'}
            </Text>
            <Text className="text-sm" style={{ color: colors.grey }}>
              @{user?.username}
            </Text>
          </View>
        </View>
        {user?.email ? (
          <Text className="text-sm mb-2" style={{ color: colors.grey }}>
            {user.email}
          </Text>
        ) : null}
      </SettingsCard>

      {/* Application Preferences */}
      <SettingsSectionHeader title="Application Preferences" />
      <SettingsCard>
        <SettingsRow
          icon="palette"
          faIcon="paint-brush"
          title="Accent Color"
          subtitle="Choose your preferred color theme"
          rightContent={
            <View className="flex-row flex-wrap gap-1.5 justify-end" style={{ maxWidth: 140 }}>
              {(['coral', 'green', 'orange', 'peach', 'rose', 'teal'] as const).map((color) => (
                <TouchableOpacity
                  key={color}
                  onPress={() => setAccentColor(color as any)}
                  style={{
                    width: 24,
                    height: 24,
                    borderRadius: 12,
                    backgroundColor: getAccentSet(color as any).base,
                    opacity: accentColor === color ? 1 : 0.5,
                    borderWidth: accentColor === color ? 2 : 0,
                    borderColor: colors.card,
                  }}
                />
              ))}
            </View>
          }
        />
        <SettingsRow
          icon={isDarkColorScheme ? 'moon' : 'sun'}
          faIcon={isDarkColorScheme ? 'moon-o' : 'sun-o'}
          title="Theme"
          subtitle={themeLabel}
          onPress={toggleColorScheme}
          rightContent={
            <View
              style={{
                width: 44,
                height: 26,
                borderRadius: 13,
                backgroundColor: isDarkColorScheme ? accentSet.base : colors.grey4,
                alignItems: 'center',
                justifyContent: 'center',
                padding: 3,
              }}>
              <View
                style={{
                  width: 20,
                  height: 20,
                  borderRadius: 10,
                  backgroundColor: 'white',
                  marginLeft: isDarkColorScheme ? 18 : 0,
                  marginRight: isDarkColorScheme ? 0 : 18,
                }}
              />
            </View>
          }
        />
      </SettingsCard>

      {/* Account Management */}
      <SettingsSectionHeader title="Account" />
      <SettingsCard style={{ marginBottom: 8 }}>
        <SettingsRow
          icon="pencil"
          faIcon="pencil"
          title="Edit Profile"
          subtitle="Display name, bio, location, website"
          onPress={navigateToProfile}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
        <SettingsRow
          icon="lock"
          faIcon="key"
          title="Change Password"
          subtitle="Update your account password"
          onPress={() => Alert.alert('Coming Soon', 'Password change is not yet available.')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
        <SettingsRow
          icon="envelope"
          faIcon="envelope-o"
          title="Change Email"
          subtitle="Update your email address"
          onPress={() => Alert.alert('Coming Soon', 'Email change is not yet available.')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
      </SettingsCard>

      {/* Sessions */}
      <SettingsSectionHeader title="Sessions" />
      <SettingsCard style={{ marginBottom: 8 }}>
        <SettingsRow
          icon="monitor"
          faIcon="desktop"
          title="Active Sessions"
          subtitle={sessionCount > 0 ? `${sessionCount} device${sessionCount > 1 ? 's' : ''} logged in` : 'No active sessions'}
          onPress={navigateToSessions}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
      </SettingsCard>

      {/* Logout */}
      {isAuthenticated && (
        <SettingsCard style={{ marginBottom: 8 }}>
          <SettingsRow
            icon="logout"
            faIcon="sign-out"
            title="Log Out"
            destructive
            onPress={handleLogout}
          />
        </SettingsCard>
      )}

      {/* Support & About */}
      <SettingsSectionHeader title="Support" />
      <SettingsCard style={{ marginBottom: 8 }}>
        <SettingsRow
          icon="question"
          faIcon="question-circle"
          title="Help & Support"
          onPress={() => navigateToMenu('help')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
        <SettingsRow
          icon="shield"
          faIcon="shield"
          title="Privacy Policy"
          onPress={() => Alert.alert('Coming Soon', 'Privacy Policy is not yet available.')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
        <SettingsRow
          icon="info"
          faIcon="info-circle"
          title="About PetMatch"
          subtitle="Version 1.0.0"
          onPress={() => Alert.alert('PetMatch', 'PetMatch v1.0.0\n\nConnect with pets in your area.')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
        <SettingsRow
          icon="star"
          faIcon="star"
          title="Rate the App"
          onPress={() => Alert.alert('Coming Soon', 'Rating is not yet available.')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
        <SettingsRow
          icon="share"
          faIcon="share-alt"
          title="Share with Friends"
          onPress={() => Alert.alert('Coming Soon', 'Sharing is not yet available.')}
          rightContent={<FontAwesome name="chevron-right" size={14} color={colors.grey2} />}
        />
      </SettingsCard>

      {/* Footer */}
      <View style={{ alignItems: 'center', marginTop: 16, paddingBottom: 16 }}>
        <View style={{ flexDirection: 'row', alignItems: 'center', gap: 6 }}>
          <FontAwesome name="paw" size={14} color={accentSet.base} />
          <Text style={{ color: colors.grey, fontSize: 12, fontWeight: '500' }}>
            PetMatch v1.0.0
          </Text>
        </View>
      </View>
    </ScrollView>
  );
}
