/* @layer renderer-app @kind component */
import { Button, CopyButton, Flex, Icon, IconButton } from '@drizztdourden08/tessera';
import { paneLink } from './model/pane-link';
import type { RecordPaneToolsProps } from './pane.type';

const RecordPaneTools = (props: RecordPaneToolsProps) => {
  const { code, pane, previous, next, wide, onWide, children } = props;
  const steps = [
    { key: 'previous', name: 'Previous in the list ([)', target: previous, icon: 'chevron-left' },
    { key: 'next', name: 'Next in the list (])', target: next, icon: 'chevron-right' },
  ] as const;
  return (
    <Flex gap="xs" align="center" role="toolbar" aria-label="Record tools">
      {pane.canBack && (
        <IconButton label="Back" size="xs" variant="ghost" onClick={pane.back}>
          <Icon name="arrow-left" />
        </IconButton>
      )}
      <CopyButton text={() => paneLink(code)} label={`Copy a link to ${code}`} variant="ghost" size="xs" />
      {steps.map((step) => (
        <IconButton key={step.key} label={step.name} size="xs" variant="ghost" disabled={step.target === null} onClick={() => step.target && pane.step(step.target)}>
          <Icon name={step.icon} />
        </IconButton>
      ))}
      <Button size="xs" variant="ghost" aria-pressed={wide} title="Focus mode: the drawer takes the whole width" onClick={onWide}>
        {wide ? 'Collapse' : 'Expand'}
      </Button>
      {children}
    </Flex>
  );
};

export { RecordPaneTools };
