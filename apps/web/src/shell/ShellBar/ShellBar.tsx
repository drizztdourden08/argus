/* @layer renderer-app @kind component */
import { Card, Flex, OverflowRow } from '@drizztdourden08/tessera';
import type { OverflowRowItem } from '@drizztdourden08/tessera';
import { BoardSwitcher } from '../BoardSwitcher';
import { LocalBanner, useLocalBanner } from '../LocalBanner';
import { ModeSwitch } from '../ModeSwitch';
import { OverlordPresence } from '../OverlordPresence';
import { SettingsEntries } from './sub-components/SettingsEntries';
import type { ShellBarProps } from './ShellBar.type';
import './ShellBar.css';

const ShellBar = (props: ShellBarProps) => {
  const { mode, boardKey, boardsAvailable, onMode } = props;
  const local = useLocalBanner();
  const items: OverflowRowItem[] = [
    { id: 'mark', pinned: true, node: <OverlordPresence /> },
    { id: 'mode', pinned: true, node: <ModeSwitch mode={mode} boardsAvailable={boardsAvailable} onChange={onMode} /> },
    ...(mode === 'boards' ? [{ id: 'boards', priority: 5, node: <BoardSwitcher boardKey={boardKey} /> }] : []),
    ...(local ? [{ id: 'local', priority: 3, node: <LocalBanner state={local} /> }] : []),
    { id: 'owner', pinned: true, end: true, node: <SettingsEntries /> },
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
