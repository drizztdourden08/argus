/* @layer renderer-app @kind component */
import { RecordRow, recordMenuGroups } from '../../../shell';
import { PORTFOLIO_SCOPE } from '../../../data/react';
import { useOpenRecord } from './useOpenRecord';
import type { OverviewRowProps } from './records.type';

const OverviewRow = (props: OverviewRowProps) => {
  const { target, assignee, title, chips, trailing, handlers } = props;
  const open = useOpenRecord();
  const go = () => open(target);
  return (
    <RecordRow
      head={{
        kind: target.kind,
        code: target.code,
        assignee,
        boardKey: target.project || PORTFOLIO_SCOPE,
        threads: [],
        menu: recordMenuGroups(target.kind, target.code, { open: target.open ? go : undefined, ...handlers }),
        menuLabel: `More actions for ${target.code}`,
      }}
      title={title}
      chips={chips}
      trailing={trailing}
      onOpen={go}
    />
  );
};

export { OverviewRow };
