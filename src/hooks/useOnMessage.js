import { useRef, useLayoutEffect } from 'react';

function useOnMessage(handler, origin) {
  const handlerRef = Object.assign(useRef(), { current: handler });
  const originRef = Object.assign(useRef(), { current: origin });

  useLayoutEffect(() => {
    const internal = (event) => {
      if (!originRef.current) {
        throw new Error(
          `inject->useOnMessage expected an origin to be provided but got ${originRef.current}`,
        );
      }
      if (event.origin === originRef.current) {
        handlerRef.current(event.data);
      }
    };
    window.addEventListener('message', internal, false);
    return () => window.removeEventListener('message', internal, false);
  }, []);
}

export default useOnMessage;
