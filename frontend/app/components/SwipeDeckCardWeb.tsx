import React, { useCallback, useRef, useState, useEffect } from 'react';
import { Ionicons } from '@expo/vector-icons';
import type { PublicPet } from '~/app/models/pets';
import { getSpeciesIcon, getAgeFromDob } from '~/app/pet-view/gallery-utils';
import { getImageUrl } from '~/app/lib/api';

const SWIPE_THRESHOLD = 100;
const STAMP_THRESHOLD = 75;
const MAX_ROTATION = 15;

const GRADIENTS = [
  ['#FF6B6B', '#FF8E53'],
  ['#4ECDC4', '#44B09E'],
  ['#667EEA', '#764BA2'],
  ['#F093FB', '#F5576C'],
  ['#4FACFE', '#00F2FE'],
  ['#43E97B', '#38F9D7'],
  ['#FA709A', '#FEE140'],
  ['#A18D1', '#FBC2EB'],
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
}

const TABS = ['All', 'Dogs', 'Cats', 'Birds'];

const getGradientForPet = (petIndex: number) => GRADIENTS[petIndex % GRADIENTS.length];

export function SwipeDeckCardWeb({
  pet,
  colors,
  accentSet,
  isDarkColorScheme,
  onLike,
  onDislike,
  onFullProfile,
  isTopCard,
  stackIndex,
}: Props) {
  const [activeTab, setActiveTab] = useState('All');
  const [isDragging, setIsDragging] = useState(false);
  const [translateX, setTranslateX] = useState(0);
  const [translateY, setTranslateY] = useState(0);
  const [isFlying, setIsFlying] = useState<'like' | 'dislike' | null>(null);
  const [entryScale, setEntryScale] = useState(0.95);
  const [entryOpacity, setEntryOpacity] = useState(0);
  const cardRef = useRef<HTMLDivElement>(null);
  const dragStart = useRef({ x: 0, y: 0 });

  const ageStr = pet.date_of_birth ? getAgeFromDob(pet.date_of_birth) : '';
  const gradient = getGradientForPet(pet.id);

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

  // Entry animation
  useEffect(() => {
    const delay = stackIndex * 80;
    const timer = setTimeout(() => {
      setEntryScale(1 - stackIndex * 0.05);
      setEntryOpacity(1);
    }, delay);
    return () => clearTimeout(timer);
  }, []);

  const handleLike = useCallback(() => {
    if (isFlying) return;
    setIsFlying('like');
    setTimeout(() => {
      onLike();
      setIsFlying(null);
      setTranslateX(0);
      setTranslateY(0);
    }, 350);
  }, [isFlying, onLike]);

  const handleDislike = useCallback(() => {
    if (isFlying) return;
    setIsFlying('dislike');
    setTimeout(() => {
      onDislike();
      setIsFlying(null);
      setTranslateX(0);
      setTranslateY(0);
    }, 350);
  }, [isFlying, onDislike]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!isTopCard || isFlying) return;
    setIsDragging(true);
    dragStart.current = { x: e.clientX, y: e.clientY };
    e.preventDefault();
  }, [isTopCard, isFlying]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDragging) return;
    const dx = e.clientX - dragStart.current.x;
    const dy = e.clientY - dragStart.current.y;
    setTranslateX(dx);
    setTranslateY(dy * 0.3);
  }, [isDragging]);

  const handleMouseUp = useCallback(() => {
    if (!isDragging) return;
    setIsDragging(false);

    if (translateX > SWIPE_THRESHOLD) {
      handleLike();
    } else if (translateX < -SWIPE_THRESHOLD) {
      handleDislike();
    } else {
      setTranslateX(0);
      setTranslateY(0);
    }
  }, [isDragging, translateX, handleLike, handleDislike]);

  // Keyboard shortcuts
  useEffect(() => {
    if (!isTopCard) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight') {
        e.preventDefault();
        handleLike();
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault();
        handleDislike();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isTopCard, handleLike, handleDislike]);

  // Compute styles
  const rotation = isTopCard ? (translateX / SWIPE_THRESHOLD) * MAX_ROTATION : 0;
  const stampOpacity = Math.max(0, Math.min(1, (Math.abs(translateX) - STAMP_THRESHOLD) / (SWIPE_THRESHOLD - STAMP_THRESHOLD)));

  const cardTransform = isFlying
    ? `translateX(${isFlying === 'like' ? 9999 : -9999}px) rotate(${isFlying === 'like' ? 30 : -30}deg)`
    : isTopCard
      ? `translateX(${translateX}px) translateY(${translateY}px) rotate(${rotation}deg)`
      : `scale(${entryScale}) translateY(${-stackIndex * 8}px)`;

  const cardTransition = isFlying ? 'transform 300ms ease-out' : isDragging ? 'none' : 'transform 300ms cubic-bezier(0.175, 0.885, 0.32, 1.275)';

  const shadowStyle = isTopCard
    ? { boxShadow: '0 8px 32px rgba(0,0,0,0.25)' }
    : { boxShadow: '0 4px 16px rgba(0,0,0,0.15)' };

  const containerStyle: React.CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: 'hidden',
    borderRadius: 16,
    transform: cardTransform,
    transition: cardTransition,
    opacity: entryOpacity,
    zIndex: isTopCard ? 3 : 3 - stackIndex,
    cursor: isTopCard && !isFlying ? (isDragging ? 'grabbing' : 'grab') : 'default',
    userSelect: isTopCard && !isFlying ? 'none' : 'auto',
    ...shadowStyle,
  };

  const imageUrl = pet.primary_image
    ? getImageUrl(pet.primary_image.uuid, 'medium')
    : undefined;

  return (
    <div style={containerStyle}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onMouseLeave={() => {
        if (isDragging && !isFlying) {
          setIsDragging(false);
          setTranslateX(0);
          setTranslateY(0);
        }
      }}
      ref={cardRef}
    >
      {/* Card content */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', height: '100%', borderRadius: 16, overflow: 'hidden', backgroundColor: colors.background }}>
        {/* Top bar */}
        <div style={{ position: 'absolute', top: 0, left: 0, right: 0, zIndex: 10, display: 'flex', alignItems: 'center', paddingLeft: 16, paddingRight: 16, paddingTop: 48, paddingBottom: 8, backgroundColor: 'rgba(0,0,0,0.3)' }}>
          <button style={{ width: 40, height: 40, borderRadius: 20, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.4)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="filter" size={24} color="#fff" />
          </button>
          <div style={{ flex: 1, display: 'flex', flexDirection: 'row', gap: 8, marginLeft: 12, marginRight: 12, overflowX: 'auto' }}>
            {TABS.map((tab) => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                style={{
                  padding: '6px 16px',
                  borderRadius: 20,
                  border: 'none',
                  cursor: 'pointer',
                  whiteSpace: 'nowrap',
                  backgroundColor: activeTab === tab ? 'rgba(255,255,255,0.95)' : 'rgba(0,0,0,0.3)',
                }}
              >
                <span style={{ fontSize: 14, fontWeight: 600, color: activeTab === tab ? '#000' : 'rgba(255,255,255,0.7)' }}>
                  {tab}
                </span>
              </button>
            ))}
          </div>
          <button style={{ width: 40, height: 40, borderRadius: 20, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.4)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="flash" size={24} color="#fff" />
          </button>
        </div>

        {/* Photo area */}
        <div style={{ flex: 1, position: 'relative' }}>
          {imageUrl ? (
            <img
              src={imageUrl}
              alt={pet.name}
              style={{ width: '100%', height: '100%', objectFit: 'cover' }}
            />
          ) : (
            <div style={{ width: '100%', height: '100%', backgroundColor: '#e2e8f0' }}>
              <div style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                <span style={{ fontSize: 120 }}>
                  {pet.species.toLowerCase().includes('dog') ? '🐕' : pet.species.toLowerCase().includes('cat') ? '🐈' : '🐾'}
                </span>
              </div>
            </div>
          )}

          {/* Gradient overlay */}
          <div style={{ position: 'absolute', top: 0, left: 0, right: 0, bottom: 0, background: 'linear-gradient(to bottom, transparent 50%, rgba(0,0,0,0.6))' }} />

          {/* Page dots */}
          <div style={{ position: 'absolute', top: 100, left: 0, right: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', zIndex: 6 }}>
            <div style={{ width: 20, height: 4, borderRadius: 2, marginLeft: 2, marginRight: 2, backgroundColor: '#fff' }} />
          </div>

          {/* Info overlay */}
          <div style={{ position: 'absolute', bottom: 0, left: 0, right: 0, padding: '20px 16px', zIndex: 6 }}>
            {/* Owner info */}
            {pet.owner && (
              <div style={{ display: 'flex', alignItems: 'center', marginBottom: 8 }}>
                <div style={{ width: 32, height: 32, borderRadius: 16, backgroundColor: 'rgba(255,255,255,0.3)', display: 'flex', alignItems: 'center', justifyContent: 'center', marginRight: 8 }}>
                  <Ionicons name="person" size={18} color="#fff" />
                </div>
                <span style={{ fontSize: 14, fontWeight: 600, color: '#fff' }}>
                  {pet.owner.display_name || 'Unknown'}
                </span>
              </div>
            )}

            {/* Trait pills */}
            <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 12 }}>
              {pills.map((pill, idx) => (
                <div key={idx} style={{ display: 'flex', alignItems: 'center', backgroundColor: 'rgba(0,0,0,0.5)', padding: '5px 10px', borderRadius: 20, gap: 4 }}>
                  <Ionicons name={pill.icon as any} size={14} color="#fff" />
                  <span style={{ fontSize: 12, fontWeight: 600, color: '#fff' }}>{pill.label}</span>
                </div>
              ))}
            </div>

            {/* Name + age */}
            <div style={{ display: 'flex', alignItems: 'flex-end', justifyContent: 'space-between', marginBottom: 16 }}>
              <div>
                <span style={{ fontSize: 32, fontWeight: 700, color: '#fff' }}>{pet.name}</span>
                {ageStr && <span style={{ fontSize: 32, fontWeight: 700, color: '#fff', marginLeft: 8 }}>{ageStr}</span>}
              </div>
              <button onClick={onFullProfile} style={{ width: 40, height: 40, borderRadius: 20, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.4)', border: 'none', cursor: 'pointer' }}>
                <Ionicons name="arrow-up" size={20} color="#fff" />
              </button>
            </div>

            {/* Action buttons */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 16, paddingBottom: 8 }}>
              <button style={{ width: 48, height: 48, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.3)', border: 'none', cursor: 'pointer' }}>
                <Ionicons name="refresh" size={22} color="#aaa" />
              </button>
              <button onClick={handleDislike} style={{ width: 60, height: 60, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.3)', border: '2px solid rgba(255,255,255,0.1)', cursor: 'pointer' }}>
                <Ionicons name="close" size={30} color="#ff4458" />
              </button>
              <button style={{ width: 48, height: 48, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.3)', border: 'none', cursor: 'pointer' }}>
                <Ionicons name="star" size={22} color="#2196f3" />
              </button>
              <button onClick={handleLike} style={{ width: 60, height: 60, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.3)', border: '2px solid rgba(255,255,255,0.1)', cursor: 'pointer' }}>
                <Ionicons name="heart" size={30} color="#4ade80" />
              </button>
              <button onClick={onFullProfile} style={{ width: 48, height: 48, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.3)', border: 'none', cursor: 'pointer' }}>
                <Ionicons name="send" size={22} color="#3b82f6" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* LIKE stamp */}
      <div style={{
        position: 'absolute',
        top: 120,
        right: 30,
        fontSize: 48,
        fontWeight: 900,
        letterSpacing: 2,
        padding: 8,
        borderWidth: 4,
        borderRadius: 8,
        color: '#4ade80',
        borderColor: '#4ade80',
        transform: 'rotate(15deg)',
        opacity: isTopCard ? stampOpacity : 0,
        pointerEvents: 'none',
      }}>
        LIKE
      </div>

      {/* NOPE stamp */}
      <div style={{
        position: 'absolute',
        top: 120,
        left: 30,
        fontSize: 48,
        fontWeight: 900,
        letterSpacing: 2,
        padding: 8,
        borderWidth: 4,
        borderRadius: 8,
        color: '#ff4458',
        borderColor: '#ff4458',
        transform: 'rotate(-15deg)',
        opacity: isTopCard ? stampOpacity : 0,
        pointerEvents: 'none',
      }}>
        NOPE
      </div>
    </div>
  );
}
