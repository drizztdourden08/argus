/* @layer renderer-app @kind types */
interface CardCounts {
  threads: number;
  working: number;
  blocked: number;
  needsYou: number;
  itemsOpen: number;
}

interface Card {
  key: string;
  name: string;
  state: string;
  root: string | null;
  url: string;
  palette?: string;
  counts?: CardCounts;
}

export type { Card };
