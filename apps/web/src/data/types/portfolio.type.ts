/* @layer renderer-app @kind types */
import type { RunRow } from '../runs';

interface CardCounts {
  threads: number;
  working: number;
  blocked: number;
  needsYou: number;
  itemsOpen: number;
  doing?: number;
  review?: number;
  testsPass?: number;
  tests?: number;
  testsFailing?: number;
  agents?: number;
  paused?: number;
}

interface CardCoordinator {
  session: string | null;
  link: string | null;
  lastSeen: string | null;
  activity: string | null;
  note: string | null;
}

interface CardMaster {
  name: string;
  title: string | null;
  cmd: string | null;
  current: boolean;
  window: boolean;
  play: string | null;
  updatedAt: string | null;
  by: string | null;
  lastRun?: RunRow | null;
}

interface OwnerItem {
  id: string;
  title: string;
  type: string;
  stage: string;
}

interface Card {
  key: string;
  name: string;
  state: string;
  root: string | null;
  url: string;
  palette?: string;
  counts?: CardCounts;
  ownerItems?: OwnerItem[];
  coordinator?: CardCoordinator | null;
  attention?: string[];
  master?: CardMaster | null;
  masterMissing?: boolean;
  updatedAt?: string;
}

export type { Card, OwnerItem };
