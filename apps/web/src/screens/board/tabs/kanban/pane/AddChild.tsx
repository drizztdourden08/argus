/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { FactSelect } from '../../../shared';
import { LIMITS, STARTING_STATUS, textProblem } from '../model';
import { AddLine } from '../sub-components/AddLine';
import { childTypes } from './model/child-types';
import type { AddChildProps } from './pane.type';

const AddChild = (props: AddChildProps) => {
  const { item, ops } = props.data;
  const kinds = childTypes(item);
  const [kind, setKind] = useState(kinds[0]?.id ?? '');
  if (kinds.length === 0) return null;
  return (
    <Flex direction="column" gap="xs">
      <FactSelect label="Child type" options={kinds.map((type) => ({ value: type.id, label: type.label }))} value={kind} onPick={setKind} />
      <AddLine
        label="Add a child"
        buttonLabel="Add child"
        limit={LIMITS.title}
        problemOf={(text) => textProblem('title', text)}
        onAdd={(title) => ops.create({ title, type: kind, thread: item.thread, status: STARTING_STATUS, parent: item.id })}
      />
    </Flex>
  );
};

export { AddChild };
