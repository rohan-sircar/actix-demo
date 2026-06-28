import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  Image as RNImage,
  ScrollView as ScrollViewNative,
  StyleSheet,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import type { PublicPet, PetImage } from '~/app/models/pets';
import { getImageUrl, getAgeFromDob, getSpeciesIcon } from './gallery-utils';

interface Props {
  pet: PublicPet;
  images: PetImage[];
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
  onFullProfile: () => void;
  onLike?: () => void;
  onDislike?: () => void;
  interactionState?: { interacted: boolean; direction: string | null };
}

const TABS = ['All', 'Dogs', 'Cats', 'Birds'];

export function NativeGallery({ pet, images, colors, accentSet, isDarkColorScheme, onFullProfile, onLike, onDislike, interactionState }: Props) {
  const [currentIndex, setCurrentIndex] = useState(0);
  const [activeTab, setActiveTab] = useState('All');

  const fallbackImages: PetImage[] = pet.primary_image
    ? [{ id: 0, uuid: pet.primary_image.uuid, format: 'jpeg' as const, is_primary: true, sort_order: 0, created_at: '' }]
    : [];
  const allImages = images.length > 0 ? images : fallbackImages;

  const goToPrev = useCallback(() => {
    setCurrentIndex((i) => (i === 0 ? allImages.length - 1 : i - 1));
  }, [allImages.length]);

  const goToNext = useCallback(() => {
    setCurrentIndex((i) => (i === allImages.length - 1 ? 0 : i + 1));
  }, [allImages.length]);

  const ageStr = pet.date_of_birth ? getAgeFromDob(pet.date_of_birth) : null;

  const pills: { icon: string; label: string }[] = [];
  pills.push({ icon: getSpeciesIcon(pet.species), label: `${pet.species}${pet.breed ? ` · ${pet.breed}` : ''}` });
  if (pet.gender) {
    pills.push({ icon: pet.gender.toLowerCase() === 'male' ? 'male' : 'female', label: pet.gender });
  }
  if (ageStr) {
    pills.push({ icon: 'calendar', label: `${ageStr} old` });
  }
  if (pet.traits) {
    for (const t of pet.traits.slice(0, 3)) {
      pills.push({ icon: 'star', label: t.name });
    }
  }

  const currentImage = allImages[currentIndex];

  return (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
      {/* Top bar */}
      {onLike && onDislike ? (
        <View style={styles.topBar}>
          <TouchableOpacity style={styles.topBarButton}>
            <Ionicons name="filter" size={24} color="#fff" />
          </TouchableOpacity>
          <ScrollViewNative
            horizontal
            showsHorizontalScrollIndicator={false}
            contentContainerStyle={styles.tabRow}
          >
            {TABS.map((tab) => (
              <TouchableOpacity
                key={tab}
                onPress={() => setActiveTab(tab)}
                style={[
                  styles.tabButton,
                  activeTab === tab ? styles.tabActive : styles.tabInactive,
                ]}
              >
                <Text style={[styles.tabText, activeTab === tab ? styles.tabTextActive : styles.tabTextInactive]}>
                  {tab}
                </Text>
              </TouchableOpacity>
            ))}
          </ScrollViewNative>
          <TouchableOpacity style={styles.topBarButton}>
            <Ionicons name="flash" size={24} color="#fff" />
          </TouchableOpacity>
        </View>
      ) : null}

      {/* Photo */}
      <View style={styles.photoArea}>
        {currentImage && (
          <RNImage
            source={{ uri: getImageUrl(currentImage.uuid, 'medium') }}
            style={styles.photo}
            resizeMode="cover"
          />
        )}

        {/* Left tap zone */}
        <TouchableOpacity
          style={styles.tapZoneLeft}
          activeOpacity={1}
          onPress={goToPrev}
        />

        {/* Right tap zone */}
        <TouchableOpacity
          style={styles.tapZoneRight}
          activeOpacity={1}
          onPress={goToNext}
        />

        {/* Page dots - top center */}
        {allImages.length > 1 && (
          <View style={styles.dotsContainer}>
            {allImages.map((_, idx) => (
              <View
                key={idx}
                style={{
                  width: idx === currentIndex ? 20 : 6,
                  height: 4,
                  borderRadius: 2,
                  marginHorizontal: 2,
                  backgroundColor: idx === currentIndex ? '#fff' : 'rgba(255,255,255,0.4)',
                }}
              />
            ))}
          </View>
        )}

        {/* Info overlay */}
        <View style={styles.infoOverlay}>
          {/* Trait pills */}
          <View style={styles.pillsContainer}>
            {pills.map((pill, idx) => (
              <View key={idx} style={styles.pill}>
                <Ionicons name={pill.icon as any} size={14} color="#fff" />
                <Text style={styles.pillText}>{pill.label}</Text>
              </View>
            ))}
          </View>

          {/* Name + age */}
          <View style={styles.nameRow}>
            <Text style={styles.nameText}>
              {pet.name}
            </Text>
            {ageStr && (
              <Text style={styles.ageText}>
                {ageStr}
              </Text>
            )}
            <TouchableOpacity style={styles.arrowButton}>
              <Ionicons name="arrow-up" size={20} color="#fff" />
            </TouchableOpacity>
          </View>

          {/* Action buttons */}
          {onLike && onDislike ? (
            <View style={styles.actionRow}>
              <TouchableOpacity style={[styles.actionButton, styles.actionSmall]} onPress={() => {}}>
                <Ionicons name="refresh" size={22} color="#aaa" />
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.actionButton, styles.actionMedium, interactionState?.interacted ? { opacity: 0.5 } : {}]}
                onPress={!interactionState?.interacted ? onDislike : undefined}>
                <Ionicons name={interactionState?.interacted && interactionState.direction === 'dislike' ? 'close-circle' : 'close'} size={30} color="#ff4458" />
              </TouchableOpacity>
              <TouchableOpacity style={[styles.actionButton, styles.actionSmall]} onPress={() => {}}>
                <Ionicons name="star" size={22} color="#2196f3" />
              </TouchableOpacity>
              <TouchableOpacity
                style={[styles.actionButton, styles.actionMedium, interactionState?.interacted ? { opacity: 0.5 } : {}]}
                onPress={!interactionState?.interacted ? onLike : undefined}>
                <Ionicons name="heart" size={30} color="#4ade80" />
              </TouchableOpacity>
              <TouchableOpacity style={[styles.actionButton, styles.actionSmall]} onPress={onFullProfile}>
                <Ionicons name="send" size={22} color="#3b82f6" />
              </TouchableOpacity>
            </View>
          ) : null}
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  topBar: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    zIndex: 10,
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingTop: 48,
    paddingBottom: 8,
    backgroundColor: 'rgba(0,0,0,0.3)',
  },
  topBarButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(0,0,0,0.4)',
  },
  tabRow: {
    flex: 1,
    flexDirection: 'row',
    gap: 8,
    marginHorizontal: 12,
  },
  tabButton: {
    paddingHorizontal: 16,
    paddingVertical: 6,
    borderRadius: 20,
  },
  tabActive: {
    backgroundColor: 'rgba(255,255,255,0.95)',
  },
  tabInactive: {
    backgroundColor: 'rgba(0,0,0,0.3)',
  },
  tabText: {
    fontSize: 14,
    fontWeight: '600',
  },
  tabTextActive: {
    color: '#000',
  },
  tabTextInactive: {
    color: 'rgba(255,255,255,0.7)',
  },
  photoArea: {
    flex: 1,
  },
  photo: {
    width: '100%',
    height: '100%',
  },
  tapZoneLeft: {
    position: 'absolute',
    top: 0,
    left: 0,
    width: '50%',
    height: '100%',
    zIndex: 5,
  },
  tapZoneRight: {
    position: 'absolute',
    top: 0,
    right: 0,
    width: '50%',
    height: '100%',
    zIndex: 5,
  },
  dotsContainer: {
    position: 'absolute',
    top: 100,
    left: 0,
    right: 0,
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 6,
  },
  infoOverlay: {
    position: 'absolute',
    bottom: 0,
    left: 0,
    right: 0,
    paddingVertical: 20,
    paddingHorizontal: 16,
    backgroundColor: 'rgba(0,0,0,0.5)',
    zIndex: 6,
  },
  pillsContainer: {
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 6,
    marginBottom: 12,
  },
  pill: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: 'rgba(0,0,0,0.5)',
    paddingHorizontal: 10,
    paddingVertical: 5,
    borderRadius: 20,
    gap: 4,
  },
  pillText: {
    fontSize: 12,
    fontWeight: '600',
    color: '#fff',
  },
  nameRow: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    justifyContent: 'space-between',
    marginBottom: 16,
  },
  nameText: {
    fontSize: 32,
    fontWeight: '700',
    color: '#fff',
  },
  ageText: {
    fontSize: 32,
    fontWeight: '700',
    color: '#fff',
    marginLeft: 8,
  },
  arrowButton: {
    width: 40,
    height: 40,
    borderRadius: 20,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'rgba(0,0,0,0.4)',
  },
  actionRow: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: 16,
    paddingBottom: 8,
  },
  actionButton: {
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: 50,
    backgroundColor: 'rgba(0,0,0,0.3)',
  },
  actionSmall: {
    width: 48,
    height: 48,
  },
  actionMedium: {
    width: 60,
    height: 60,
    borderWidth: 2,
    borderColor: 'rgba(255,255,255,0.1)',
  },
});
