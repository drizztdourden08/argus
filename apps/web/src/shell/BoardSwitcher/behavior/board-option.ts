/* @layer renderer-app @kind logic */
import { createElement } from 'react';
import type { Card } from '../../../data/types';
import type { BoardOption } from '../BoardSwitcher.type';
import { BoardOptionIcon } from '../sub-components/BoardOptionIcon';

const runningText = (running: number): string => (running > 0 ? `${running} running` : 'idle');

const boardOption = (card: Card): BoardOption => ({
  value: card.key,
  icon: createElement(BoardOptionIcon, { card }),
  hint: { label: card.name, description: runningText(card.counts?.working ?? 0) },
});

export { boardOption };
