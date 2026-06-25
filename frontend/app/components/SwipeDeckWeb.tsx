import React, { useState, useCallback } from 'react';
import { useRouter } from 'expo-router';
import type { MockPet } from '~/data/pets';
import { SwipeDeckCardWeb } from './SwipeDeckCardWeb';

interface Props {
  pets: MockPet[];
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
}

const STACK_DEPTH = 3;

export function SwipeDeckWeb({ pets, colors, accentSet, isDarkColorScheme }: Props) {
  const router = useRouter();
  const [deck, setDeck] = useState<MockPet[]>([...pets]);

  const handleLike = useCallback(() => {
    setDeck((prev) => {
      if (prev.length <= 1) return [];
      return prev.slice(1);
    });
  }, []);

  const handleDislike = useCallback(() => {
    setDeck((prev) => {
      if (prev.length <= 1) return [];
      return prev.slice(1);
    });
  }, []);

  const handleFullProfile = useCallback(() => {
    const pet = deck[0];
    if (pet) {
      router.push(`/pet-view/profile/${pet.id}`);
    }
  }, [deck, router]);

  const handleReset = useCallback(() => {
    setDeck([...pets]);
  }, [pets]);

  if (deck.length === 0) {
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
        <button
          onClick={handleReset}
          style={{
            padding: '14px 32px',
            borderRadius: 24,
            border: 'none',
            cursor: 'pointer',
            backgroundColor: accentSet.base,
          }}
        >
          <span style={{ fontSize: 16, fontWeight: 600, color: '#fff' }}>Start Over</span>
        </button>
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
            key={pet.id}
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
