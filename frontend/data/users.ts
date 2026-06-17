export interface User {
  userId: number;
  name: string;
  handle: string;
  about: string;
  followers: number;
  following: number;
  color: string;
  avatar: any | undefined;
}

const USERS: User[] = [
  {
    userId: 0,
    name: 'John Doe',
    handle: 'johndoe',
    about: 'Coding, hiking, and photography.',
    followers: 34,
    following: 18,
    color: '#4CAF50',
    avatar: undefined,
  },
  {
    userId: 1,
    name: 'Natalia Rodman',
    handle: 'natrod',
    about: 'Developing, herding cats, and eating avocados.',
    followers: 17,
    following: 11,
    color: '#FFB84C',
    avatar: require('../assets/avatar.png'),
  },
  {
    userId: 2,
    name: 'Elijah Summers',
    handle: 'elisum',
    about: 'Passionate coder, coffee addict, and book lover.',
    followers: 312,
    following: 248,
    color: '#2CD3E1',
    avatar: undefined,
  },
  {
    userId: 3,
    name: 'Olivia Johnson',
    handle: 'olijoh',
    about: 'UI/UX designer, nature enthusiast, and yoga practitioner.',
    followers: 754,
    following: 612,
    color: '#F266AB',
    avatar: undefined,
  },
  {
    userId: 4,
    name: 'Maxwell Foster',
    handle: 'maxfos',
    about: 'Frontend developer, music lover, and travel junkie.',
    followers: 521,
    following: 398,
    color: '#F45050',
    avatar: undefined,
  },
  {
    userId: 5,
    name: 'Sophia Ruiz',
    handle: 'sopruiz',
    about: 'Full-stack developer, foodie, and dog person.',
    followers: 1256,
    following: 873,
    color: '#A459D1',
    avatar: undefined,
  },
  {
    userId: 6,
    name: 'Sebastian Lee',
    handle: 'sebalee',
    about: 'Software engineer, gamer, and anime enthusiast.',
    followers: 981,
    following: 789,
    color: '#00AF91',
    avatar: undefined,
  },
  {
    userId: 7,
    name: 'Ava Chen',
    handle: 'avache',
    about: 'Data scientist, fitness enthusiast, and beach lover.',
    followers: 412,
    following: 268,
    color: '#007965',
    avatar: undefined,
  },
  {
    userId: 8,
    name: 'Ethan Watson',
    handle: 'ethwat',
    about: 'Backend developer, coffee connoisseur, and soccer fan.',
    followers: 672,
    following: 521,
    color: '#FF6000',
    avatar: undefined,
  },
  {
    userId: 9,
    name: 'Isabella Liu',
    handle: 'isaliu',
    about: 'Mobile app developer, photography lover, and hiker.',
    followers: 1037,
    following: 846,
    color: '#3DB2FF',
    avatar: undefined,
  },
];

export default USERS;
