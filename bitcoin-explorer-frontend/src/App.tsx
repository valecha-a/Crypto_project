import React from 'react';
import { BrowserRouter as Router, Route, Routes } from 'react-router-dom';
import OffChain from './components/OffChain/OffChain';
import Header from './components/Header/Header';
import OnChain from './components/OnChain/OnChain'; 
import PriceData from './components/PriceData/PriceData';

const App: React.FC = () => {
  return (
    <Router>
      <div className="App">
        <header className="App-header">
          <Header />
          <h1>Bitcoin Explorer</h1>
        </header>
        <main>
          <Routes>
            <Route path="/OffChain" element={<OffChain />} />
            <Route path="/OnChain" element={<OnChain />} />
            <Route path="/PriceData" element={<PriceData />} />
            
            <Route path="/" element={<PriceData />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
};

export default App;
