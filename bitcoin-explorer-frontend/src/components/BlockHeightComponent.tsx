// Part 1 & 2 of project - Fetches latest data continuously 
import React, { useEffect, useState } from 'react';
import './BlockHeightComponent.css';

interface Block {
  height: number;
  blockHash: string;
  blockSize: number;
  blockWeight: number;
  blockVersion: number;
  blockStrippedSize: number;
  difficulty: number;
  transactionCount: number;
}

const BlockHeightComponent: React.FC = () => {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchBlocks();

    const intervalId = setInterval(fetchBlocks, 2 * 60 * 1000); // 2 minutes

    const ws = new WebSocket('ws://localhost:8080/ws');

    ws.onopen = () => {
      console.log('WebSocket connected');
    };

    ws.onmessage = (event) => {
      const newBlock: Block = JSON.parse(event.data);
      setBlocks((prevBlocks) => [newBlock, ...prevBlocks]); 
    };

    ws.onclose = () => {
      console.log('WebSocket connection closed');
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return () => {
      clearInterval(intervalId); 
      ws.close(); 
    };
  }, []);

  const fetchBlocks = async () => {
    try {
      const response = await fetch('http://localhost:8080/api/blocks');
      if (response.ok) {
        const data = await response.json();
        setBlocks(data); 
      } else {
        console.error('Failed to fetch data');
        setError('Failed to fetch data');
      }
    } catch (error) {
      console.error('Error fetching data:', error);
      setError('Error fetching data');
    }
  };

  return (
    <div className="block-container">
      <h1>Bitcoin Blocks</h1>
      <p className="subtitle">Data fetched from Bitquery API</p>
      {error && <p className="error">Error: {error}</p>}
      <ul className="block-list">
        {blocks.map((block, index) => (
          <li key={index} className="block-item">
            {index === 0 && <h2 className="latest-block">LATEST BLOCK HEIGHT</h2>}
            <div className="block-detail">Height: {block.height}</div>
            <div className="block-detail">Block Hash: {block.blockHash}</div>
            <div className="block-detail">Block Size: {block.blockSize}</div>
            <div className="block-detail">Block Weight: {block.blockWeight}</div>
            <div className="block-detail">Block Version: {block.blockVersion}</div>
            <div className="block-detail">Block Stripped Size: {block.blockStrippedSize}</div>
            <div className="block-detail">Difficulty: {block.difficulty}</div>
            <div className="block-detail">Transaction Count: {block.transactionCount}</div>
          </li>
        ))}
      </ul>
    </div>
  );
};

export default BlockHeightComponent;