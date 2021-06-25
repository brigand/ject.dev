import React from 'react';
// import pt from 'prop-types';
import Editor from './Editor';
import QuadSplit from './QuadSplit';
import PageFrame from './PageFrame';
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

function MainPage() {
  const [resize] = React.useState(() => new EventType());
  const session = React.useRef({ files: defaultFiles() });
  const [submitCount, setSubmitCount] = React.useState(1);

  const createSession = useAsync(async () => {
    const { session_id } = await api.createSession(session.current);
    return session_id;
  }, []);

  return (
    <QuadSplit
      resize={resize}
      onSubmit={() => {
        api.updateSession(createSession.value, session.current).then(() => {
          setSubmitCount((c) => c + 1);
        });
      }}
    >
      <>
        {/* {'value:' + createSession.value} */}
        <Editor
          resize={resize}
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
          resize={resize}
          language="typescript"
          onChange={(value) => {
            session.current = {
              ...session.current,
              files: session.current.files.map((file) =>
                file.kind === 'JavaScript' ? { ...file, contents: value } : file,
              ),
            };
            console.log(`Changed to:`, value);
          }}
          value={session.current.files.find((file) => file.kind === 'JavaScript')}
        />
      </>
      <>
        {/* {'value:' + createSession.value} */}
        <Editor
          resize={resize}
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
            resize={resize}
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
