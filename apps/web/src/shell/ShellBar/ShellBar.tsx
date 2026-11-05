/* @layer renderer-app @kind component */
import { BrandMark, Flex } from '@drizztdourden08/tessera';
import { BoardSwitcher } from '../BoardSwitcher';
import { LocalBanner } from '../LocalBanner';
import { ModeSwitch } from '../ModeSwitch';
import type { ShellBarProps } from './ShellBar.type';
import './ShellBar.css';

const ShellBar = (props: ShellBarProps) => {
  const { mode, boardKey, boardsAvailable, onMode } = props;
  return (
    <Flex as="header" className="shell-bar" align="center" gap="md">
      <BrandMark app="argus" size="sm" />
      <ModeSwitch mode={mode} boardsAvailable={boardsAvailable} onChange={onMode} />
      {mode === 'boards' ? <BoardSwitcher boardKey={boardKey} /> : null}
      <Flex className="shell-bar__status" align="center">
        <LocalBanner />
      </Flex>
    </Flex>
  );
};

export { ShellBar };
