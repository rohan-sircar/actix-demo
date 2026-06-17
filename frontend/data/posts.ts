export interface Post {
  message: string;
  userId: number;
  likes: number;
  replies: number;
}

const POSTS: Post[] = [
  {
    message:
      'Concept: bioengineering single-serving sizes of produce.\n\nI simply cannot finish an entire napa cabbage on my own!!!',
    userId: 0,
    likes: 8,
    replies: 2,
  },
  {
    message: "Excited to announce that I'll be speaking at the upcoming conference!",
    userId: 1,
    likes: 22,
    replies: 5,
  },
  {
    message: 'Enjoying a cup of coffee and watching the sunrise. Perfect way to start the day!',
    userId: 2,
    likes: 35,
    replies: 8,
  },
  {
    message: 'Trying out a new recipe tonight. Fingers crossed it turns out delicious!',
    userId: 3,
    likes: 9,
    replies: 1,
  },
  {
    message: 'Exploring a new city and falling in love with its charm.',
    userId: 5,
    likes: 23,
    replies: 5,
  },
  {
    message: 'Enjoying a lazy Sunday with a good book and a cup of tea. Perfect relaxation!',
    userId: 3,
    likes: 24,
    replies: 7,
  },
  {
    message: "Enjoying a beautiful day at the beach. Life's a beach!",
    userId: 6,
    likes: 26,
    replies: 6,
  },
  {
    message: 'Exploring a hidden gem in the city. So much to discover!',
    userId: 7,
    likes: 18,
    replies: 4,
  },
  {
    message: 'Finished reading a great book today. Highly recommended!',
    userId: 2,
    likes: 14,
    replies: 3,
  },
  {
    message: "Can't wait for the weekend! Any fun plans?",
    userId: 1,
    likes: 6,
    replies: 1,
  },
  {
    message: 'Exploring the wonders of nature on a hiking trip. Loving every moment!',
    userId: 3,
    likes: 29,
    replies: 6,
  },
  {
    message: 'Spending quality time with family and creating precious memories.',
    userId: 6,
    likes: 48,
    replies: 11,
  },
  {
    message: 'Heading to the airport. See you all in 2 weeks!',
    userId: 0,
    likes: 12,
    replies: 0,
  },
  {
    message: 'Attending a fascinating workshop on personal growth. Learning so much!',
    userId: 8,
    likes: 22,
    replies: 5,
  },
  {
    message: 'Feeling proud of my recent accomplishments. Hard work pays off!',
    userId: 7,
    likes: 25,
    replies: 6,
  },
  {
    message: 'Having a fantastic time exploring the beautiful city of Paris!',
    userId: 2,
    likes: 27,
    replies: 6,
  },
  {
    message: 'Celebrating my birthday today! Grateful for another year of adventures.',
    userId: 4,
    likes: 42,
    replies: 8,
  },
  {
    message: "Just started learning a new programming language. It's challenging but exciting!",
    userId: 2,
    likes: 19,
    replies: 4,
  },
  {
    message: 'Just launched my new website! Check it out at www.example.com',
    userId: 1,
    likes: 15,
    replies: 4,
  },
  {
    message: 'New blog post: 10 Tips for Productivity',
    userId: 2,
    likes: 10,
    replies: 2,
  },
  {
    message: 'Just finished a marathon! Proud of myself for pushing through!',
    userId: 4,
    likes: 61,
    replies: 15,
  },
  {
    message: "Just launched a new business venture. Excited for what's to come!",
    userId: 6,
    likes: 20,
    replies: 4,
  },
  {
    message:
      'I am in Pittsburgh this week. Hit me up if you’re around. I’ll be at La Prima Espresso.',
    userId: 0,
    likes: 0,
    replies: 3,
  },
  {
    message: 'Just finished a painting. Art is such a therapeutic outlet!',
    userId: 5,
    likes: 17,
    replies: 4,
  },
  {
    message: 'Spent the day volunteering at the local shelter. Such a rewarding experience!',
    userId: 3,
    likes: 43,
    replies: 9,
  },
  {
    message: "Feeling inspired by the beautiful sunset. Nature's artwork!",
    userId: 5,
    likes: 11,
    replies: 2,
  },
  {
    message: 'New blog post: 5 Tips for Better Time Management',
    userId: 6,
    likes: 14,
    replies: 3,
  },
  {
    message: 'Relaxing in a hammock and enjoying the sound of waves. Paradise!',
    userId: 8,
    likes: 36,
    replies: 9,
  },
  {
    message: 'New blog post: How to Stay Motivated on Your Fitness Journey',
    userId: 5,
    likes: 13,
    replies: 3,
  },
  {
    message: 'Feeling grateful for all the amazing opportunities that have come my way!',
    userId: 4,
    likes: 37,
    replies: 10,
  },
  {
    message: 'Just finished a challenging workout. Feeling accomplished!',
    userId: 3,
    likes: 16,
    replies: 3,
  },
  {
    message: 'Just watched an incredible movie. It left me speechless!',
    userId: 3,
    likes: 12,
    replies: 3,
  },
  {
    message: 'Enjoying a cup of hot chocolate by the fireplace. Cozy vibes!',
    userId: 5,
    likes: 28,
    replies: 6,
  },
  {
    message: 'Just adopted a new puppy. Meet Max!',
    userId: 1,
    likes: 55,
    replies: 13,
  },
  {
    message: 'Attended an inspiring conference today. So many great ideas to implement!',
    userId: 3,
    likes: 18,
    replies: 4,
  },
  {
    message:
      "Reflecting on the past year and setting goals for the future. Excited for what's to come!",
    userId: 3,
    likes: 21,
    replies: 5,
  },
  {
    message: 'Just tried a new restaurant and it exceeded all expectations. Yum!',
    userId: 7,
    likes: 9,
    replies: 2,
  },
  {
    message: 'Celebrating a milestone today. Cheers to new beginnings!',
    userId: 7,
    likes: 32,
    replies: 7,
  },
  {
    message: 'Feeling grateful for the amazing friends in my life.',
    userId: 1,
    likes: 32,
    replies: 7,
  },
  {
    message: 'Attended an exciting concert last night. The energy was incredible!',
    userId: 5,
    likes: 33,
    replies: 7,
  },
  {
    message: "Missing my favorite band's concert tonight. Hoping they'll come back soon!",
    userId: 2,
    likes: 7,
    replies: 2,
  },
];

export default POSTS;
