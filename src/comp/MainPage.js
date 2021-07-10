import React from 'react';
// import pt from 'prop-types';
import styled from '@emotion/styled';
import Editor from './Editor';
import QuadSplit from './QuadSplit';
import PageFrame from './PageFrame';
import RadialMenu from './RadialMenu';
import { EventType } from '../EventType';
import { useAsync } from 'react-use';
import * as api from '../api';

function defaultFiles() {
  return [
    {
      kind: 'JavaScript',
      version: 1,
      contents: ``,
    },
    {
      kind: 'Html',
      version: 1,
      contents: `<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <link rel="stylesheet" href="inject!(urls.css)" />

    <!-- inject!(deps.react) -->
    <!-- inject!(deps.jquery) -->
  </head>

  <body>
    <div id="root"></div>
    <script src="inject!(urls.js)"></script>
  </body>
</html>
`,
    },
    {
      kind: 'Css',
      version: 1,
      contents: `html {
  font-family: Arial, sans;
  background: #23262e;
  color: #d5ced9;
}`,
    },
  ];
}

const MenuItem = styled.div`
  font-size: 1.3em;
`;

function MainPage() {
  const [events] = React.useState(() => ({
    resize: new EventType(),
    save: new EventType(),
    run: new EventType(),
  }));
  const session = React.useRef({ files: defaultFiles() });
  const [submitCount, setSubmitCount] = React.useState(1);

  const createSession = useAsync(async () => {
    const { session_id } = await api.createSession(session.current);
    return session_id;
  }, []);

  events.run.use(() => {
    api.updateSession(createSession.value, session.current).then(() => {
      setSubmitCount((c) => c + 1);
    });
  });

  return (
    <QuadSplit
      resize={events.resize}
      onSubmit={() => {}}
      center={() => (
        <RadialMenu>
          <MenuItem
            style={{ color: 'var(--green)' }}
            onClick={() => console.log('TODO: Save')}
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
            onClick={() => console.log('TODO: Open Github')}
          >
            <span>Source</span>
          </MenuItem>
          <MenuItem
            style={{ color: 'var(--blue)' }}
            onClick={() => console.log('TODO: prompt to add dependency')}
          >
            <span>+ Dep</span>
          </MenuItem>
        </RadialMenu>
      )}
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
          <PageFrame
            sessionId={createSession.value}
            resize={events.resize}
            key={submitCount}
          />
        ) : (
          'Unexpected state. Report a bug.'
        )}
      </>
    </QuadSplit>
  );
}

MainPage.propTypes = {};

export default MainPage;
