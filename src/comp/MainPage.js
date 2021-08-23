import React from 'react';
import pt from 'prop-types';
import styled from '@emotion/styled';
import Editor from './Editor';
import QuadSplit from './QuadSplit';
import PageFrame from './PageFrame';
import RadialMenu from './RadialMenu';
import ResultsTabs from './ResultsTabs';
import Console from './Console';
import { EventType } from '../EventType';
import { useAsync, useEvent } from 'react-use';
import { queueMeasureRender } from '../async';
import * as api from '../api';
import * as templates from '../templates';
import useUrl from '../hooks/useUrl';

let { JECT_DOMAIN_MAIN, JECT_DOMAIN_FRAME } = process.env;
if (location.hostname === `${JECT_DOMAIN_MAIN}.local`) {
  JECT_DOMAIN_MAIN += '.local';
  JECT_DOMAIN_FRAME += '.local';
}

const MenuItem = styled.div`
  font-size: 1.3em;
`;

function MainPage(props) {
  const url = useUrl();
  const [events] = React.useState(() => ({
    resize: new EventType(),
    save: new EventType(),
    run: new EventType(),
    consoleMessage: new EventType(),
  }));
  const session = React.useRef(templates.get(props.templateName) ?? templates.get());
  const resultTab = url.query('rt') === 'console' ? 'console' : 'frame';
  const urlSaveId = url.query('saved');
  const [submitCount, setSubmitCount] = React.useState(1);

  const loadSave = useAsync(async () => {
    if (!urlSaveId) return null;
    const initial = await api.getSaved(urlSaveId);
    return initial;
  }, []);

  const createSession = useAsync(async () => {
    if (urlSaveId) {
      if (loadSave.value) {
        const version =
          Math.max(...session.current.files.map((file) => file.version || 1)) + 1;
        const next = loadSave.value;
        session.current = {
          ...next,
          files: next.files.map((file) => ({ ...file, version })),
        };
      } else if (loadSave.loading) {
        return null;
      }
    }
    const { session_id } = await api.createSession(session.current);
    return session_id;
  }, [loadSave.value, loadSave.error]);

  events.run.use(() => {
    console.log('Running');
    api.updateSession(createSession.value, session.current).then(() => {
      setSubmitCount((c) => c + 1);
    });
  });

  events.save.use(() => {
    console.log('Saving');
    api.save(session.current).then(({ save_id }) => {
      url.withQuery('saved', save_id).withPath('/').applyByPush();
    });
  });

  const cleanupResize = React.useRef();
  useEvent(
    'resize',
    () => {
      cleanupResize.current?.();
      cleanupResize.current = queueMeasureRender(
        () => () => events.resize.emit(null),
      );
    },
    window,
  );

  const renderNewItem = (name, key) => (
    <MenuItem
      onClick={() => {
        window.open(`/new/${encodeURIComponent(key)}`);
      }}
    >
      {name}
    </MenuItem>
  );

  const centerRadialMenu = React.useMemo(
    () => (
      <RadialMenu>
        <MenuItem
          style={{ color: 'var(--green)' }}
          onClick={() => events.save.emit()}
        >
          <span>Save</span>
        </MenuItem>
        <MenuItem
          style={{ color: 'var(--purple)' }}
          onClick={() => events.run.emit()}
        >
          <span>Run</span>
        </MenuItem>
        <MenuItem
          style={{ color: 'var(--yellow)' }}
          onClick={() => console.log('TODO: Open About Page')}
        >
          <span>About</span>
        </MenuItem>
        <MenuItem
          style={{ color: 'var(--cyan)' }}
          onClick={() => window.open('https://github.com/brigand/ject.dev')}
        >
          <span>Source</span>
        </MenuItem>
        <MenuItem
          style={{ color: 'var(--blue)' }}
          onClick={() => console.log('TODO: prompt to add dependency')}
        >
          <span>+ Dep</span>
        </MenuItem>
        <MenuItem
          style={{ color: 'var(--red)' }}
          secondary={[
            renderNewItem('Default', 'default'),
            renderNewItem('React', 'react'),
          ]}
        >
          <span>New</span>
        </MenuItem>
      </RadialMenu>
    ),
    [],
  );

  if (!createSession.value) {
    return null;
  }

  return (
    <QuadSplit
      resize={events.resize}
      onSubmit={() => {}}
      center={() => centerRadialMenu}
    >
      <>
        {/* {'value:' + createSession.value} */}
        <Editor
          events={events}
          language="html"
          onChange={(value) => {
            session.current = {
              ...session.current,
              files: session.current.files.map((file) =>
                file.kind === 'Html' ? { ...file, contents: value } : file,
              ),
            };
          }}
          value={session.current.files.find((file) => file.kind === 'Html')}
        />
      </>
      <>
        <Editor
          events={events}
          language="typescript"
          onChange={(value) => {
            session.current = {
              ...session.current,
              files: session.current.files.map((file) =>
                file.kind === 'JavaScript' ? { ...file, contents: value } : file,
              ),
            };
          }}
          value={session.current.files.find((file) => file.kind === 'JavaScript')}
        />
      </>
      <>
        {/* {'value:' + createSession.value} */}
        <Editor
          events={events}
          language="css"
          onChange={(value) => {
            session.current = {
              ...session.current,
              files: session.current.files.map((file) =>
                file.kind === 'Css' ? { ...file, contents: value } : file,
              ),
            };
          }}
          value={session.current.files.find((file) => file.kind === 'Css')}
        />
      </>
      <>
        {createSession.loading ? (
          'Creating Session'
        ) : createSession.error ? (
          'Failed to create session'
        ) : createSession.value ? (
          <ResultsTabs
            value={resultTab}
            firstChild="frame"
            onChange={(value) => {
              url.withQuery('rt', value).applyByReplace();
            }}
          >
            <PageFrame
              host={JECT_DOMAIN_FRAME}
              sessionId={createSession.value}
              resize={events.resize}
              consoleMessage={events.consoleMessage}
              key={submitCount}
              data-tab="0"
            />
            <Console
              consoleMessage={events.consoleMessage}
              submitCount={submitCount}
              data-tab="1"
            />
          </ResultsTabs>
        ) : (
          'Unexpected state. Report a bug.'
        )}
      </>
    </QuadSplit>
  );
}

MainPage.propTypes = {
  templateName: pt.string,
};

export default MainPage;
