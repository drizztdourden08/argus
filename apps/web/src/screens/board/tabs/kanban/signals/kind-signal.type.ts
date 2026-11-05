/* @layer renderer-app @kind types */
import type { CardData } from '../model';
import type { CardSignal } from './signal.type';

type SignalMaker = (card: CardData, now: number) => CardSignal | CardSignal[] | null;

export type { SignalMaker };
