import './global-init';
import React from 'react';
import { render } from 'react-dom';
import { BrowserRouter as Router, Switch, Route } from 'react-router-dom';
import 'normalize.css/normalize.css';
import './index.css';
import MainPage from './comp/MainPage';

function App() {
  return (
    <Router>
      <Switch>
        <Route component={MainPage} />
      </Switch>
    </Router>
  );
}

render(<App />, document.getElementById('root'));
