/*-------------working code ---------------
import React, { useEffect, useState } from 'react';
import './PriceData.css';

interface ExchangeRate {
  currency_code: string;
  rate_15m: number;
  rate_last: number;
  rate_buy: number;
  rate_sell: number;
  symbol: string;
  updated_at: string;
}

const PriceData: React.FC = () => {
  const [exchangeRates, setExchangeRates] = useState<ExchangeRate[]>([]);

  useEffect(() => {
    fetch('http://localhost:8080/api/exchange-rates')
      .then(response => response.json())
      .then(data => setExchangeRates(data))
      .catch(error => console.error('Error fetching exchange rates:', error));
  }, []);

  return (
    <div className="price-data">
      <h1>Exchange Rates</h1>
      <table>
        <thead>
          <tr>
            <th>Currency Code</th>
            <th>Rate (15m)</th>
            <th>Last Rate</th>
            <th>Buy Rate</th>
            <th>Sell Rate</th>
            <th>Symbol</th>
            <th>Updated At</th>
          </tr>
        </thead>
        <tbody>
          {exchangeRates.map(rate => (
            <tr key={rate.currency_code}>
              <td>{rate.currency_code}</td>
              <td>{rate.rate_15m}</td>
              <td>{rate.rate_last}</td>
              <td>{rate.rate_buy}</td>
              <td>{rate.rate_sell}</td>
              <td>{rate.symbol}</td>
              <td>{new Date(rate.updated_at).toLocaleString()}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default PriceData;
*/

import React, { useEffect, useState } from 'react';
import './PriceData.css';
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

interface ExchangeRate {
    currency_code: string;
    rate_15m: number;
    rate_last: number;
    rate_buy: number;
    rate_sell: number;
    symbol: string;
    updated_at: string;
}

const formatNumber = (value: number) => {
    if (value >= 1e6) {
        return `${(value / 1e6).toFixed(2)}M`; // Convert to millions
    } else if (value >= 1e3) {
        return `${(value / 1e3).toFixed(1)}K`; // Convert to thousands
    }
    return value.toFixed(0); // No conversion needed
};

const PriceData: React.FC = () => {
    const [exchangeRates, setExchangeRates] = useState<ExchangeRate[]>([]);

    useEffect(() => {
        fetch('http://localhost:8080/api/exchange-rates')
            .then(response => response.json())
            .then(data => {
                console.log('Fetched data:', data);
                setExchangeRates(data);
            })
            .catch(error => console.error('Error fetching exchange rates:', error));
    }, []);

    return (
        <div className="price-data">
            <h1>Exchange Rates</h1>
            <ResponsiveContainer width="100%" height={400}>
                <LineChart data={exchangeRates}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="currency_code" interval={0} />
                    <YAxis domain={['dataMin', 'dataMax']} tickFormatter={formatNumber} />
                    <Tooltip />
                    <Legend />
                    <Line type="monotone" dataKey="rate_15m" name="Rate (15m)" stroke="#8884d8" activeDot={{ r: 8 }} />
                    <Line type="monotone" dataKey="rate_last" name="Last Rate" stroke="#82ca9d" />
                    <Line type="monotone" dataKey="rate_buy" name="Buy Rate" stroke="#ffc658" />
                    <Line type="monotone" dataKey="rate_sell" name="Sell Rate" stroke="#ff7300" />
                </LineChart>
            </ResponsiveContainer>
            <div className="exchange-rates-table">
                <h2>Exchange Rates Data</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Currency Code</th>
                            <th>Rate (15m)</th>
                            <th>Last Rate</th>
                            <th>Buy Rate</th>
                            <th>Sell Rate</th>
                            <th>Symbol</th>
                            <th>Updated At</th>
                        </tr>
                    </thead>
                    <tbody>
                        {exchangeRates.map(rate => (
                            <tr key={rate.currency_code}>
                                <td>{rate.currency_code}</td>
                                <td>{rate.rate_15m}</td>
                                <td>{rate.rate_last}</td>
                                <td>{rate.rate_buy}</td>
                                <td>{rate.rate_sell}</td>
                                <td>{rate.symbol}</td>
                                <td>{new Date(rate.updated_at).toLocaleString()}</td>
                            </tr>
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
};

export default PriceData;




