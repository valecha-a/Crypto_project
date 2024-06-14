/* --------working perf------
import React from 'react';
import './App.css';
import { BlockHeightComponent } from './components/BlockHeightComponent';

function App() {
    return (
        <div className="App">
            <header className="App-header">
                <BlockHeightComponent />
            </header>
        </div>
    );
}

export default App;
*/

// src/App.tsx

import React from 'react';
import './App.css';
import BlockHeightComponent from './components/BlockHeightComponent';

const App: React.FC = () => {
  return (
    <div className="App">
      <BlockHeightComponent />
    </div>
  );
};

export default App;