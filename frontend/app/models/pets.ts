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
