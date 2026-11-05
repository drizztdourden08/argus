/* @layer renderer-app @kind component */
import { BrandMark, Card, Flex, OverflowRow } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { BoardSwitcher } from '../BoardSwitcher';
import { LocalBanner } from '../LocalBanner';
import { ModeSwitch } from '../ModeSwitch';
import { SettingsEntries } from './sub-components/SettingsEntries';
import type { ShellBarProps } from './ShellBar.type';
import './ShellBar.css';

const ShellBar = (props: ShellBarProps) => {
  const { mode, boardKey, boardsAvailable, onMode } = props;
  const items: OverflowRowItem[] = [
    { id: 'mark', pinned: true, node: <BrandMark app="argus" size="sm" /> },
    { id: 'mode', pinned: true, node: <ModeSwitch mode={mode} boardsAvailable={boardsAvailable} onChange={onMode} /> },
    ...(mode === 'boards' ? [{ id: 'boards', priority: 5, node: <BoardSwitcher boardKey={boardKey} /> }] : []),
    { id: 'local', priority: 3, node: <LocalBanner /> },
    { id: 'owner', pinned: true, node: <SettingsEntries /> },
  ];
  return (
    <Card surface="glass" className="shell-bar">
      <Flex as="header" align="center">
        <OverflowRow items={items} />
      </Flex>
    </Card>
  );
};

export { ShellBar };
