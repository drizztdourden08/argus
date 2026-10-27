/* @layer renderer-app @kind component */
import { List, ListItem, Small, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { plainText } from '../../plain-text';
import type { LogListProps } from '../OwnerFeed.type';
import './LogList.css';

const LogList = (props: LogListProps) => {
  const { lines } = props;
  return (
    <List aria-label="Work log" className="log-list">
      {lines.map((line) => (
        <ListItem key={`${line.subject}:${line.seq}:${line.ts}`} bare className="log-list__row">
          <Small tone="muted" className="log-list__age"><Age value={line.ts} /></Small>
          <Small tone="info" className="log-list__source" title={line.thread}>{line.thread}</Small>
          <Small className="log-list__text" title={line.text}>
            {line.item ? <><Text variant="caption" tone="dim" mono>{line.item}</Text>{' '}</> : null}
            {plainText(line.text)}
            {line.subject && line.subject !== line.thread ? <> <Small tone="muted">{line.subject}</Small></> : null}
          </Small>
        </ListItem>
      ))}
    </List>
  );
};

export { LogList };
