/* @layer renderer-app @kind component */
import { List, ListItem, Small, Tag } from '@drizztdourden08/tessera';
import { FaultTip } from '../../RegisteredStrip';
import type { BoardFaultsProps } from '../BoardHeader.type';
import './BoardNotices.css';

const BoardFaults = (props: BoardFaultsProps) => (
  <List aria-label="Faults">
    {props.faults.map((row) => (
      <ListItem key={row.id} bare className="board-notices__row">
        <Tag variant="urgency" color="danger">{row.session ?? row.thread}</Tag>
        {row.fault ? <Small className="board-notices__text"><FaultTip fault={row.fault} main={row.role === 'coordinator'} /></Small> : null}
      </ListItem>
    ))}
  </List>
);

export { BoardFaults };
