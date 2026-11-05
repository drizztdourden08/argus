/* @layer renderer-app @kind test */
import type { BoardItem } from '../../src/data/boards';
import type { Card } from '../../src/data/types';
import { boardRecord } from '../../src/screens/overview/boards/model/board-record';
import type { BoardRecord } from '../../src/screens/overview/boards/model/board.type';

const NOW = Date.parse('2026-11-04T12:00:00.000Z');

const COUNTS = { threads: 0, working: 0, blocked: 0, needsYou: 0, itemsOpen: 0, done: 0 };

const cardAt = (key: string, now: number, extra: Partial<Card> = {}): Card => ({
  key,
  name: key,
  state: 'running',
  root: null,
  url: `/p/${key}/`,
  coordinator: { session: 's', link: null, lastSeen: new Date(now - 60000).toISOString(), activity: 'idle', note: null },
  ...extra,
});

const card = (key: string, extra: Partial<Card> = {}): Card => cardAt(key, NOW, extra);

const item = (key: string, extra: Partial<Card> | null = {}, doc: Partial<BoardItem['doc']> = {}): BoardItem => ({
  doc: { id: `BD-${key}`, key, ...doc },
  card: extra === null ? null : card(key, extra),
});

const record = (key: string, extra: Partial<Card> | null = {}, doc: Partial<BoardItem['doc']> = {}): BoardRecord => boardRecord(item(key, extra, doc), NOW);

export { COUNTS, NOW, card, cardAt, item, record };
