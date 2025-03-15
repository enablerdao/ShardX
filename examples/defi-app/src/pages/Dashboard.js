import React, { useEffect, useState } from 'react';
import { Container, Row, Col, Card, Button } from 'react-bootstrap';
import { Link } from 'react-router-dom';
import { useShardX } from '../contexts/ShardXContext';
import { formatAmount, formatPercentage } from '../utils/helpers';

const Dashboard = () => {
  const { isConnected, balance, networkStats, loading, refreshData } = useShardX();
  const [marketData, setMarketData] = useState({
    totalValueLocked: '1,234,567.89',
    dailyVolume: '45,678.90',
    activeUsers: '12,345',
    topPairs: [
      { name: 'ETH/USDT', volume: '12,345.67', change: 2.5 },
      { name: 'BTC/USDT', volume: '98,765.43', change: -1.2 },
      { name: 'XRP/USDT', volume: '5,432.10', change: 0.8 },
      { name: 'DOT/USDT', volume: '3,456.78', change: 5.3 },
    ],
  });
  
  useEffect(() => {
    // Refresh data when component mounts
    refreshData();
    
    // Set up interval to refresh data
    const interval = setInterval(() => {
      refreshData();
    }, 30000);
    
    // Clean up interval
    return () => clearInterval(interval);
  }, [refreshData]);
  
  return (
    <Container>
      <h1 className="mb-4">Dashboard</h1>
      
      {isConnected ? (
        <Card className="mb-4 wallet-card">
          <div className="wallet-header">
            <h2 className="mb-0">Your Balance</h2>
          </div>
          <div className="wallet-body">
            <h3 className="display-4 mb-4">{formatAmount(balance)} SHX</h3>
            <Row>
              <Col md={6}>
                <Link to="/wallet">
                  <Button variant="outline-light" className="w-100">View Wallet</Button>
                </Link>
              </Col>
              <Col md={6}>
                <Link to="/swap">
                  <Button variant="light" className="w-100">Swap Tokens</Button>
                </Link>
              </Col>
            </Row>
          </div>
        </Card>
      ) : (
        <Card className="mb-4 text-center p-5">
          <h2 className="mb-3">Welcome to ShardX DeFi</h2>
          <p className="lead mb-4">Connect your wallet to start using the DeFi platform</p>
          <Button variant="gradient" size="lg">Connect Wallet</Button>
        </Card>
      )}
      
      <h2 className="mb-3">Market Overview</h2>
      <Row className="mb-4">
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-primary">
              <i className="bi bi-currency-exchange"></i>
            </div>
            <div className="stats-title">Total Value Locked</div>
            <div className="stats-value">${marketData.totalValueLocked}</div>
          </Card>
        </Col>
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-success">
              <i className="bi bi-graph-up"></i>
            </div>
            <div className="stats-title">24h Volume</div>
            <div className="stats-value">${marketData.dailyVolume}</div>
          </Card>
        </Col>
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-info">
              <i className="bi bi-people"></i>
            </div>
            <div className="stats-title">Active Users</div>
            <div className="stats-value">{marketData.activeUsers}</div>
          </Card>
        </Col>
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-warning">
              <i className="bi bi-lightning"></i>
            </div>
            <div className="stats-title">Transactions Per Second</div>
            <div className="stats-value">{networkStats?.tps || '0'}</div>
          </Card>
        </Col>
      </Row>
      
      <Row className="mb-4">
        <Col md={6}>
          <Card className="h-100">
            <Card.Header>
              <h5 className="mb-0">Top Trading Pairs</h5>
            </Card.Header>
            <Card.Body>
              <div className="table-responsive">
                <table className="table table-hover">
                  <thead>
                    <tr>
                      <th>Pair</th>
                      <th>Volume (24h)</th>
                      <th>Change (24h)</th>
                    </tr>
                  </thead>
                  <tbody>
                    {marketData.topPairs.map((pair, index) => (
                      <tr key={index}>
                        <td>{pair.name}</td>
                        <td>${pair.volume}</td>
                        <td className={pair.change >= 0 ? 'text-success' : 'text-danger'}>
                          {formatPercentage(pair.change)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </Card.Body>
          </Card>
        </Col>
        <Col md={6}>
          <Card className="h-100">
            <Card.Header>
              <h5 className="mb-0">Quick Actions</h5>
            </Card.Header>
            <Card.Body>
              <Row>
                <Col xs={6} className="mb-3">
                  <Link to="/swap" className="text-decoration-none">
                    <Card className="card-dashboard text-center p-3">
                      <div className="mb-2">
                        <i className="bi bi-arrow-left-right fs-1 text-primary"></i>
                      </div>
                      <h5>Swap</h5>
                      <p className="text-muted small">Exchange tokens</p>
                    </Card>
                  </Link>
                </Col>
                <Col xs={6} className="mb-3">
                  <Link to="/liquidity" className="text-decoration-none">
                    <Card className="card-dashboard text-center p-3">
                      <div className="mb-2">
                        <i className="bi bi-droplet fs-1 text-info"></i>
                      </div>
                      <h5>Liquidity</h5>
                      <p className="text-muted small">Provide liquidity</p>
                    </Card>
                  </Link>
                </Col>
                <Col xs={6} className="mb-3">
                  <Link to="/staking" className="text-decoration-none">
                    <Card className="card-dashboard text-center p-3">
                      <div className="mb-2">
                        <i className="bi bi-lock fs-1 text-success"></i>
                      </div>
                      <h5>Staking</h5>
                      <p className="text-muted small">Stake tokens</p>
                    </Card>
                  </Link>
                </Col>
                <Col xs={6} className="mb-3">
                  <Link to="/lending" className="text-decoration-none">
                    <Card className="card-dashboard text-center p-3">
                      <div className="mb-2">
                        <i className="bi bi-cash-coin fs-1 text-warning"></i>
                      </div>
                      <h5>Lending</h5>
                      <p className="text-muted small">Lend & borrow</p>
                    </Card>
                  </Link>
                </Col>
              </Row>
            </Card.Body>
          </Card>
        </Col>
      </Row>
      
      <Row>
        <Col md={12}>
          <Card>
            <Card.Header>
              <h5 className="mb-0">Network Status</h5>
            </Card.Header>
            <Card.Body>
              <Row>
                <Col md={3} className="mb-3">
                  <div className="text-muted">Total Transactions</div>
                  <div className="fw-bold fs-5">{networkStats?.totalTransactions?.toLocaleString() || '0'}</div>
                </Col>
                <Col md={3} className="mb-3">
                  <div className="text-muted">Average Block Time</div>
                  <div className="fw-bold fs-5">{networkStats?.avgBlockTime || '0'} seconds</div>
                </Col>
                <Col md={3} className="mb-3">
                  <div className="text-muted">Total Accounts</div>
                  <div className="fw-bold fs-5">{networkStats?.totalAccounts?.toLocaleString() || '0'}</div>
                </Col>
                <Col md={3} className="mb-3">
                  <div className="text-muted">Current Fee</div>
                  <div className="fw-bold fs-5">{networkStats?.currentFee || '0'} SHX</div>
                </Col>
              </Row>
            </Card.Body>
          </Card>
        </Col>
      </Row>
    </Container>
  );
};

export default Dashboard;