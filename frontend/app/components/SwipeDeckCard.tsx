import React, { useCallback, useEffect, useState, useRef } from 'react';
import {
  View,
  Text,
  TouchableOpacity,
  ScrollView as ScrollViewNative,
  StyleSheet,
  Platform,
  Dimensions,
} from 'react-native';
import { Ionicons } from '@expo/vector-icons';
import Animated, {
  useAnimatedStyle,
  withSpring,
  withTiming,
  interpolate,
  runOnJS,
  useSharedValue,
  withSequence,
  withDelay,
} from 'react-native-reanimated';
import { GestureDetector, Gesture } from 'react-native-gesture-handler';
import type { PublicPet, PetImage } from '~/app/models/pets';
import { getSpeciesIcon, getAgeFromDob } from '~/app/pet-view/gallery-utils';
import { petApi } from '~/app/lib/api';
import { AuthenticatedImage } from './AuthenticatedImage';

const SWIPE_THRESHOLD = 100;
const STAMP_THRESHOLD = 75;
const MAX_ROTATION = 0.15;

const GRADIENTS = [
  ['#FF6B6B', '#FF8E53'],
  ['#4ECDC4', '#44B09E'],
  ['#667EEA', '#764BA2'],
  ['#F093FB', '#F5576C'],
  ['#4FACFE', '#00F2FE'],
  ['#43E97B', '#38F9D7'],
  ['#FA709A', '#FEE140'],
  ['#A18CD1', '#FBC2EB'],
  ['#FF9A9E', '#FECFEF'],
  ['#A1C4FD', '#C2E9FB'],
];

interface Props {
  pet: PublicPet;
  colors: { background: string; text: string; grey: string; grey4?: string; card?: string; grey5?: string };
  accentSet: { base: string; bgSubtle?: string };
  isDarkColorScheme: boolean;
  onLike: () => void;
  onDislike: () => void;
  onFullProfile: () => void;
  isTopCard: boolean;
  stackIndex: number;
  petIndex: number;
}

const shadowStyle = Platform.OS === 'ios'
  ? { shadowColor: '#000' as any, shadowOffset: { width: 0, height: 4 }, shadowOpacity: 0.15, shadowRadius: 8 }
  : { elevation: 4 };

const shadowStyleTop = Platform.OS === 'ios'
  ? { shadowColor: '#000' as any, shadowOffset: { width: 0, height: 8 }, shadowOpacity: 0.25, shadowRadius: 16 }
  : { elevation: 8 };

const TABS = ['All', 'Dogs', 'Cats', 'Birds'];

const getGradientForPet = (petIndex: number) => GRADIENTS[petIndex % GRADIENTS.length];

