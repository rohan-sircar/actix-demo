import { Ionicons } from '@expo/vector-icons';
import React, { useState } from 'react';
import { Image, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import { useRouter } from 'expo-router';

import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { getImageUrl, likesApi } from '~/app/lib/api';
import { useQuery } from '@tanstack/react-query';
import type { LikeWithPet } from '~/app/models/pets';

type Tab = 'sent' | 'received';

function LikeItem({ like, colors, isDarkColorScheme, accentSet }: {
  like: LikeWithPet;
  colors: any;
  isDarkColorScheme: boolean;
  accentSet: any;
}) {
  const router = useRouter();

  const displayName = like.liker?.display_name || 'Unknown';
  const hasLiker = !!like.liker;

  return (
    <TouchableOpacity
      onPress={() => router.push(`/pet-view/${like.pet_uuid}`)}
      className="flex-row items-center rounded-xl px-4 py-3"
      style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.04)' : 'rgba(0,0,0,0.03)' }}>
      {like.primary_image_uuid ? (
        <Image source={{ uri: getImageUrl(like.primary_image_uuid, 'thumbnail') }} style={{ width: 56, height: 56, borderRadius: 14 }} resizeMode="cover" />
      ) : (
        <View style={{ width: 56, height: 56, borderRadius: 14, backgroundColor: accentSet.bgSubtle || '#f0e0d8', justifyContent: 'center', alignItems: 'center' }}>
          <Ionicons name="paw" size={24} color={accentSet.base} />
        </View>
      )}
      <View className="flex-1 ml-3">
        <Text className="text-base font-semibold" style={{ color: colors.text }}>{like.pet_name}</Text>
        <Text className="text-xs" style={{ color: colors.grey }}>{like.species}</Text>
      </View>
      {hasLiker && (
        <TouchableOpacity
          onPress={(e) => {
            e.stopPropagation();
            router.push(`/likes/user-profile/${like.liker!.user_uuid}`);
          }}
          className="flex-row items-center gap-2 pr-3">
          {like.liker!.avatar_url ? (
            <Image source={{ uri: getImageUrl(like.liker!.avatar_url, 'thumbnail') }} style={{ width: 28, height: 28, borderRadius: 14 }} resizeMode="cover" />
          ) : (
            <Ionicons name="person-circle" size={28} color={colors.grey} />
          )}
          <View>
            <Text className="text-xs" style={{ color: colors.grey }}>Liked by</Text>
            <Text className="text-sm font-semibold" style={{ color: accentSet.base }}>{displayName}</Text>
          </View>
        </TouchableOpacity>
      )}
      {like.is_match ? (
        <View style={{ backgroundColor: '#4ade80', paddingHorizontal: 10, paddingVertical: 4, borderRadius: 10 }}>
          <Text style={{ color: '#fff', fontSize: 11, fontWeight: 700 }}>MATCH</Text>
        </View>
      ) : null}
    </TouchableOpacity>
  );
}

function LikeListSection({ title, likes, colors, isDarkColorScheme, accentSet }: {
  title: string;
  likes: LikeWithPet[] | undefined;
  colors: any;
  isDarkColorScheme: boolean;
  accentSet: any;
}) {
  return (
    <View className="px-4 py-2 gap-2">
      {likes?.length === 0 ? (
        <Text className="text-xs text-center py-8" style={{ color: colors.grey }}>
          {title === 'Sent' ? 'No likes sent yet' : 'No likes received yet'}
        </Text>
      ) : null}
      {likes?.map((like) => (
        <LikeItem key={like.pet_uuid} like={like} colors={colors} isDarkColorScheme={isDarkColorScheme} accentSet={accentSet} />
      ))}
    </View>
  );
}

export default function LikesScreen() {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [activeTab, setActiveTab] = useState<Tab>('sent');

  const { data: likesSent } = useQuery({
    queryKey: ['likes-sent'],
    queryFn: () => likesApi.listSent(),
  });

  const { data: likesReceived } = useQuery({
    queryKey: ['likes-received'],
    queryFn: () => likesApi.listReceived(),
  });

  const tabs: { key: Tab; label: string; count: number }[] = [
    { key: 'sent', label: 'Sent', count: likesSent?.length ?? 0 },
    { key: 'received', label: 'Received', count: likesReceived?.length ?? 0 },
  ];

  const activeLikes = activeTab === 'sent' ? likesSent : likesReceived;
  const activeTitle = activeTab === 'sent' ? 'Sent' : 'Received';

  return (
    <View className="w-full flex-1" style={{ backgroundColor: colors.background }}>
      <View className="px-4 pt-4">
        <Text className="text-2xl font-bold mb-4" style={{ color: colors.text }}>
          Likes
        </Text>

        <View className="flex-row rounded-xl p-1 gap-1" style={{ backgroundColor: isDarkColorScheme ? 'rgba(255,255,255,0.06)' : 'rgba(0,0,0,0.05)' }}>
          {tabs.map((tab) => (
            <TouchableOpacity
              key={tab.key}
              onPress={() => setActiveTab(tab.key)}
              className="flex-1 items-center rounded-lg py-2.5"
              style={{
                backgroundColor: activeTab === tab.key ? accentSet.base : 'transparent',
              }}>
              <Text
                className="text-sm font-semibold"
                style={{ color: activeTab === tab.key ? '#fff' : colors.grey }}>
                {tab.label} ({tab.count})
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      <ScrollView className="flex-1 mt-2" showsVerticalScrollIndicator={false} contentContainerStyle={{ paddingBottom: 80 }}>
        <LikeListSection
          title={activeTitle}
          likes={activeLikes}
          colors={colors}
          isDarkColorScheme={isDarkColorScheme}
          accentSet={accentSet}
        />
      </ScrollView>
    </View>
  );
}
