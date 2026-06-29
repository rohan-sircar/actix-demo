import React from 'react';
import { Platform, View } from 'react-native';
import { Image } from 'expo-image';
import { useAuthStore } from '~/app/stores/AuthStore';
import { getImageUrl } from '~/app/lib/api';

interface Props {
  imageUuid: string | null | undefined;
  variant?: 'thumbnail' | 'medium' | 'original';
  style?: object;
  resizeMode?: 'cover' | 'contain' | 'stretch';
  pointerEvents?: 'box-none' | 'none' | 'box-only' | 'auto';
}

const CONTENT_FIT_MAP: Record<string, 'contain' | 'cover' | 'fill'> = {
  cover: 'cover',
  contain: 'contain',
  stretch: 'fill',
};

const OBJECT_FIT_MAP: Record<string, string> = {
  cover: 'cover',
  contain: 'contain',
  stretch: 'fill',
};

export function AuthenticatedImage({ imageUuid, variant = 'medium', style, resizeMode = 'cover', pointerEvents }: Props) {
  const token = useAuthStore(s => s.token);

  if (!imageUuid) {
    return null;
  }

  // On web, use native <img> tag — browser sends cookies automatically
  if (Platform.OS === 'web') {
    const uri = getImageUrl(imageUuid, variant);
    const objectFit = OBJECT_FIT_MAP[resizeMode] || 'cover';
    return (
      <View style={style} pointerEvents={pointerEvents}>
        {/* @ts-ignore: img tag on web */}
        <img src={uri} alt="" style={{ width: '100%', height: '100%', objectFit }} />
      </View>
    );
  }

  // On native, use expo-image with Bearer token headers
  const baseUrl = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';
  const uri = `${baseUrl}/api/v1/private/pets/images/${imageUuid}/${variant}`;

  return (
    <Image
      source={{ uri, headers: token ? { Authorization: `Bearer ${token}` } : {} }}
      style={style}
      contentFit={CONTENT_FIT_MAP[resizeMode] || 'cover'}
      transition={200}
      pointerEvents={pointerEvents}
    />
  );
}
