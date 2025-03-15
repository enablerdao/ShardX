import React from 'react';
import { Routes, Route } from 'react-router-dom';
import 'bootstrap/dist/css/bootstrap.min.css';
import './App.css';

// Components
import Navigation from './components/Navigation';
import Footer from './components/Footer';

// Pages
import Dashboard from './pages/Dashboard';
import Wallet from './pages/Wallet';
import Swap from './pages/Swap';
import Liquidity from './pages/Liquidity';
import Staking from './pages/Staking';
import Lending from './pages/Lending';
import MultisigWallet from './pages/MultisigWallet';
import Analytics from './pages/Analytics';
import Settings from './pages/Settings';
import NotFound from './pages/NotFound';

function App() {
  return (
    <div className="App d-flex flex-column min-vh-100">
      <Navigation />
      <main className="flex-grow-1">
        <Routes>
          <Route path="/" element={<Dashboard />} />
          <Route path="/wallet" element={<Wallet />} />
          <Route path="/swap" element={<Swap />} />
          <Route path="/liquidity" element={<Liquidity />} />
          <Route path="/staking" element={<Staking />} />
          <Route path="/lending" element={<Lending />} />
          <Route path="/multisig" element={<MultisigWallet />} />
          <Route path="/analytics" element={<Analytics />} />
          <Route path="/settings" element={<Settings />} />
          <Route path="*" element={<NotFound />} />
        </Routes>
      </main>
      <Footer />
    </div>
  );
}

export default App;