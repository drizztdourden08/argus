/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { useBoardList } from './behavior/useBoardList';
import { useBoardWarm } from './behavior/useBoardWarm';
import { useSwitchBoard } from './behavior/useSwitchBoard';
import type { BoardSwitcherProps } from './BoardSwitcher.type';
import './BoardSwitcher.css';

const BoardSwitcher = (props: BoardSwitcherProps) => {
  const { boardKey } = props;
  const { cards, options } = useBoardList();
  const [warm, setWarm] = useState<string | null>(null);
  const switchBoard = useSwitchBoard();
  useBoardWarm(warm);
  const onHint = useCallback((hint: { label: string } | null) => setWarm(cards.find((card) => card.name === hint?.label)?.key ?? null), [cards]);
  if (options.length === 0) return null;
  return (
    <SegmentedControl aria-label="Board" size="sm" selectedTone="neutral" options={options} value={boardKey ?? ''} onChange={switchBoard} onHint={onHint} />
  );
};

export { BoardSwitcher };
