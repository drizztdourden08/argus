/* @layer renderer-app @kind component */
import { BrandMark, Card, Flex } from '@drizztdourden08/tessera';
import { BoardSwitcher } from '../BoardSwitcher';
import { LocalBanner } from '../LocalBanner';
import { ModeSwitch } from '../ModeSwitch';
import { SettingsEntries } from './sub-components/SettingsEntries';
import type { ShellBarProps } from './ShellBar.type';
import './ShellBar.css';

const ShellBar = (props: ShellBarProps) => {
  const { mode, boardKey, boardsAvailable, onMode } = props;
  return (
    <Card surface="glass" className="shell-bar">
      <Flex as="header" align="center" gap="md">
        <BrandMark app="argus" size="sm" />
        <ModeSwitch mode={mode} boardsAvailable={boardsAvailable} onChange={onMode} />
        {mode === 'boards' ? <BoardSwitcher boardKey={boardKey} /> : null}
        <Flex className="shell-bar__status" align="center" gap="sm">
          <LocalBanner />
          <SettingsEntries />
        </Flex>
      </Flex>
    </Card>
  );
};

export { ShellBar };
