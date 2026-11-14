/* @layer renderer-app @kind component */
import { Disclosure, Stack, Text } from '@drizztdourden08/tessera';
import type { ToolsRowProps } from '../OverlordChat.type';

const ToolsRow = (props: ToolsRowProps) => {
  const { names } = props;
  const count = names.length;
  return (
    <Disclosure size="sm" summary={`${count} tool ${count === 1 ? 'call' : 'calls'} folded: ${names.join(', ')}`}>
      <Stack gap="2xs">
        {names.map((name, at) => <Text key={`${at}-${name}`} variant="caption" tone="muted">{name}</Text>)}
      </Stack>
    </Disclosure>
  );
};

export { ToolsRow };
