import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import './index.css';
import App from './App';
import reportWebVitals from './reportWebVitals';
import { ShardXProvider } from './contexts/ShardXContext';

const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <BrowserRouter>
      <ShardXProvider>
        <App />
      </ShardXProvider>
    </BrowserRouter>
  </React.StrictMode>
);

reportWebVitals();