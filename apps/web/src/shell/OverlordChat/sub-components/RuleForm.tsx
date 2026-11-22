/* @layer renderer-app @kind component */
import { Button, Callout, Field, Flex, NumberInput, Stack, TextInput, Textarea } from '@drizztdourden08/tessera';
import { useRuleForm } from '../behavior/useRuleForm';
import { MAX_EVERY_MIN, MIN_EVERY_MIN } from '../OverlordChat.constants';
import type { RuleFormProps } from '../OverlordChat.type';

const RuleForm = (props: RuleFormProps) => {
  const { rule, onClose } = props;
  const form = useRuleForm(rule, onClose);
  const { draft, action } = form;
  return (
    <Stack gap="md">
      <Field label="Title">
        <TextInput value={draft.title} maxLength={80} onChange={(event) => form.change({ title: event.target.value })} />
      </Field>
      <Field label="Prompt">
        <Textarea rows={6} value={draft.prompt} onChange={(event) => form.change({ prompt: event.target.value })} />
      </Field>
      <Field label="Every (minutes)">
        <NumberInput
          min={MIN_EVERY_MIN}
          max={MAX_EVERY_MIN}
          value={draft.everyMin}
          onChange={(everyMin) => form.change({ everyMin })}
        />
      </Field>
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
      <Flex gap="sm">
        <Button variant="primary" disabled={action.busy} onClick={form.save}>Save</Button>
        <Button disabled={action.busy} onClick={onClose}>Cancel</Button>
      </Flex>
    </Stack>
  );
};

export { RuleForm };