export function SwipeDeckCard({
  pet,
  colors,
  accentSet,
  isDarkColorScheme,
  onLike,
  onDislike,
  onFullProfile,
  isTopCard,
  stackIndex,
  petIndex,
}: Props) {
  const [activeTab, setActiveTab] = useState('All');
  const [images, setImages] = useState<PetImage[]>([]);
  const [currentImageIndex, setCurrentImageIndex] = useState(0);
  const imagesLengthRef = useRef(0);
  imagesLengthRef.current = images.length;

  useEffect(() => {
    let cancelled = false;
    const fetchImages = async () => {
      try {
        const res = await petApi.getImages(pet.pet_uuid);
        if (!cancelled) {
          const sorted = res.sort((a, b) => a.sort_order - b.sort_order);
          setImages(sorted);
          // Find index of primary image
          const primaryIdx = sorted.findIndex((img) => img.is_primary);
          setCurrentImageIndex(primaryIdx >= 0 ? primaryIdx : 0);
        }
      } catch (err) {
        if (!cancelled) {
          console.error(`[SwipeDeckCard] Failed to fetch images for ${pet.pet_uuid}:`, err);
        }
      }
    };
    fetchImages();
    return () => { cancelled = true; };
  }, [pet.pet_uuid]);

  const translationX = useSharedValue(0);
  const translationY = useSharedValue(0);
  const isAnimating = useSharedValue(false);
  const entryScale = useSharedValue(0.95);
  const entryOpacity = useSharedValue(0);

  useEffect(() => {
    const delay = stackIndex * 80;
    entryScale.value = withDelay(delay, withSpring(1, { damping: 12, stiffness: 200 }));
    entryOpacity.value = withDelay(delay, withTiming(1, { duration: 300 }));
  }, []);

  const goToPrevImage = useCallback(() => {
    setCurrentImageIndex((i) => (i === 0 ? images.length - 1 : i - 1));
  }, [images.length]);

  const goToNextImage = useCallback(() => {
    setCurrentImageIndex((i) => (i === images.length - 1 ? 0 : i + 1));
  }, [images.length]);

  const handleImageTap = useCallback((e: any) => {
    if (imagesLengthRef.current <= 1) return;
    // If the card moved, the swipe gesture consumed this touch
    if (Math.abs(translationX.value) > 3) return;
    const screenWidth = Dimensions.get('window').width;
    const x = e.nativeEvent.pageX;
    if (x < screenWidth * 0.35) {
      goToPrevImage();
    } else if (x > screenWidth * 0.65) {
      goToNextImage();
    }
  }, [goToPrevImage, goToNextImage, translationX]);

  const ageStr = pet.date_of_birth ? getAgeFromDob(pet.date_of_birth) : '';
  const gradient = getGradientForPet(petIndex);

  const currentImage = images[currentImageIndex] ?? pet.primary_image;

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

  const jsOnLike = useCallback(() => {
    onLike();
  }, [onLike]);

  const jsOnDislike = useCallback(() => {
    onDislike();
  }, [onDislike]);

  const panGesture = Gesture.Pan()
    .minDistance(5)
    .onUpdate((e) => {
      if (isTopCard) {
        translationX.value = e.translationX;
        translationY.value = e.translationY;
      }
    })
    .onFinalize((e) => {
      if (!isTopCard || isAnimating.value) return;
      const tx = e.translationX;
      const vx = e.velocityX;
      if (tx > SWIPE_THRESHOLD || (tx > 50 && vx > 1000)) {
        isAnimating.value = true;
        translationX.value = withSequence(
          withTiming(9999, { duration: 300 }),
          withSpring(0)
        );
        translationY.value = withSpring(0);
        runOnJS(jsOnLike)();
      } else if (tx < -SWIPE_THRESHOLD || (tx < -50 && vx < -1000)) {
        isAnimating.value = true;
        translationX.value = withSequence(
          withTiming(-9999, { duration: 300 }),
          withSpring(0)
        );
        translationY.value = withSpring(0);
        runOnJS(jsOnDislike)();
      } else {
        translationX.value = withSpring(0);
        translationY.value = withSpring(0);
      }
    });

  const cardStyle = useAnimatedStyle(() => {
    if (!isTopCard) {
      const scale = (1 - stackIndex * 0.05) * entryScale.value;
      const translateY = -stackIndex * 8;
      return {
        transform: [{ scale }, { translateY }],
        opacity: entryOpacity.value,
        zIndex: 3 - stackIndex,
        ...shadowStyle,
      };
    }
    const rotation = interpolate(translationX.value, [-200, 0, 200], [-MAX_ROTATION, 0, MAX_ROTATION]);
    return {
      transform: [
        { translateX: translationX.value },
        { translateY: translationY.value },
        { rotate: `${rotation}rad` },
      ],
      opacity: entryOpacity.value,
      zIndex: 3,
      ...shadowStyleTop,
    };
  }, [isTopCard, stackIndex]);

  const likeStampStyle = useAnimatedStyle(() => {
    if (!isTopCard) return { opacity: 0 };
    const opacity = interpolate(translationX.value, [STAMP_THRESHOLD, SWIPE_THRESHOLD], [0, 1], 'clamp');
    return { opacity };
  }, [isTopCard]);

  const nopeStampStyle = useAnimatedStyle(() => {
    if (!isTopCard) return { opacity: 0 };
    const opacity = interpolate(translationX.value, [-STAMP_THRESHOLD, -SWIPE_THRESHOLD], [0, 1], 'clamp');
    return { opacity };
  }, [isTopCard]);

  const Content = (
    <View style={[styles.container, { backgroundColor: colors.background }]}>
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
              style={[styles.tabButton, activeTab === tab ? styles.tabActive : styles.tabInactive]}
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

      <View style={styles.photoArea}>
        {currentImage ? (
          <AuthenticatedImage
            imageUuid={currentImage.uuid}
            variant="medium"
            style={[styles.gradientBackground, { backgroundColor: gradient[0] }]}
            pointerEvents="none"
          />
        ) : (
          <View style={[styles.gradientBackground, { backgroundColor: gradient[0] }]}>
            <View style={[styles.gradientOverlay, { backgroundColor: gradient[1] }]} />
            <View style={styles.petEmojiContainer}>
              <Text style={styles.petEmoji}>
                {pet.species.toLowerCase().includes('dog') ? '🐕' : pet.species.toLowerCase().includes('cat') ? '🐈' : '🐾'}
              </Text>
            </View>
          </View>
        )}

        {/* Tap overlay for image navigation */}
        {imagesLengthRef.current > 1 && (
          <TouchableOpacity
            style={styles.tapOverlay}
            activeOpacity={0}
            onPress={handleImageTap}
          />
        )}

        <View style={styles.dotsContainer}>
          {images.map((_, idx) => (
            <View
              key={idx}
              style={{
                width: idx === currentImageIndex ? 20 : 6,
                height: 4,
                borderRadius: 2,
                marginHorizontal: 2,
                backgroundColor: idx === currentImageIndex ? '#fff' : 'rgba(255,255,255,0.4)',
              }}
            />
          ))}
        </View>

        <View style={styles.infoOverlay}>
          {pet.owner && (
            <View style={styles.ownerRow}>
              <View style={styles.ownerAvatar}>
                <Ionicons name="person" size={18} color="#fff" />
              </View>
              <Text style={styles.ownerName}>
                {pet.owner.display_name || 'Unknown'}
              </Text>
            </View>
          )}

          <View style={styles.pillsContainer}>
            {pills.map((pill, idx) => (
              <View key={idx} style={styles.pill}>
                <Ionicons name={pill.icon as any} size={14} color="#fff" />
                <Text style={styles.pillText}>{pill.label}</Text>
              </View>
            ))}
          </View>

          <View style={styles.nameRow}>
            <Text style={styles.nameText}>{pet.name}</Text>
            {ageStr && <Text style={styles.ageText}>{ageStr}</Text>}
            <TouchableOpacity style={styles.arrowButton} onPress={onFullProfile}>
              <Ionicons name="arrow-up" size={20} color="#fff" />
            </TouchableOpacity>
          </View>

          <View style={styles.actionRow}>
            <TouchableOpacity style={[styles.actionButton, styles.actionSmall]} onPress={() => {}}>
              <Ionicons name="refresh" size={22} color="#aaa" />
            </TouchableOpacity>
            <TouchableOpacity style={[styles.actionButton, styles.actionMedium]} onPress={jsOnDislike}>
              <Ionicons name="close" size={30} color="#ff4458" />
            </TouchableOpacity>
            <TouchableOpacity style={[styles.actionButton, styles.actionSmall]} onPress={() => {}}>
              <Ionicons name="star" size={22} color="#2196f3" />
            </TouchableOpacity>
            <TouchableOpacity style={[styles.actionButton, styles.actionMedium]} onPress={jsOnLike}>
              <Ionicons name="heart" size={30} color="#4ade80" />
            </TouchableOpacity>
            <TouchableOpacity style={[styles.actionButton, styles.actionSmall]} onPress={onFullProfile}>
              <Ionicons name="send" size={22} color="#3b82f6" />
            </TouchableOpacity>
          </View>
        </View>
      </View>
    </View>
  );

  if (isTopCard) {
    return (
      <GestureDetector gesture={panGesture}>
        <Animated.View style={[styles.cardWrapper, cardStyle]}>
          {Content}
          <Animated.Text style={[styles.stamp, styles.likeStamp, likeStampStyle]}>LIKE</Animated.Text>
          <Animated.Text style={[styles.stamp, styles.nopeStamp, nopeStampStyle]}>NOPE</Animated.Text>
        </Animated.View>
      </GestureDetector>
    );
  }

  return (
    <Animated.View style={[styles.cardWrapper, cardStyle]}>
      {Content}
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  cardWrapper: {
    ...StyleSheet.absoluteFill,
    overflow: 'hidden',
    borderRadius: 16,
  },
  container: {
    flex: 1,
    borderRadius: 16,
    overflow: 'hidden',
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
    paddingTop: Platform.OS === 'ios' ? 48 : 32,
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
  tapOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    zIndex: 5,
  },
  gradientBackground: {
    width: '100%',
    height: '100%',
  },
  gradientOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    opacity: 0.5,
  },
  petEmojiContainer: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
  },
  petEmoji: {
    fontSize: 120,
  },
  dotsContainer: {
    position: 'absolute',
    top: Platform.OS === 'ios' ? 100 : 84,
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
    zIndex: 6,
  },
  ownerRow: {
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 8,
  },
  ownerAvatar: {
    width: 32,
    height: 32,
    borderRadius: 16,
    backgroundColor: 'rgba(0,0,0,0.4)',
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 8,
  },
  ownerName: {
    fontSize: 14,
    fontWeight: '600',
    color: '#fff',
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
  stamp: {
    position: 'absolute',
    top: 120,
    fontSize: 48,
    fontWeight: '900',
    letterSpacing: 2,
    padding: 8,
    borderWidth: 4,
    borderRadius: 8,
  },
  likeStamp: {
    right: 30,
    color: '#4ade80',
    borderColor: '#4ade80',
    transform: [{ rotate: '15deg' }],
  },
  nopeStamp: {
    left: 30,
    color: '#ff4458',
    borderColor: '#ff4458',
    transform: [{ rotate: '-15deg' }],
  },
});
