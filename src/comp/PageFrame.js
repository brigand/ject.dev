import React from 'react';
import { useWindowSize } from 'react-use';
import pt from 'prop-types';
import { queueMeasureRender } from '../async';
import { EventType } from '../EventType';

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

  const url = new URL(
    `${location.origin}/api/session/${encodeURIComponent(props.sessionId)}/page`,
  );
  url.hostname = props.host;

  return (
    <iframe
      ref={ref}
      width={frameSize.width}
      height={frameSize.height}
      src={url.toString()}
      allow="allow-modals allow-forms allow-scripts allow-same-origin allow-popups allow-top-navigation-by-user-activation allow-downloads"
      allowFullScreen
      frameBorder="0"
    />
  );
}

PageFrame.propTypes = {
  resize: pt.instanceOf(EventType).isRequired,
  sessionId: pt.string.isRequired,
  host: pt.string.isRequired,
};

export default PageFrame;
