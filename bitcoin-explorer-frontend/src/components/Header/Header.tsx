import React from 'react';
import './Header.css';
import { Link } from 'react-router-dom';

const Header: React.FC = () => {
  return (
    <header>
      <div className='navbar'>
        <nav>
          <Link to="/"><h1>CRYPTO</h1></Link>
          <ul className='right-nav'>
            <li><Link to="/OnChain">OnChain</Link></li>
            <li><Link to="/OffChain">OffChain</Link></li>
          </ul>
        </nav>
      </div>
    </header>
  );
}

export default Header;
