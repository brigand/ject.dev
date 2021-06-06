import React from 'react';
import { render } from 'react-dom';
import 'normalize.css/normalize.css';
import './index.css';

import QuadSplit from './comp/QuadSplit';

function App() {
  return <QuadSplit />;
}

render(<App />, document.getElementById('root'));
