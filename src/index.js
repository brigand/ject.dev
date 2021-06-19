import './global-init';
import React from 'react';
import { render } from 'react-dom';
import 'normalize.css/normalize.css';
import './index.css';
import MainPage from './comp/MainPage';

function App() {
  return <MainPage />;
}

render(<App />, document.getElementById('root'));
