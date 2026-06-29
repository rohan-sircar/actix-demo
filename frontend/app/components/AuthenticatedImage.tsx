import React from 'react';
import { Image } from 'expo-image';
import { useAuthStore } from '~/app/stores/AuthStore';

interface Props {
  imageUuid: string | null | undefined;
  variant?: 'thumbnail' | 'medium' | 'original';
  style?: object;
  resizeMode?: 'cover' | 'contain' | 'stretch';
}

const CONTENT_FIT_MAP: Record<string, 'contain' | 'cover' | 'fill'> = {
  cover: 'cover',
  contain: 'contain',
  stretch: 'fill',
};

export function AuthenticatedImage({ imageUuid, variant = 'medium', style, resizeMode = 'cover' }: Props) {
  const token = useAuthStore(s => s.token);

  if (!imageUuid) {
    return null;
  }

  const baseUrl = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:8800';
  const uri = `${baseUrl}/api/v1/private/pets/images/${imageUuid}/${variant}`;

  return (
    <Image
      source={{ uri, headers: token ? { Authorization: `Bearer ${token}` } : {} }}
      style={style}
      contentFit={CONTENT_FIT_MAP[resizeMode] || 'cover'}
      transition={200}
    />
  );
}
