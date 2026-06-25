export interface PetImage {
  id: number;
  uuid: string;
  format: string;
  is_primary: boolean;
  sort_order: number;
  created_at: string;
}

export interface Pet {
  id: number;
  pet_uuid: string;
  name: string;
  species: string;
  breed?: string | null;
  date_of_birth?: string | null;
  gender?: string | null;
  weight?: number | null;
  color_markings?: string | null;
  description?: string | null;
  traits?: string[];
  primary_image: PetImage | null;
}

export interface OwnerInfo {
  user_uuid: string;
  display_name: string | null;
  avatar_url: string | null;
  pets_owned: number;
}

export interface PublicPet {
  id: number;
  pet_uuid: string;
  name: string;
  species: string;
  breed?: string | null;
  date_of_birth?: string | null;
  gender?: string | null;
  weight?: number | null;
  color_markings?: string | null;
  description?: string | null;
  traits?: string[];
  primary_image: PetImage | null;
  owner: OwnerInfo;
}

export interface LikeRecord {
  like_id: number;
  user_uuid: string;
  pet_owner_uuid: string;
  pet_uuid: string;
  direction: 'like' | 'dislike';
  is_match: boolean;
  created_at: string;
}

export interface PaginatedResponse<T> {
  pets: T[];
  total_count: number;
}

export interface DiscoverQuery {
  species?: string;
  gender?: string;
  age_min?: number;
  age_max?: number;
  limit?: number;
  offset?: number;
}
