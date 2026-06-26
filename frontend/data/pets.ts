import type { PetTrait } from '~/app/models/pets';

export interface MockPet {
  id: string;
  name: string;
  species: string;
  breed: string;
  age: string;
  weight?: string;
  distance: string;
  description: string;
  traits: PetTrait[];
  gender: 'male' | 'female';
  imageUrl?: string;
}

const MOCK_PETS: MockPet[] = [
  {
    id: '1',
    name: 'Bella',
    species: 'Dog',
    breed: 'Golden Retriever',
    age: '2y 3m',
    weight: '28',
    distance: '0.5 mi',
    description: 'Friendly and loves long walks in the park. Great with kids and other dogs!',
    traits: [{ id: 1, name: 'Friendly' }, { id: 2, name: 'Active' }, { id: 3, name: 'Trained' }],
    gender: 'female',
  },
  {
    id: '2',
    name: 'Max',
    species: 'Dog',
    breed: 'French Bulldog',
    age: '1y 8m',
    weight: '12',
    distance: '1.2 mi',
    description: 'Playful pup who loves belly rubs and chasing balls.',
    traits: [{ id: 4, name: 'Playful' }, { id: 5, name: 'Cuddly' }],
    gender: 'male',
  },
  {
    id: '3',
    name: 'Luna',
    species: 'Cat',
    breed: 'Siamese',
    age: '3y 1m',
    distance: '0.8 mi',
    description: 'Elegant and affectionate. Loves window watching and sunny spots.',
    traits: [{ id: 6, name: 'Gentle' }, { id: 7, name: 'Independent' }, { id: 8, name: 'Vocal' }],
    gender: 'female',
  },
  {
    id: '4',
    name: 'Charlie',
    species: 'Dog',
    breed: 'Labrador',
    age: '4y 5m',
    weight: '30',
    distance: '2.0 mi',
    description: 'Loyal companion who excels at fetch. Looking for playdate buddies!',
    traits: [{ id: 9, name: 'Loyal' }, { id: 10, name: 'Swimmer' }, { id: 11, name: 'Foodie' }],
    gender: 'male',
  },
  {
    id: '5',
    name: 'Milo',
    species: 'Cat',
    breed: 'Orange Tabby',
    age: '1y 2m',
    distance: '0.3 mi',
    description: 'Adventurous kitty who loves exploring and climbing cat trees.',
    traits: [{ id: 12, name: 'Curious' }, { id: 13, name: 'Energetic' }, { id: 14, name: 'Affectionate' }],
    gender: 'male',
  },
  {
    id: '6',
    name: 'Daisy',
    species: 'Dog',
    breed: 'Corgi',
    age: '2y 7m',
    weight: '13',
    distance: '1.5 mi',
    description: 'Sploot enthusiast with the biggest smile. Loves hikes and treats.',
    traits: [{ id: 15, name: 'Cheerful' }, { id: 16, name: 'Hiker' }, { id: 17, name: 'Snack Lover' }],
    gender: 'female',
  },
  {
    id: '7',
    name: 'Oliver',
    species: 'Cat',
    breed: 'Maine Coon',
    age: '5y',
    weight: '8',
    distance: '3.1 mi',
    description: 'Gentle giant who enjoys lounging and gentle play sessions.',
    traits: [{ id: 18, name: 'Gentle' }, { id: 19, name: 'Fluffy' }, { id: 20, name: 'Calm' }],
    gender: 'male',
  },
  {
    id: '8',
    name: 'Coco',
    species: 'Dog',
    breed: 'Poodle',
    age: '1y 5m',
    distance: '0.7 mi',
    description: 'Smart and stylish! Loves agility courses and grooming sessions.',
    traits: [{ id: 21, name: 'Smart' }, { id: 22, name: 'Elegant' }, { id: 23, name: 'Athletic' }],
    gender: 'female',
  },
  {
    id: '9',
    name: 'Shadow',
    species: 'Cat',
    breed: 'British Shorthair',
    age: '2y 9m',
    distance: '1.8 mi',
    description: 'Mysterious and calm. Enjoys quiet afternoons and laser pointers.',
    traits: [{ id: 24, name: 'Calm' }, { id: 25, name: 'Mysterious' }, { id: 26, name: 'Laser Chaser' }],
    gender: 'male',
  },
  {
    id: '10',
    name: 'Ruby',
    species: 'Dog',
    breed: 'Beagle',
    age: '3y 4m',
    weight: '14',
    distance: '2.5 mi',
    description: 'Nose for adventure! Loves sniffing walks and meeting new friends.',
    traits: [{ id: 27, name: 'Adventurous' }, { id: 28, name: 'Sociable' }, { id: 29, name: 'Sniffer' }],
    gender: 'female',
  },
];

export default MOCK_PETS;
