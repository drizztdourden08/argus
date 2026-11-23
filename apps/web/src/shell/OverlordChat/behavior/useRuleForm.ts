/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { createRule, updateRule, useAction } from '../../../data/overlord';
import type { OverlordRule, RuleDraft } from '../../../data/overlord';
import { BLANK_DRAFT } from '../OverlordChat.constants';
import type { RuleFormState } from '../chat-panels.type';

const draftOf = (rule: OverlordRule | null): RuleDraft =>
  rule ? { title: rule.title, prompt: rule.prompt, everyMin: rule.everyMin } : BLANK_DRAFT;

const useRuleForm = (rule: OverlordRule | null, onClose: () => void): RuleFormState => {
  const [draft, setDraft] = useState<RuleDraft>(() => draftOf(rule));
  const action = useAction(onClose);
  return {
    draft,
    action,
    change: (patch) => setDraft({ ...draft, ...patch }),
    save: () => void action.run(() => (rule ? updateRule(rule.id, draft) : createRule(draft))),
  };
};

export { useRuleForm };
