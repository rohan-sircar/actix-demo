import React, { useState, useCallback, useEffect, useRef } from 'react';
import { useRouter } from 'expo-router';
import type { PublicPet } from '~/app/models/pets';
import { discoverApi, likesApi } from '~/app/lib/api';
import { SwipeDeckCardWeb } from './SwipeDeckCardWeb';

interface Props {
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
}

const STACK_DEPTH = 3;

function buildDeck(pets: PublicPet[]): PublicPet[] {
  return [...pets];
}

export function SwipeDeckWeb({ colors, accentSet, isDarkColorScheme }: Props) {
  const router = useRouter();
  const [deck, setDeck] = useState<PublicPet[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const fetchCountRef = useRef(0);
  const MAX_PETS = 50;

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
        for (let i = 0; i < STACK_DEPTH && i < MAX_PETS; i++) {
          const pet = await discoverApi.next();
          if (pet) {
            pets.push(pet);
            fetchCountRef.current++;
          } else {
            break;
          }
        }
        setDeck(pets);
      } catch (err) {
        console.error('Failed to initial fetch pets:', err);
        setError('Failed to load pets');
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

  if (loading) {
    return (
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 32,
        backgroundColor: colors.background,
      }}>
        <span style={{ fontSize: 64, marginBottom: 16 }}>🐾</span>
        <p style={{ fontSize: 16, color: colors.grey }}>Loading pets...</p>
      </div>
    );
  }

  if (error || deck.length === 0) {
    return (
      <div style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        padding: 32,
        backgroundColor: colors.background,
      }}>
        <span style={{ fontSize: 64, marginBottom: 16 }}>🐾</span>
        <h2 style={{ fontSize: 22, fontWeight: 700, textAlign: 'center', marginBottom: 8, color: colors.text }}>
          No more pets to discover
        </h2>
        <p style={{ fontSize: 15, textAlign: 'center', marginBottom: 32, color: colors.grey }}>
          You&apos;ve seen all the pets in your area
        </p>
      </div>
    );
  }

  const visibleCards = deck.slice(0, STACK_DEPTH).reverse();

  return (
    <div style={{
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      padding: '8px 16px',
      maxWidth: 430,
      width: '100%',
      margin: '0 auto',
      height: '100%',
      position: 'relative',
    }}>
      {visibleCards.map((pet, reverseIndex) => {
        const index = visibleCards.length - 1 - reverseIndex;
        const isTop = index === 0;
        return (
          <SwipeDeckCardWeb
            key={pet.pet_uuid}
            pet={pet}
            colors={colors}
            accentSet={accentSet}
            isDarkColorScheme={isDarkColorScheme}
            onLike={handleLike}
            onDislike={handleDislike}
            onFullProfile={handleFullProfile}
            isTopCard={isTop}
            stackIndex={index}
          />
        );
      })}
    </div>
  );
}
