import React, { useState, useEffect } from 'react';
import { Container, Row, Col, Card, Button, Form, Modal, Alert, Table } from 'react-bootstrap';
import { useShardX } from '../contexts/ShardXContext';
import { formatAmount, formatTimestamp, truncateAddress } from '../utils/helpers';

const Wallet = () => {
  const { isConnected, wallet, balance, transactions, sendTransaction, loading, error, clearError } = useShardX();
  
  const [showSendModal, setShowSendModal] = useState(false);
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [memo, setMemo] = useState('');
  const [sendError, setSendError] = useState('');
  const [tokens, setTokens] = useState([
    { symbol: 'SHX', name: 'ShardX Token', balance: balance || '0', icon: 'https://via.placeholder.com/36' },
    { symbol: 'BTC', name: 'Bitcoin', balance: '0.05', icon: 'https://via.placeholder.com/36' },
    { symbol: 'ETH', name: 'Ethereum', balance: '1.2', icon: 'https://via.placeholder.com/36' },
    { symbol: 'USDT', name: 'Tether', balance: '500', icon: 'https://via.placeholder.com/36' },
  ]);
  
  useEffect(() => {
    // Update SHX balance when it changes
    setTokens(prevTokens => {
      const updatedTokens = [...prevTokens];
      const shxToken = updatedTokens.find(token => token.symbol === 'SHX');
      if (shxToken) {
        shxToken.balance = balance || '0';
      }
      return updatedTokens;
    });
  }, [balance]);
  
  const handleSend = async () => {
    setSendError('');
    
    // Validate inputs
    if (!recipient) {
      setSendError('Recipient address is required');
      return;
    }
    
    if (!amount || parseFloat(amount) <= 0) {
      setSendError('Amount must be greater than 0');
      return;
    }
    
    try {
      // Send transaction
      const tx = await sendTransaction(recipient, amount, memo);
      
      if (tx) {
        // Close modal and reset form
        setShowSendModal(false);
        setRecipient('');
        setAmount('');
        setMemo('');
      }
    } catch (err) {
      setSendError(err.message);
    }
  };
  
  if (!isConnected) {
    return (
      <Container>
        <Card className="text-center p-5 my-5">
          <h2 className="mb-3">Wallet Not Connected</h2>
          <p className="lead mb-4">Please connect your wallet to view your balance and transactions</p>
          <Button variant="gradient" size="lg">Connect Wallet</Button>
        </Card>
      </Container>
    );
  }
  
  return (
    <Container>
      <h1 className="mb-4">Wallet</h1>
      
      {error && (
        <Alert variant="danger" onClose={clearError} dismissible>
          {error}
        </Alert>
      )}
      
      <Row className="mb-4">
        <Col lg={4} className="mb-4 mb-lg-0">
          <Card className="wallet-card h-100">
            <div className="wallet-header">
              <h2 className="mb-0">Your Balance</h2>
            </div>
            <div className="wallet-body">
              <h3 className="display-4 mb-4">{formatAmount(balance)} SHX</h3>
              <p className="text-muted mb-4">
                {wallet && truncateAddress(wallet.getAddress())}
              </p>
              <Row>
                <Col>
                  <Button variant="gradient" className="w-100" onClick={() => setShowSendModal(true)}>
                    Send
                  </Button>
                </Col>
                <Col>
                  <Button variant="outline-primary" className="w-100">
                    Receive
                  </Button>
                </Col>
              </Row>
            </div>
          </Card>
        </Col>
        
        <Col lg={8}>
          <Card className="h-100">
            <Card.Header>
              <h5 className="mb-0">Your Tokens</h5>
            </Card.Header>
            <Card.Body>
              <div className="token-list">
                {tokens.map((token, index) => (
                  <div key={index} className="token-item">
                    <img src={token.icon} alt={token.symbol} className="token-icon" />
                    <div className="token-details">
                      <div className="fw-bold">{token.name}</div>
                      <div className="text-muted small">{token.symbol}</div>
                    </div>
                    <div className="token-balance">
                      {formatAmount(token.balance)} {token.symbol}
                    </div>
                  </div>
                ))}
              </div>
            </Card.Body>
          </Card>
        </Col>
      </Row>
      
      <Card>
        <Card.Header>
          <h5 className="mb-0">Transaction History</h5>
        </Card.Header>
        <Card.Body>
          {transactions.length === 0 ? (
            <div className="text-center p-4">
              <p className="text-muted">No transactions found</p>
            </div>
          ) : (
            <div className="table-responsive">
              <Table hover>
                <thead>
                  <tr>
                    <th>Transaction ID</th>
                    <th>Type</th>
                    <th>Amount</th>
                    <th>To/From</th>
                    <th>Date</th>
                    <th>Status</th>
                  </tr>
                </thead>
                <tbody>
                  {transactions.map((tx, index) => (
                    <tr key={index}>
                      <td>{truncateAddress(tx.id, 8, 8)}</td>
                      <td>{tx.from_address === wallet.getAddress() ? 'Send' : 'Receive'}</td>
                      <td className={tx.from_address === wallet.getAddress() ? 'text-danger' : 'text-success'}>
                        {tx.from_address === wallet.getAddress() ? '-' : '+'}{formatAmount(tx.amount)} SHX
                      </td>
                      <td>{truncateAddress(tx.from_address === wallet.getAddress() ? tx.to : tx.from_address)}</td>
                      <td>{formatTimestamp(tx.timestamp)}</td>
                      <td>
                        <span className={`badge bg-${tx.status === 'confirmed' ? 'success' : tx.status === 'pending' ? 'warning' : 'danger'}`}>
                          {tx.status}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </Table>
            </div>
          )}
        </Card.Body>
      </Card>
      
      {/* Send Modal */}
      <Modal show={showSendModal} onHide={() => setShowSendModal(false)}>
        <Modal.Header closeButton>
          <Modal.Title>Send Tokens</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {sendError && (
            <Alert variant="danger" onClose={() => setSendError('')} dismissible>
              {sendError}
            </Alert>
          )}
          
          <Form>
            <Form.Group className="mb-3">
              <Form.Label>Recipient Address</Form.Label>
              <Form.Control
                type="text"
                placeholder="Enter recipient address"
                value={recipient}
                onChange={(e) => setRecipient(e.target.value)}
              />
            </Form.Group>
            
            <Form.Group className="mb-3">
              <Form.Label>Amount</Form.Label>
              <Form.Control
                type="number"
                placeholder="Enter amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </Form.Group>
            
            <Form.Group className="mb-3">
              <Form.Label>Memo (Optional)</Form.Label>
              <Form.Control
                as="textarea"
                rows={3}
                placeholder="Enter memo"
                value={memo}
                onChange={(e) => setMemo(e.target.value)}
              />
            </Form.Group>
          </Form>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={() => setShowSendModal(false)}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSend} disabled={loading}>
            {loading ? 'Sending...' : 'Send'}
          </Button>
        </Modal.Footer>
      </Modal>
    </Container>
  );
};

export default Wallet;