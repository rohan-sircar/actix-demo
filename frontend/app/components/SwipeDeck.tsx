import React, { useState, useCallback } from 'react';
import { View, Text, TouchableOpacity, StyleSheet, Platform } from 'react-native';
import { useRouter } from 'expo-router';
import { useSafeAreaInsets } from 'react-native-safe-area-context';
import type { MockPet } from '~/data/pets';
import { SwipeDeckCard } from './SwipeDeckCard';

interface Props {
  pets: MockPet[];
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
}

const STACK_DEPTH = 3;

export function SwipeDeck({ pets, colors, accentSet, isDarkColorScheme }: Props) {
  const router = useRouter();
  const insets = useSafeAreaInsets();
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
      <View style={[styles.emptyContainer, { backgroundColor: colors.background, paddingTop: insets.top + 32, paddingBottom: insets.bottom + 32 }]}>
        <Text style={[styles.emptyEmoji]}>🐾</Text>
        <Text style={[styles.emptyTitle, { color: colors.text }]}>
          No more pets to discover
        </Text>
        <Text style={[styles.emptySubtitle, { color: colors.grey }]}>
          You've seen all the pets in your area
        </Text>
        <TouchableOpacity style={[styles.resetButton, { backgroundColor: accentSet.base }]} onPress={handleReset}>
          <Text style={styles.resetButtonText}>Start Over</Text>
        </TouchableOpacity>
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
  resetButton: {
    paddingHorizontal: 32,
    paddingVertical: 14,
    borderRadius: 24,
  },
  resetButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
