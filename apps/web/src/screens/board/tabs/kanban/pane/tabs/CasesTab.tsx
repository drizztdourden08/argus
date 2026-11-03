/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { AddLine } from '../../sub-components/AddLine';
import { DrawerSection } from '../../sub-components/DrawerSection';
import { EmptyLine } from '../../sub-components/EmptyLine';
import { CaseRow } from '../CaseRow';
import { caseIds } from '../model/case-ids';
import { fieldProblem } from '../model/field-problem';
import type { PaneBodyProps } from '../pane.type';
import { CASE_ID_LIMIT } from './CasesTab.constants';

const CasesTab = (props: PaneBodyProps) => {
  const { data } = props;
  const { item, ops } = data;
  const ids = caseIds(item);
  const save = (next: string[], message: string) => ops.setField(item, 'fields.cases', next, message);
  const problemOf = (text: string): string | null => {
    const id = text.trim();
    if (ids.includes(id)) return `${id} is already a case.`;
    return fieldProblem('cases', [...ids, id].join(', '), {}) ?? null;
  };
  return (
    <DrawerSection title={`Cases · ${ids.length}`}>
      {ids.length === 0 && <EmptyLine>No case on this plan yet.</EmptyLine>}
      <Flex direction="column" gap="xs" role="list" aria-label="Cases">
        {ids.map((id) => (
          <CaseRow key={id} id={id} data={data} onRemove={(gone) => void save(ids.filter((other) => other !== gone), `Removed ${gone} from ${item.id}.`)} />
        ))}
      </Flex>
      <AddLine
        label="Add a case (T-n)"
        buttonLabel="Add case"
        limit={CASE_ID_LIMIT}
        problemOf={problemOf}
        onAdd={(text) => save([...ids, text.trim()], `Added ${text.trim()} to ${item.id}.`)}
      />
    </DrawerSection>
  );
};

export { CasesTab };
