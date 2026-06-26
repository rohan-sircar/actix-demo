import React, { useCallback, useState } from 'react';
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
  onLike: () => void;
  onDislike: () => void;
}

const TABS = ['All', 'Dogs', 'Cats', 'Birds'];

export function WebGallery({ pet, images, colors, accentSet, isDarkColorScheme, onFullProfile, onLike, onDislike }: Props) {
  const [currentIndex, setCurrentIndex] = useState(0);
  const [activeTab, setActiveTab] = useState('All');

  const fallbackImages: PetImage[] = pet.primary_image
    ? [{ id: 0, uuid: pet.primary_image.uuid, format: 'jpeg', is_primary: true, sort_order: 0, created_at: '' }]
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
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', backgroundColor: colors.background }}>
      {/* Top navbar */}
      <div style={{ display: 'flex', alignItems: 'center', padding: '12px 16px', gap: 12, backgroundColor: isDarkColorScheme ? 'rgba(0,0,0,0.3)' : colors.card || '#fff', borderBottom: `1px solid ${colors.grey4 || 'rgba(128,128,128,0.15)'}` }}>
        <button style={{ width: 36, height: 36, borderRadius: 18, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(128,128,128,0.15)', border: 'none', cursor: 'pointer' }}>
          <Ionicons name="filter" size={20} color={colors.text} />
        </button>
        <div style={{ flex: 1, display: 'flex', flexDirection: 'row', gap: 6, overflowX: 'auto' }}>
          {TABS.map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                padding: '6px 14px',
                borderRadius: 20,
                border: 'none',
                cursor: 'pointer',
                whiteSpace: 'nowrap',
                backgroundColor: activeTab === tab ? accentSet.base : 'rgba(128,128,128,0.15)',
              }}
            >
              <span style={{ fontSize: 13, fontWeight: 600, color: activeTab === tab ? '#fff' : colors.text }}>
                {tab}
              </span>
            </button>
          ))}
        </div>
        <button style={{ width: 36, height: 36, borderRadius: 18, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(128,128,128,0.15)', border: 'none', cursor: 'pointer' }}>
          <Ionicons name="flash" size={20} color={colors.text} />
        </button>
      </div>

      {/* Photo area - fills remaining space */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', padding: 16, position: 'relative', minHeight: 0 }}>
        <div style={{ position: 'relative', width: '100%', maxWidth: 430, maxHeight: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          {currentImage && (
            <img
              src={getImageUrl(currentImage.uuid, 'medium')}
              alt=""
              style={{ maxWidth: '100%', maxHeight: '100%', objectFit: 'contain', display: 'block', borderRadius: 16 }}
            />
          )}

          {/* Left tap zone */}
          <div
            onClick={goToPrev}
            style={{ position: 'absolute', top: 0, left: 0, width: '50%', height: '100%', cursor: 'pointer' }}
          />

          {/* Right tap zone */}
          <div
            onClick={goToNext}
            style={{ position: 'absolute', top: 0, right: 0, width: '50%', height: '100%', cursor: 'pointer' }}
          />

          {/* Page dots */}
          {allImages.length > 1 && (
            <div style={{ position: 'absolute', bottom: 12, left: 0, right: 0, display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 4 }}>
              {allImages.map((_, idx) => (
                <div
                  key={idx}
                  style={{
                    width: idx === currentIndex ? 16 : 6,
                    height: 6,
                    borderRadius: 3,
                    backgroundColor: idx === currentIndex ? '#fff' : 'rgba(255,255,255,0.5)',
                    boxShadow: '0 1px 3px rgba(0,0,0,0.3)',
                  }}
                />
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Bottom bio + actions */}
      <div style={{ padding: '16px 20px', backgroundColor: isDarkColorScheme ? 'rgba(0,0,0,0.3)' : colors.card || '#fff', borderTop: `1px solid ${colors.grey4 || 'rgba(128,128,128,0.15)'}` }}>
        {/* Name + age */}
        <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'space-between', marginBottom: 10 }}>
          <div>
            <span style={{ fontSize: 24, fontWeight: 700, color: colors.text }}>{pet.name}</span>
            {ageStr && (
              <span style={{ fontSize: 16, fontWeight: 500, color: colors.grey, marginLeft: 8 }}>{ageStr}</span>
            )}
          </div>
        </div>

        {/* Trait pills */}
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 14 }}>
          {pills.map((pill, idx) => (
            <div key={idx} style={{ display: 'flex', alignItems: 'center', backgroundColor: 'rgba(128,128,128,0.12)', padding: '4px 10px', borderRadius: 20, gap: 4 }}>
              <Ionicons name={pill.icon as any} size={13} color={colors.grey} />
              <span style={{ fontSize: 12, fontWeight: 500, color: colors.text }}>{pill.label}</span>
            </div>
          ))}
        </div>

        {/* Action buttons */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 14 }}>
          <button style={{ width: 44, height: 44, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(128,128,128,0.1)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="refresh" size={20} color="#aaa" />
          </button>
          <button onClick={onDislike} style={{ width: 54, height: 54, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(255,68,88,0.1)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="close" size={28} color="#ff4458" />
          </button>
          <button style={{ width: 44, height: 44, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(33,150,243,0.1)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="star" size={20} color="#2196f3" />
          </button>
          <button onClick={onLike} style={{ width: 54, height: 54, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(74,222,128,0.1)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="heart" size={28} color="#4ade80" />
          </button>
          <button onClick={onFullProfile} style={{ width: 44, height: 44, borderRadius: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(59,130,246,0.1)', border: 'none', cursor: 'pointer' }}>
            <Ionicons name="send" size={20} color="#3b82f6" />
          </button>
        </div>
      </div>
    </div>
  );
}
