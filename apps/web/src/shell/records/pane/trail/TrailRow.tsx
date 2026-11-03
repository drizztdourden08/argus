/* @layer renderer-app @kind component */
import { Divider, Flex, Pressable } from '@drizztdourden08/tessera';
import { nodeOpens } from '../model/node-opens';
import { nodeCode, nodeRef } from '../model/node-ref';
import type { TrailRowProps } from './trail.type';
import { TrailLabel } from './TrailLabel';
import { useTrailGo } from './useTrailGo';

const TrailRow = (props: TrailRowProps) => {
  const { node, boardKey, now, open } = props;
  const go = useTrailGo(open);
  const target = nodeOpens(node, boardKey);
  const label = <TrailLabel node={node} boardKey={boardKey} now={now} />;
  return (
    <Flex direction="column" gap="xs" role="listitem">
      {target ? <Pressable aria-label={`Open ${nodeRef(node, boardKey)}: ${node.title}`} onClick={() => go(target)}>{label}</Pressable> : label}
      {node.children.length > 0 && (
        <Flex gap="sm">
          <Divider orientation="vertical" />
          <Flex direction="column" gap="xs" role="list" aria-label={`Waits on, ${nodeCode(node)}`}>
            {node.children.map((child) => (
              <TrailRow key={`${child.project}/${child.item}`} node={child} boardKey={boardKey} now={now} open={open} />
            ))}
          </Flex>
        </Flex>
      )}
    </Flex>
  );
};

export { TrailRow };
