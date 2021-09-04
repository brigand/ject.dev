import { useEffect, useState } from 'react';

let promise = null;
let monaco = null;

export default function useMonaco() {
  const [m, setM] = useState(monaco);

  useEffect(() => {
    if (!promise) {
      promise = import('../monaco').then((m2) => {
        monaco = m2;
      });
    }

    if (!m) {
      promise.then(() => {
        setM(monaco);
      });
    }
  }, []);

  return m;
}
