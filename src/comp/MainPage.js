import React from 'react';
import Editor from './Editor';
import QuadSplit from './QuadSplit';
import { EventType } from '../EventType';
import { useAsync } from 'react-use';
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
          <iframe
            height={500}
            width={500}
            src={`/api/session/${encodeURIComponent(createSession.value)}/page`}
            allow="allow-modals allow-forms allow-scripts allow-same-origin allow-popups allow-top-navigation-by-user-activation allow-downloads"
            allowFullScreen
            frameBorder="0"
            style={{ background: 'white' }}
          />
        ) : (
          'Unexpected state. Report a bug.'
        )}
      </>
    </QuadSplit>
  );
}

export default MainPage;
