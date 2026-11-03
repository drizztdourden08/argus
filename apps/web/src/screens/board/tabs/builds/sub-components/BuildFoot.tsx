/* @layer renderer-app @kind component */
import { Flex, Spacer, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { RunPlay } from '../../../run-play';
import type { BuildFootProps } from '../builds-props.type';
import { buildStage, isMaster, noPlayHint } from '../model';
import { BuildChips } from './BuildChips';
import { RetiredNote } from './RetiredNote';
import { UsageLine } from './UsageLine';

const BuildFoot = (props: BuildFootProps) => {
  const { view, boardKey, runs } = props;
  const { launcher } = view;
  return (
    <Flex gap="sm" align="center" wrap>
      <BuildChips view={view} />
      <Text variant="caption" tone="muted" title="Stage">{buildStage(launcher)}</Text>
      {isMaster(launcher) && <UsageLine view={view} />}
      <RetiredNote view={view} boardKey={boardKey} />
      <Spacer />
      {view.playable && <RunPlay label={launcher.name} def={view.def} hint={noPlayHint(launcher.name)} boardKey={boardKey} runs={runs} />}
      {launcher.updatedAt && <Text variant="caption" tone="muted"><Age value={launcher.updatedAt} /></Text>}
    </Flex>
  );
};

export { BuildFoot };
