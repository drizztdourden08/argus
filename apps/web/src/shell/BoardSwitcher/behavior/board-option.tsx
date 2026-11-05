/* @layer renderer-app @kind component */
import type { Card } from '../../../data/types';
import { resolvePalette } from '../../../palette';
import { BoardIcon } from '../../BoardIcon';
import type { BoardOption } from '../BoardSwitcher.type';

const runningText = (running: number): string => (running > 0 ? `${running} running` : 'idle');

const boardOption = (card: Card): BoardOption => {
  const running = card.counts?.working ?? 0;
  return {
    value: card.key,
    icon: (
      <span className="board-switcher__icon" data-palette={resolvePalette(card.key, card.palette)}>
        <BoardIcon boardKey={card.key} cardIcon={card.icon} size="sm" />
      </span>
    ),
    hint: { label: card.name, description: runningText(running) },
  };
};

export { boardOption };
