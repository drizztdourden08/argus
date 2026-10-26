/* @layer renderer-app @kind component */
import { Button, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { DescriptionSectionProps } from '../drawer-props.type';
import { DescriptionEditor } from './DescriptionEditor';
import { DrawerSection } from './DrawerSection';

const DescriptionSection = (props: DescriptionSectionProps) => {
  const { item, ops } = props;
  const [editing, setEditing] = useState(false);
  const description = item.description ?? '';
  return (
    <DrawerSection title="Description">
      {editing ? (
        <DescriptionEditor
          value={description}
          onCancel={() => setEditing(false)}
          onSave={(value) => {
            setEditing(false);
            if (value !== description) void ops.setField(item, 'description', value);
          }}
        />
      ) : (
        <Flex direction="column" gap="xs" align="start">
          {description ? <Markdown>{description}</Markdown> : <Text tone="muted">No description yet.</Text>}
          <Button size="xs" variant="ghost" onClick={() => setEditing(true)}>Edit description</Button>
        </Flex>
      )}
    </DrawerSection>
  );
};

export { DescriptionSection };
