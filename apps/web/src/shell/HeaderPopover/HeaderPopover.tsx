/* @layer renderer-app @kind component */
import { Anchored, Button, Card, Flex } from '@drizztdourden08/tessera';
import { usePopover } from './behavior/usePopover';
import type { HeaderPopoverProps } from './HeaderPopover.type';
import './HeaderPopover.css';

const HeaderPopover = (props: HeaderPopoverProps) => {
  const { trigger, label, hint, children } = props;
  const { anchor, open, toggle, close } = usePopover();
  return (
    <>
      <Button ref={anchor} size="sm" variant="ghost" title={hint} aria-haspopup="dialog" aria-expanded={open} onClick={toggle}>
        {trigger}
      </Button>
      {open ? (
        <Anchored anchorRef={anchor} placement="bottom-start" onDismiss={close} className="header-popover" role="dialog" aria-label={label}>
          <Card>
            <Flex className="header-popover__body" direction="column" gap="sm">
              {children}
            </Flex>
          </Card>
        </Anchored>
      ) : null}
    </>
  );
};

export { HeaderPopover };
