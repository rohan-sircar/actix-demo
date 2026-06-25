import React, { useCallback, useRef, useState } from 'react';
import { Ionicons } from '@expo/vector-icons';
import type { PublicPet, PetImage } from '~/app/models/pets';
import { getImageUrl, getAgeFromDob } from './gallery-utils';

interface Props {
  pet: PublicPet;
  images: PetImage[];
  colors: { background: string; text: string; grey: string };
  accentSet: { base: string };
  onFullProfile: () => void;
}

export function WebGallery({ pet, images, colors, accentSet, onFullProfile }: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [currentPage, setCurrentPage] = useState(0);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);

  const handleScroll = useCallback(() => {
    if (scrollRef.current) {
      const page = Math.round(scrollRef.current.scrollLeft / scrollRef.current.clientWidth);
      setCurrentPage(page);
    }
  }, []);

  const navigatePage = useCallback((dir: -1 | 1) => {
    if (scrollRef.current) {
      const target = (currentPage + dir) * scrollRef.current.clientWidth;
      scrollRef.current.scrollTo({ left: target, behavior: 'smooth' });
    }
  }, [currentPage]);

  const fallbackImages: PetImage[] = pet.primary_image ? [{ id: 0, uuid: pet.primary_image.uuid, format: 'jpeg', is_primary: true, sort_order: 0, created_at: '' }] : [];
  const allImages = images.length > 0 ? images : fallbackImages;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', backgroundColor: colors.background, overflow: 'hidden' }}>
      <div style={{ flex: 1, position: 'relative', overflow: 'hidden' }}>
        <div
          ref={scrollRef}
          onScroll={handleScroll}
          style={{
            height: '100%',
            overflowX: 'auto',
            overflowY: 'hidden',
            scrollSnapType: 'x mandatory',
            scrollBehavior: 'smooth',
            WebkitOverflowScrolling: 'touch',
          }}
        >
          <div style={{ display: 'flex', flexDirection: 'row', height: '100%' }}>
            {allImages.map((image) => (
              <div
                key={image.uuid}
                style={{ width: '100%', height: '100%', flexShrink: 0, scrollSnapAlign: 'start', cursor: 'pointer' }}
                onClick={() => setPreviewUrl(getImageUrl(image.uuid, 'original'))}
              >
                <img
                  src={getImageUrl(image.uuid, 'original')}
                  alt=""
                  style={{ width: '100%', height: '100%', objectFit: 'cover', pointerEvents: 'none', display: 'block' }}
                />
              </div>
            ))}
          </div>
        </div>

        {allImages.length > 1 && currentPage > 0 && (
          <button
            onClick={() => navigatePage(-1)}
            style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)', width: 40, height: 40, borderRadius: 20, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.4)', cursor: 'pointer', border: 'none' }}
          >
            <Ionicons name="chevron-back" size={24} color="#fff" />
          </button>
        )}

        {allImages.length > 1 && currentPage < allImages.length - 1 && (
          <button
            onClick={() => navigatePage(1)}
            style={{ position: 'absolute', right: 12, top: '50%', transform: 'translateY(-50%)', width: 40, height: 40, borderRadius: 20, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.4)', cursor: 'pointer', border: 'none' }}
          >
            <Ionicons name="chevron-forward" size={24} color="#fff" />
          </button>
        )}

        <div
          style={{
            position: 'absolute', bottom: 0, left: 0, right: 0,
            padding: '24px 16px 24px',
            background: `linear-gradient(to top, ${colors.background} 60%, transparent)`,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center' }}>
            <h2 style={{ fontSize: 24, fontWeight: 700, marginRight: 8, color: colors.text }}>
              {pet.name}
            </h2>
            {pet.date_of_birth && (
              <span style={{ fontSize: 12, padding: '2px 8px', borderRadius: 999, color: colors.grey, backgroundColor: 'rgba(128,128,128,0.15)' }}>
                {getAgeFromDob(pet.date_of_birth)}
              </span>
            )}
          </div>
          <p style={{ fontSize: 16, marginTop: 4, color: colors.grey }}>
            {pet.species}
            {pet.breed ? ` · ${pet.breed}` : ''}
          </p>

          {allImages.length > 1 && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', marginTop: 12, gap: 4 }}>
              {allImages.map((_, idx) => (
                <div
                  key={idx}
                  style={{
                    width: idx === currentPage ? 8 : 6,
                    height: idx === currentPage ? 8 : 6,
                    borderRadius: 999,
                    backgroundColor: idx === currentPage ? accentSet.base : `${colors.grey}80`,
                  }}
                />
              ))}
            </div>
          )}

          <button
            onClick={onFullProfile}
            style={{
              marginTop: 16, display: 'flex', flexDirection: 'row', alignItems: 'center', justifyContent: 'center',
              borderRadius: 12, padding: '14px 0', width: '100%',
              backgroundColor: accentSet.base,
              boxShadow: `0 4px 12px ${accentSet.base}4d`,
              border: 'none', cursor: 'pointer',
            }}
          >
            <Ionicons name="information-circle" size={20} color="#fff" />
            <span style={{ marginLeft: 8, fontSize: 16, fontWeight: 600, color: '#fff' }}>Full Profile</span>
          </button>
        </div>
      </div>

      {previewUrl && (
        <div
          style={{ position: 'fixed', inset: 0, zIndex: 50, display: 'flex', alignItems: 'center', justifyContent: 'center', backgroundColor: 'rgba(0,0,0,0.9)', cursor: 'zoom-out' }}
          onClick={() => setPreviewUrl(null)}
        >
          <img
            src={previewUrl}
            alt=""
            style={{ maxWidth: '90vw', maxHeight: '90vh', objectFit: 'contain' }}
          />
        </div>
      )}
    </div>
  );
}
