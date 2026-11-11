/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { useNavigate, useSearchParams } from 'react-router';
import { TAB_LINK_PARAM } from '../switch.constants';

const useDeepTab = (): void => {
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const wanted = params.get(TAB_LINK_PARAM);
  useEffect(() => {
    if (!wanted) return;
    const rest = new URLSearchParams(params);
    rest.delete(TAB_LINK_PARAM);
    void navigate({ search: rest.toString(), hash: wanted }, { replace: true });
  }, [wanted, params, navigate]);
};

export { useDeepTab };
