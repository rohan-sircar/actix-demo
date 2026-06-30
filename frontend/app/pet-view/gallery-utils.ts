import type { PetImage } from '~/app/models/pets';
import { getImageUrl } from '~/app/lib/api';

export { getImageUrl };

export const getAgeFromDob = (dob: string) => {
  const birth = new Date(dob);
  const today = new Date();
  const ageYears = today.getFullYear() - birth.getFullYear();
  const ageMonths = today.getMonth() - birth.getMonth();
  if (ageYears > 0) {
    return `${ageYears}y${ageMonths > 0 ? ` ${ageMonths}m` : ''}`;
  }
  const ageDays = Math.floor((today.getTime() - birth.getTime()) / (1000 * 60 * 60 * 24));
  if (ageDays > 30) {
    return `${Math.floor(ageDays / 30)}m`;
  }
  return `${ageDays}d`;
};

export const getSpeciesIcon = (sp: string) => {
  const lower = sp.toLowerCase();
  if (lower.includes('dog')) return 'paw' as const;
  if (lower.includes('cat')) return 'paw' as const;
  if (lower.includes('bird')) return 'planet' as const;
  if (lower.includes('fish')) return 'water' as const;
  if (lower.includes('reptile') || lower.includes('snake') || lower.includes('lizard'))
    return 'leaf' as const;
  return 'paw' as const;
};

export { PetImage };
