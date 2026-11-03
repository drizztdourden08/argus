/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { CriteriaSection } from '../../../../screens/board/tabs/kanban/sub-components/CriteriaSection';
import { DescriptionSection } from '../../../../screens/board/tabs/kanban/sub-components/DescriptionSection';
import { FieldGroup } from '../fields/FieldGroup';
import { FieldRow } from '../fields/FieldRow';
import { fieldGroups } from '../model/field-groups';
import { paneType } from '../model/pane-type';
import { peopleOptions } from '../model/people-options';
import { PaneFacts } from '../PaneFacts';
import type { PaneBodyProps } from '../pane.type';

const OverviewTab = (props: PaneBodyProps) => {
  const { data } = props;
  const { item, tests, threads, ops } = data;
  const type = paneType(item);
  const shared = { item, ops, people: peopleOptions(threads), cx: { threads: threads.map((thread) => thread.id), images: [] } };
  return (
    <Flex direction="column" gap="lg">
      <PaneFacts data={data} />
      {type && fieldGroups(type).map((group) => <FieldGroup key={group.id} group={group} render={(name) => <FieldRow name={name} {...shared} />} />)}
      <DescriptionSection item={item} ops={ops} />
      <CriteriaSection item={item} tests={tests} ops={ops} />
    </Flex>
  );
};

export { OverviewTab };
