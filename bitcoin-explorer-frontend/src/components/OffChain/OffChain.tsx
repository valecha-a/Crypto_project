import React, { useEffect, useState } from 'react';
import axios from 'axios';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer
} from 'recharts';

import './OffChain.css';

interface BlockchainTransaction {
    id: number;
    chart_name: string;
    unit: string;
    period: string;
    description: string;
    timestamp: string;  
    value_x: number;   
    value_y: number;   
}

const OffChain: React.FC = () => {
    const [transactions, setTransactions] = useState<BlockchainTransaction[]>([]);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        const fetchData = () => {
            axios.get<BlockchainTransaction[]>('http://localhost:8080/api/transactions')
                .then(response => {
                    setTransactions(response.data);
                    setLoading(false);
                })
                .catch(error => {
                    setError('Failed to fetch transactions.');
                    setLoading(false);
                });
        };

        fetchData();

        const intervalId = setInterval(fetchData, 300); // Fetch data every 5 min

        return () => clearInterval(intervalId); 
    }, []);

    if (loading) {
        return <div>Loading...</div>;
    }

    if (error) {
        return <div className="error">{error}</div>;
    }

    return (
        <div className="metrics-container">
            <h1>Blockchain Transactions (Off-Chain)</h1>
            <ResponsiveContainer width="100%" height={400}>
                <LineChart data={transactions}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="timestamp" 
                           label={{ value: 'x = Time', position: 'outside', textAnchor: 'middle', offset: 10 }}
                           angle={-90} 
                           interval={100} 
                           tick={{ fontSize: 12, textAnchor: 'middle', dy: 10 }} 
                    />
                    <YAxis label={{ value: 'y = Number of Transactions', angle: -90, position: 'insideCenter' }} />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="value_y" name="Transactions" stroke="#8884d8" activeDot={{ r: 8 }} />
                </LineChart>
            </ResponsiveContainer>
            <div className="table-container">
                <h2>Transactions Data Below</h2> {/* Move the title lower */}
                <table>
                    <thead>
                        <tr>
                            <th>ID</th>
                            <th>Chart Name</th>
                            <th>Unit</th>
                            <th>Period</th>
                            <th>Description</th>
                            <th>Timestamp</th>
                            <th>Value X</th>
                            <th>Value Y</th>
                        </tr>
                    </thead>
                    <tbody>
                        {transactions.map(transaction => (
                            <tr key={transaction.id}>
                                <td>{transaction.id}</td>
                                <td>{transaction.chart_name}</td>
                                <td>{transaction.unit}</td>
                                <td>{transaction.period}</td>
                                <td>{transaction.description}</td>
                                <td>{transaction.timestamp}</td>
                                <td>{transaction.value_x}</td>
                                <td>{transaction.value_y}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

export default OffChain;