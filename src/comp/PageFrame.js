import React from 'react';
import { useWindowSize } from 'react-use';
import { queueMeasureRender } from '../async';

function PageFrame(props) {
  const ref = React.useRef();
  const [frameSize, setFrameSize] = React.useState(null);
  const updateSize = () => {
    queueMeasureRender(() => {
      if (ref.current) {
        const { width, height } = ref.current.parentNode.getBoundingClientRect();
        return () => {
          setFrameSize({
            width: Math.floor(width - 1),
            height: Math.floor(height - 1),
          });
        };
      }
    });
  };
  const win = useWindowSize();
  props.resize.use(updateSize);
  React.useLayoutEffect(updateSize, [win.width, win.height]);

  if (!frameSize) {
    return <div ref={ref} />;
  }

  return (
    <iframe
      ref={ref}
      width={frameSize.width}
      height={frameSize.height}
      src={`/api/session/${encodeURIComponent(props.sessionId)}/page`}
      allow="allow-modals allow-forms allow-scripts allow-same-origin allow-popups allow-top-navigation-by-user-activation allow-downloads"
      allowFullScreen
      frameBorder="0"
      style={{ background: 'white' }}
    />
  );
}

export default PageFrame;
