/* @layer renderer-app @kind component */
import { Button, Flex, Image, Text } from '@drizztdourden08/tessera';
import { useObjectUrl } from '../behavior/useObjectUrl';
import type { ShotPreviewProps } from '../tests-drawer-props.type';

const ShotPreview = (props: ShotPreviewProps) => {
  const { file, testId, onRemove } = props;
  const url = useObjectUrl(file);
  return (
    <Flex gap="sm" align="center">
      {url && <Image src={url} alt={`Screenshot to attach to ${testId}`} width={96} />}
      <Text variant="caption" tone="muted" mono>{file.name}</Text>
      <Button size="xs" variant="ghost" aria-label="Remove the screenshot" onClick={onRemove}>Remove</Button>
    </Flex>
  );
};

export { ShotPreview };
