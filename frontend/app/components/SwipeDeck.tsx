import React, { useState, useCallback, useEffect, useRef } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Platform } from 'react-native';
import { useRouter } from 'expo-router';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import type { PublicPet } from '~/app/models/pets';
import { discoverApi, likesApi } from '~/app/lib/api';
import { SwipeDeckCard } from './SwipeDeckCard';

interface Props {
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
}

const STACK_DEPTH = 3;
const MAX_PETS = 50;

export function SwipeDeck({ colors, accentSet, isDarkColorScheme }: Props) {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const [deck, setDeck] = useState<PublicPet[]>([]);
  const [loading, setLoading] = useState(true);
  const fetchCountRef = useRef(0);

  const fetchNextPet = useCallback(async () => {
    try {
      const pet = await discoverApi.next();
      if (pet) {
        setDeck((prev) => {
          const exists = prev.some((p) => p.pet_uuid === pet.pet_uuid);
          if (exists) return prev;
          const next = [...prev, pet];
          if (next.length > MAX_PETS) {
            return next.slice(1);
          }
          return next;
        });
      }
    } catch (err) {
      console.error('Failed to fetch next pet:', err);
      if ((err as any)?.response?.status === 404) {
        setLoading(false);
      }
    }
  }, []);

  useEffect(() => {
    const initialFetch = async () => {
      try {
        setLoading(true);
        const pets: PublicPet[] = [];
        for (let i = 0; i < MAX_PETS && pets.length < STACK_DEPTH; i++) {
          const pet = await discoverApi.next();
          if (pet && !pets.some((p) => p.pet_uuid === pet.pet_uuid)) {
            pets.push(pet);
            fetchCountRef.current++;
          } else if (!pet) {
            break;
          }
        }
        setDeck(pets);
      } catch (err) {
        console.error('Failed to initial fetch pets:', err);
      } finally {
        setLoading(false);
      }
    };
    initialFetch();
  }, []);

  const handleLike = useCallback(async () => {
    const pet = deck[0];
    if (!pet) return;

    try {
      await likesApi.create(pet.pet_uuid, 'like');
    } catch (err) {
      console.error('Failed to record like:', err);
    }

    setDeck((prev) => {
      if (prev.length <= 1) return [];
      return prev.slice(1);
    });
    fetchCountRef.current += 1;
    if (fetchCountRef.current < MAX_PETS) {
      fetchNextPet();
    }
  }, [deck, fetchNextPet]);

  const handleDislike = useCallback(async () => {
    const pet = deck[0];
    if (!pet) return;

    try {
      await likesApi.create(pet.pet_uuid, 'dislike');
    } catch (err) {
      console.error('Failed to record dislike:', err);
    }

    setDeck((prev) => {
      if (prev.length <= 1) return [];
      return prev.slice(1);
    });
    fetchCountRef.current += 1;
    if (fetchCountRef.current < MAX_PETS) {
      fetchNextPet();
    }
  }, [deck, fetchNextPet]);

  const handleFullProfile = useCallback(() => {
    const pet = deck[0];
    if (pet) {
      router.push(`/pet-view/profile/${pet.pet_uuid}`);
    }
  }, [deck, router]);

  const handleReset = useCallback(() => {
    setDeck([]);
    fetchCountRef.current = 0;
  }, []);

  if (loading) {
    return (
      <View style={[styles.emptyContainer, { backgroundColor: colors.background, paddingTop: insets.top + 32, paddingBottom: insets.bottom + 32 }]}>
        <Text style={[styles.emptyEmoji]}>🐾</Text>
        <Text style={[styles.emptySubtitle, { color: colors.grey }]}>Loading pets...</Text>
      </View>
    );
  }

  if (deck.length === 0) {
    return (
      <View style={[styles.emptyContainer, { backgroundColor: colors.background, paddingTop: insets.top + 32, paddingBottom: insets.bottom + 32 }]}>
        <Text style={[styles.emptyEmoji]}>🐾</Text>
        <Text style={[styles.emptyTitle, { color: colors.text }]}>
          No more pets to discover
        </Text>
        <Text style={[styles.emptySubtitle, { color: colors.grey }]}>
          You've seen all the pets in your area
        </Text>
      </View>
    );
  }

  const visibleCards = deck.slice(0, STACK_DEPTH).reverse();

  return (
    <View style={[styles.container, { paddingTop: insets.top + 8, paddingBottom: insets.bottom + 16 }]}>
      {visibleCards.map((pet, reverseIndex) => {
        const index = visibleCards.length - 1 - reverseIndex;
        const isTop = index === 0;
        return (
          <SwipeDeckCard
            key={pet.pet_uuid}
            pet={pet}
            colors={colors}
            accentSet={accentSet}
            isDarkColorScheme={isDarkColorScheme}
            onLike={handleLike}
            onDislike={handleDislike}
            onFullProfile={() => router.push({ pathname: '/(tabs)/pet-profiles/[pet_uuid]', params: { pet_uuid: pet.pet_uuid } })}
            isTopCard={isTop}
            stackIndex={index}
            petIndex={visibleCards.length - 1 - reverseIndex}
          />
        );
      })}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    paddingHorizontal: 16,
  },
  emptyContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    paddingHorizontal: 32,
  },
  emptyEmoji: {
    fontSize: 64,
    marginBottom: 16,
  },
  emptyTitle: {
    fontSize: 22,
    fontWeight: '700',
    textAlign: 'center',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 15,
    textAlign: 'center',
    marginBottom: 32,
  },
});
