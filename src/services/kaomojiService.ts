import type { Kaomoji } from '../types/clipboard'

import kaomojisData from '../data/kaomojis.json'

export const KAOMOJI_CATEGORIES = [
  'Happy',
  'Sad',
  'Angry',
  'Love',
  'Confused',
  'Surprised',
  'Action',
  'Greetings',
  'Sleeping',
  'Apology',
  'Magic',
  'Dancing',
  'Food',
  'Music',
  'Study',
  'Cats',
  'Bears',
  'Dogs',
  'Rabbits',
  'Friends',
  'Skeptical',
  'Cute',
  'Shy',
  'Sympathy',
] as const

export type KaomojiCategory = (typeof KAOMOJI_CATEGORIES)[number]

export const KAOMOJI_LIST: Kaomoji[] = kaomojisData as Kaomoji[]

export function getKaomojis(
  category?: string | null,
  search?: string,
  customList: Kaomoji[] = []
): Kaomoji[] {
  // Merge static list with custom list (custom first or last? let's do last but clearly categorized)
  // Ensure custom items map to the Kaomoji interface if needed, but they should match.
  let list = [...customList, ...KAOMOJI_LIST]

  if (category && category !== 'All') {
    list = list.filter((k) => k.category === category)
  }

  if (search) {
    const term = search.toLowerCase()
    list = list.filter(
      (k) =>
        k.text.toLowerCase().includes(term) ||
        k.keywords.some((key) => key.includes(term)) ||
        k.category.toLowerCase().includes(term)
    )
  }

  return list
}
