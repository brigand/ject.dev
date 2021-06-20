import React from 'react';
import Editor from './Editor';
import QuadSplit from './QuadSplit';
import PageFrame from './PageFrame';
import { EventType } from '../EventType';
import { useAsync, useWindowSize } from 'react-use';
import * as api from '../api';

function defaultFiles() {
  return [
    {
      kind: 'JavaScript',
      version: 1,
      contents: `// This is an example
// const f = () => location.href.toLowerCase();
// console.log(f());
// const div = <div className={css.foo}>Hello, world!</div>;
document.querySelector('#root').textContent = 'Greetings';
console.log('Updated div');
`,
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
      contents: '',
    },
  ];
}

function MainPage() {
  const [resize] = React.useState(() => new EventType());
  const session = React.useRef({ files: defaultFiles() });
  useWindowSize();

  const createSession = useAsync(async () => {
    const { session_id } = await api.createSession(session.current);
    return session_id;
  }, []);

  return (
    <QuadSplit resize={resize}>
      <>
        {'value:' + createSession.value}
        <Editor
          resize={resize}
          language="html"
          onChange={(value) => {
            session.current = {
              ...session.current,
              files: session.current.files.map((file) =>
                file.kind === 'JavaScript' ? { ...file, contents: value } : file,
              ),
            };
          }}
          value={session.current.files.find((file) => file.kind === 'Html')}
        />
      </>
      <>
        <Editor
          resize={resize}
          language="javascript"
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
      <div>css</div>
      <>
        {createSession.loading ? (
          'Creating Session'
        ) : createSession.error ? (
          'Failed to create session'
        ) : createSession.value ? (
          <PageFrame sessionId={createSession.value} resize={resize} />
        ) : (
          'Unexpected state. Report a bug.'
        )}
      </>
    </QuadSplit>
  );
}

export default MainPage;
