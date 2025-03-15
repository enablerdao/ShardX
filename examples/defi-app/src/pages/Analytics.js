import React, { useState, useEffect } from 'react';
import { Container, Row, Col, Card, Button, Form, Tabs, Tab, Dropdown } from 'react-bootstrap';
import { Line, Bar, Pie, Doughnut } from 'react-chartjs-2';
import { Chart as ChartJS, CategoryScale, LinearScale, PointElement, LineElement, BarElement, ArcElement, Title, Tooltip, Legend, Filler } from 'chart.js';
import { useShardX } from '../contexts/ShardXContext';
import { formatAmount, formatPercentage } from '../utils/helpers';

// ChartJS登録
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  BarElement,
  ArcElement,
  Title,
  Tooltip,
  Legend,
  Filler
);

const Analytics = () => {
  const { isConnected, networkStats, refreshData } = useShardX();
  
  const [activeTab, setActiveTab] = useState('network');
  const [chartPeriod, setChartPeriod] = useState('week');
  const [chartType, setChartType] = useState('line');
  
  // ダミーデータ
  const [networkData, setNetworkData] = useState({
    transactions: generateDummyTimeSeriesData(30, 1000, 5000),
    tps: generateDummyTimeSeriesData(30, 10, 100),
    fees: generateDummyTimeSeriesData(30, 100, 500),
    activeUsers: generateDummyTimeSeriesData(30, 500, 2000),
    crossShardTx: generateDummyTimeSeriesData(30, 50, 300),
  });
  
  const [tokenData, setTokenData] = useState({
    price: generateDummyTimeSeriesData(30, 1, 5, true),
    volume: generateDummyTimeSeriesData(30, 10000, 50000),
    marketCap: generateDummyTimeSeriesData(30, 1000000, 5000000),
    holders: generateDummyTimeSeriesData(30, 5000, 10000, false, true),
  });
  
  const [shardData, setShardData] = useState({
    distribution: [
      { name: 'Shard 1', value: 35 },
      { name: 'Shard 2', value: 25 },
      { name: 'Shard 3', value: 20 },
      { name: 'Shard 4', value: 15 },
      { name: 'Shard 5', value: 5 },
    ],
    performance: [
      { name: 'Shard 1', tps: 85, latency: 120 },
      { name: 'Shard 2', tps: 75, latency: 150 },
      { name: 'Shard 3', tps: 90, latency: 100 },
      { name: 'Shard 4', tps: 65, latency: 180 },
      { name: 'Shard 5', tps: 95, latency: 90 },
    ],
    crossShardMatrix: [
      [0, 120, 85, 45, 30],
      [110, 0, 65, 40, 25],
      [90, 70, 0, 55, 35],
      [50, 45, 60, 0, 40],
      [35, 30, 40, 45, 0],
    ],
  });
  
  const [predictionData, setPredictionData] = useState({
    pricePrediction: generatePredictionData(30, 2, 6),
    volumePrediction: generatePredictionData(30, 20000, 60000),
    tpsPrediction: generatePredictionData(30, 50, 150),
    confidenceScores: [
      { name: 'Price', value: 85 },
      { name: 'Volume', value: 72 },
      { name: 'TPS', value: 90 },
      { name: 'User Growth', value: 65 },
    ],
  });
  
  useEffect(() => {
    refreshData();
    
    // 定期的にデータを更新
    const interval = setInterval(() => {
      refreshData();
    }, 60000);
    
    return () => clearInterval(interval);
  }, [refreshData]);
  
  // チャート期間に基づいてデータをフィルタリング
  const getFilteredData = (data, period) => {
    switch (period) {
      case 'day':
        return data.slice(-24);
      case 'week':
        return data.slice(-7);
      case 'month':
        return data;
      case 'year':
        // 実際の実装では、年間データを取得
        return data;
      default:
        return data;
    }
  };
  
  // ネットワークチャートデータ
  const getNetworkChartData = (metric) => {
    const filteredData = getFilteredData(networkData[metric], chartPeriod);
    const labels = filteredData.map(d => d.label);
    const values = filteredData.map(d => d.value);
    
    return {
      labels,
      datasets: [
        {
          label: getMetricLabel(metric),
          data: values,
          borderColor: getChartColor(metric),
          backgroundColor: getChartColor(metric, 0.2),
          fill: true,
          tension: 0.4,
        },
      ],
    };
  };
  
  // トークンチャートデータ
  const getTokenChartData = (metric) => {
    const filteredData = getFilteredData(tokenData[metric], chartPeriod);
    const labels = filteredData.map(d => d.label);
    const values = filteredData.map(d => d.value);
    
    return {
      labels,
      datasets: [
        {
          label: getMetricLabel(metric),
          data: values,
          borderColor: getChartColor(metric),
          backgroundColor: getChartColor(metric, 0.2),
          fill: true,
          tension: 0.4,
        },
      ],
    };
  };
  
  // シャード分布チャートデータ
  const getShardDistributionChartData = () => {
    return {
      labels: shardData.distribution.map(d => d.name),
      datasets: [
        {
          data: shardData.distribution.map(d => d.value),
          backgroundColor: [
            'rgba(255, 99, 132, 0.7)',
            'rgba(54, 162, 235, 0.7)',
            'rgba(255, 206, 86, 0.7)',
            'rgba(75, 192, 192, 0.7)',
            'rgba(153, 102, 255, 0.7)',
          ],
          borderColor: [
            'rgba(255, 99, 132, 1)',
            'rgba(54, 162, 235, 1)',
            'rgba(255, 206, 86, 1)',
            'rgba(75, 192, 192, 1)',
            'rgba(153, 102, 255, 1)',
          ],
          borderWidth: 1,
        },
      ],
    };
  };
  
  // シャードパフォーマンスチャートデータ
  const getShardPerformanceChartData = () => {
    return {
      labels: shardData.performance.map(d => d.name),
      datasets: [
        {
          label: 'TPS',
          data: shardData.performance.map(d => d.tps),
          backgroundColor: 'rgba(54, 162, 235, 0.7)',
          borderColor: 'rgba(54, 162, 235, 1)',
          borderWidth: 1,
        },
        {
          label: 'Latency (ms)',
          data: shardData.performance.map(d => d.latency),
          backgroundColor: 'rgba(255, 99, 132, 0.7)',
          borderColor: 'rgba(255, 99, 132, 1)',
          borderWidth: 1,
        },
      ],
    };
  };
  
  // 予測チャートデータ
  const getPredictionChartData = (metric) => {
    const data = predictionData[metric];
    const labels = data.map(d => d.label);
    const actualValues = data.map(d => d.actual);
    const predictedValues = data.map(d => d.predicted);
    
    return {
      labels,
      datasets: [
        {
          label: 'Actual',
          data: actualValues,
          borderColor: 'rgba(54, 162, 235, 1)',
          backgroundColor: 'rgba(54, 162, 235, 0.2)',
          fill: false,
          tension: 0.4,
        },
        {
          label: 'Predicted',
          data: predictedValues,
          borderColor: 'rgba(255, 99, 132, 1)',
          backgroundColor: 'rgba(255, 99, 132, 0.2)',
          fill: false,
          tension: 0.4,
          borderDash: [5, 5],
        },
      ],
    };
  };
  
  // 信頼度スコアチャートデータ
  const getConfidenceScoreChartData = () => {
    return {
      labels: predictionData.confidenceScores.map(d => d.name),
      datasets: [
        {
          label: 'Confidence Score (%)',
          data: predictionData.confidenceScores.map(d => d.value),
          backgroundColor: [
            'rgba(54, 162, 235, 0.7)',
            'rgba(255, 99, 132, 0.7)',
            'rgba(75, 192, 192, 0.7)',
            'rgba(255, 206, 86, 0.7)',
          ],
          borderColor: [
            'rgba(54, 162, 235, 1)',
            'rgba(255, 99, 132, 1)',
            'rgba(75, 192, 192, 1)',
            'rgba(255, 206, 86, 1)',
          ],
          borderWidth: 1,
        },
      ],
    };
  };
  
  // チャートオプション
  const getChartOptions = (title) => {
    return {
      responsive: true,
      plugins: {
        legend: {
          position: 'top',
        },
        title: {
          display: true,
          text: title,
        },
      },
      scales: {
        y: {
          beginAtZero: false,
        },
      },
    };
  };
  
  // メトリックのラベルを取得
  const getMetricLabel = (metric) => {
    switch (metric) {
      case 'transactions':
        return 'Transactions';
      case 'tps':
        return 'Transactions Per Second';
      case 'fees':
        return 'Transaction Fees';
      case 'activeUsers':
        return 'Active Users';
      case 'crossShardTx':
        return 'Cross-Shard Transactions';
      case 'price':
        return 'Token Price (USD)';
      case 'volume':
        return 'Trading Volume (USD)';
      case 'marketCap':
        return 'Market Cap (USD)';
      case 'holders':
        return 'Token Holders';
      case 'pricePrediction':
        return 'Price Prediction';
      case 'volumePrediction':
        return 'Volume Prediction';
      case 'tpsPrediction':
        return 'TPS Prediction';
      default:
        return metric;
    }
  };
  
  // チャートの色を取得
  const getChartColor = (metric, alpha = 1) => {
    switch (metric) {
      case 'transactions':
      case 'pricePrediction':
        return `rgba(54, 162, 235, ${alpha})`;
      case 'tps':
      case 'tpsPrediction':
        return `rgba(75, 192, 192, ${alpha})`;
      case 'fees':
        return `rgba(153, 102, 255, ${alpha})`;
      case 'activeUsers':
        return `rgba(255, 159, 64, ${alpha})`;
      case 'crossShardTx':
        return `rgba(255, 99, 132, ${alpha})`;
      case 'price':
        return `rgba(54, 162, 235, ${alpha})`;
      case 'volume':
      case 'volumePrediction':
        return `rgba(255, 206, 86, ${alpha})`;
      case 'marketCap':
        return `rgba(75, 192, 192, ${alpha})`;
      case 'holders':
        return `rgba(153, 102, 255, ${alpha})`;
      default:
        return `rgba(54, 162, 235, ${alpha})`;
    }
  };
  
  // チャートコンポーネントを取得
  const getChartComponent = (data, options, type = chartType) => {
    switch (type) {
      case 'line':
        return <Line data={data} options={options} />;
      case 'bar':
        return <Bar data={data} options={options} />;
      case 'pie':
        return <Pie data={data} options={options} />;
      case 'doughnut':
        return <Doughnut data={data} options={options} />;
      default:
        return <Line data={data} options={options} />;
    }
  };
  
  return (
    <Container>
      <h1 className="mb-4">Analytics</h1>
      
      <Row className="mb-4">
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-primary">
              <i className="bi bi-graph-up"></i>
            </div>
            <div className="stats-title">Total Transactions</div>
            <div className="stats-value">{networkStats?.totalTransactions?.toLocaleString() || '0'}</div>
          </Card>
        </Col>
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-success">
              <i className="bi bi-lightning"></i>
            </div>
            <div className="stats-title">Current TPS</div>
            <div className="stats-value">{networkStats?.tps || '0'}</div>
          </Card>
        </Col>
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-info">
              <i className="bi bi-people"></i>
            </div>
            <div className="stats-title">Active Accounts</div>
            <div className="stats-value">{networkStats?.totalAccounts?.toLocaleString() || '0'}</div>
          </Card>
        </Col>
        <Col md={3}>
          <Card className="stats-card">
            <div className="stats-icon text-warning">
              <i className="bi bi-layers"></i>
            </div>
            <div className="stats-title">Total Shards</div>
            <div className="stats-value">{networkStats?.shardCount || '5'}</div>
          </Card>
        </Col>
      </Row>
      
      <div className="d-flex justify-content-between align-items-center mb-4">
        <div>
          <Dropdown className="d-inline-block me-2">
            <Dropdown.Toggle variant="outline-primary" id="dropdown-period">
              {chartPeriod === 'day' ? 'Last 24 Hours' : 
               chartPeriod === 'week' ? 'Last 7 Days' : 
               chartPeriod === 'month' ? 'Last 30 Days' : 'Last Year'}
            </Dropdown.Toggle>
            <Dropdown.Menu>
              <Dropdown.Item onClick={() => setChartPeriod('day')}>Last 24 Hours</Dropdown.Item>
              <Dropdown.Item onClick={() => setChartPeriod('week')}>Last 7 Days</Dropdown.Item>
              <Dropdown.Item onClick={() => setChartPeriod('month')}>Last 30 Days</Dropdown.Item>
              <Dropdown.Item onClick={() => setChartPeriod('year')}>Last Year</Dropdown.Item>
            </Dropdown.Menu>
          </Dropdown>
          
          <Dropdown className="d-inline-block">
            <Dropdown.Toggle variant="outline-secondary" id="dropdown-chart-type">
              {chartType === 'line' ? 'Line Chart' : 
               chartType === 'bar' ? 'Bar Chart' : 
               chartType === 'pie' ? 'Pie Chart' : 'Doughnut Chart'}
            </Dropdown.Toggle>
            <Dropdown.Menu>
              <Dropdown.Item onClick={() => setChartType('line')}>Line Chart</Dropdown.Item>
              <Dropdown.Item onClick={() => setChartType('bar')}>Bar Chart</Dropdown.Item>
              <Dropdown.Item onClick={() => setChartType('pie')}>Pie Chart</Dropdown.Item>
              <Dropdown.Item onClick={() => setChartType('doughnut')}>Doughnut Chart</Dropdown.Item>
            </Dropdown.Menu>
          </Dropdown>
        </div>
        
        <Button variant="outline-primary" onClick={refreshData}>
          <i className="bi bi-arrow-clockwise me-2"></i>
          Refresh Data
        </Button>
      </div>
      
      <Tabs
        activeKey={activeTab}
        onSelect={(k) => setActiveTab(k)}
        className="mb-4"
      >
        <Tab eventKey="network" title="Network">
          <Row>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getNetworkChartData('transactions'),
                    getChartOptions('Transaction Count')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getNetworkChartData('tps'),
                    getChartOptions('Transactions Per Second (TPS)')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getNetworkChartData('fees'),
                    getChartOptions('Transaction Fees')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getNetworkChartData('activeUsers'),
                    getChartOptions('Active Users')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={12} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getNetworkChartData('crossShardTx'),
                    getChartOptions('Cross-Shard Transactions')
                  )}
                </Card.Body>
              </Card>
            </Col>
          </Row>
        </Tab>
        
        <Tab eventKey="token" title="Token">
          <Row>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getTokenChartData('price'),
                    getChartOptions('Token Price (USD)')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getTokenChartData('volume'),
                    getChartOptions('Trading Volume (USD)')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getTokenChartData('marketCap'),
                    getChartOptions('Market Cap (USD)')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getTokenChartData('holders'),
                    getChartOptions('Token Holders')
                  )}
                </Card.Body>
              </Card>
            </Col>
          </Row>
        </Tab>
        
        <Tab eventKey="shards" title="Shards">
          <Row>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getShardDistributionChartData(),
                    getChartOptions('Transaction Distribution by Shard'),
                    'pie'
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getShardPerformanceChartData(),
                    getChartOptions('Shard Performance'),
                    'bar'
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={12} className="mb-4">
              <Card className="h-100">
                <Card.Header>
                  <h5 className="mb-0">Cross-Shard Transaction Matrix</h5>
                </Card.Header>
                <Card.Body>
                  <div className="table-responsive">
                    <table className="table table-bordered">
                      <thead>
                        <tr>
                          <th>From / To</th>
                          {shardData.performance.map((shard, index) => (
                            <th key={index}>{shard.name}</th>
                          ))}
                        </tr>
                      </thead>
                      <tbody>
                        {shardData.crossShardMatrix.map((row, rowIndex) => (
                          <tr key={rowIndex}>
                            <th>{shardData.performance[rowIndex].name}</th>
                            {row.map((value, colIndex) => (
                              <td key={colIndex} className={rowIndex === colIndex ? 'table-light' : ''}>
                                {value > 0 ? value : '-'}
                              </td>
                            ))}
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                  <p className="text-muted small mt-2">Values represent average transactions per minute between shards</p>
                </Card.Body>
              </Card>
            </Col>
          </Row>
        </Tab>
        
        <Tab eventKey="predictions" title="AI Predictions">
          <Row>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getPredictionChartData('pricePrediction'),
                    getChartOptions('Price Prediction')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getPredictionChartData('volumePrediction'),
                    getChartOptions('Volume Prediction')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getPredictionChartData('tpsPrediction'),
                    getChartOptions('TPS Prediction')
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={6} className="mb-4">
              <Card className="h-100">
                <Card.Body>
                  {getChartComponent(
                    getConfidenceScoreChartData(),
                    getChartOptions('AI Prediction Confidence Scores'),
                    'bar'
                  )}
                </Card.Body>
              </Card>
            </Col>
            <Col lg={12}>
              <Card>
                <Card.Header>
                  <h5 className="mb-0">AI Trading Recommendations</h5>
                </Card.Header>
                <Card.Body>
                  <div className="table-responsive">
                    <table className="table">
                      <thead>
                        <tr>
                          <th>Asset</th>
                          <th>Action</th>
                          <th>Predicted Change</th>
                          <th>Confidence</th>
                          <th>Reasoning</th>
                        </tr>
                      </thead>
                      <tbody>
                        <tr>
                          <td>SHX/USD</td>
                          <td><Badge bg="success">Buy</Badge></td>
                          <td className="text-success">+5.2%</td>
                          <td>85%</td>
                          <td>Strong upward trend with increasing volume and network activity</td>
                        </tr>
                        <tr>
                          <td>SHX/BTC</td>
                          <td><Badge bg="warning">Hold</Badge></td>
                          <td>+0.8%</td>
                          <td>62%</td>
                          <td>Sideways movement expected in the short term</td>
                        </tr>
                        <tr>
                          <td>SHX/ETH</td>
                          <td><Badge bg="danger">Sell</Badge></td>
                          <td className="text-danger">-3.1%</td>
                          <td>78%</td>
                          <td>Technical indicators suggest short-term correction</td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                  <p className="text-muted small mt-2">
                    <i className="bi bi-info-circle me-1"></i>
                    Recommendations are based on AI analysis of historical data and market trends. Not financial advice.
                  </p>
                </Card.Body>
              </Card>
            </Col>
          </Row>
        </Tab>
      </Tabs>
    </Container>
  );
};

// ダミーの時系列データを生成
function generateDummyTimeSeriesData(days, min, max, isPrice = false, isAccumulating = false) {
  const data = [];
  let prevValue = isPrice ? 2.5 : Math.floor(min + Math.random() * (max - min));
  let accumulator = 0;
  
  for (let i = days; i >= 0; i--) {
    const date = new Date();
    date.setDate(date.getDate() - i);
    
    // 値を生成
    const change = isPrice
      ? prevValue * (0.98 + Math.random() * 0.04) // 価格は前日比±2%
      : min + Math.random() * (max - min);
    
    const value = isAccumulating
      ? (accumulator += Math.floor(min / 10 + Math.random() * (max / 10 - min / 10)))
      : isPrice ? change : Math.floor(change);
    
    prevValue = value;
    
    data.push({
      label: `${date.getMonth() + 1}/${date.getDate()}`,
      value: value,
    });
  }
  
  return data;
}

// 予測データを生成
function generatePredictionData(days, min, max) {
  const data = [];
  let actualValue = (min + max) / 2;
  
  for (let i = days; i >= 0; i--) {
    const date = new Date();
    date.setDate(date.getDate() - i);
    
    // 実際の値を生成
    const actualChange = actualValue * (0.98 + Math.random() * 0.04);
    actualValue = actualChange;
    
    // 予測値を生成（実際の値に±10%のノイズを加える）
    const predictedValue = i < 7 ? null : actualValue * (0.9 + Math.random() * 0.2);
    
    data.push({
      label: `${date.getMonth() + 1}/${date.getDate()}`,
      actual: actualValue,
      predicted: predictedValue,
    });
  }
  
  return data;
}

export default Analytics;