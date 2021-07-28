import React from 'react';
import { useWindowSize } from 'react-use';
import pt from 'prop-types';
import { queueMeasureRender } from '../async';
import { EventType } from '../EventType';
import useOnMessage from '../hooks/useOnMessage';

function PageFrame(props) {
  const ref = React.useRef();
  const [frameSize, setFrameSize] = React.useState(null);

  const url = new URL(
    `${location.origin}/api/session/${encodeURIComponent(props.sessionId)}/page`,
  );
  url.hostname = props.host;

  const frameOrigin = new URL(url).origin;
  useOnMessage((data) => {
    if (data.type !== 'console') {
      return;
    }
    const method =
      typeof data.method === 'string' && /^[a-z]{1,16}$/.test(data.method)
        ? data.method
        : null;
    if (!method) {
      console.warn(
        `Received a console message with an unexpected 'method' of`,
        data.method,
      );
    }

    const args = Array.isArray(data.args) ? data.args : null;
    if (!args) {
      console.warn(
        `Received a console message with an unexpected 'args' of`,
        data.args,
      );
      return;
    }
    const invalidArg = args.findIndex((arg) => typeof arg !== 'string');
    if (invalidArg !== -1) {
      console.warn(
        `Received a console message with an unexpected 'args[${invalidArg}]' of`,
        data.args[invalidArg],
      );
      return;
    }

    props.consoleMessage.emit({ method: method, args });
  }, frameOrigin);

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
      src={url.toString()}
      allow="allow-forms allow-scripts allow-same-origin allow-popups allow-top-navigation-by-user-activation allow-downloads"
      allowFullScreen
      frameBorder="0"
    />
  );
}

PageFrame.propTypes = {
  resize: pt.instanceOf(EventType).isRequired,
  sessionId: pt.string.isRequired,
  host: pt.string.isRequired,
  consoleMessage: pt.instanceOf(EventType).isRequired,
};

export default PageFrame;
