import React, { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import { Navbar, Nav, Container, Button, Modal, Form, Alert } from 'react-bootstrap';
import { useShardX } from '../contexts/ShardXContext';
import { truncateAddress } from '../utils/helpers';

const Navigation = () => {
  const location = useLocation();
  const { isConnected, wallet, connectWallet, disconnectWallet, error, clearError } = useShardX();
  
  const [showConnectModal, setShowConnectModal] = useState(false);
  const [connectMethod, setConnectMethod] = useState('create');
  const [privateKey, setPrivateKey] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  
  const handleConnectWallet = () => {
    if (connectMethod === 'create') {
      connectWallet();
    } else if (connectMethod === 'import' && privateKey) {
      connectWallet(privateKey);
    } else if (connectMethod === 'mnemonic' && mnemonic) {
      // Use the mnemonic to create a wallet
      // This is a simplified example
      connectWallet();
    }
    
    setShowConnectModal(false);
    setPrivateKey('');
    setMnemonic('');
  };
  
  const handleDisconnectWallet = () => {
    disconnectWallet();
  };
  
  return (
    <>
      <Navbar bg="white" expand="lg" className="shadow-sm">
        <Container>
          <Navbar.Brand as={Link} to="/" className="fw-bold">
            <span className="text-gradient">ShardX DeFi</span>
          </Navbar.Brand>
          <Navbar.Toggle aria-controls="basic-navbar-nav" />
          <Navbar.Collapse id="basic-navbar-nav">
            <Nav className="me-auto">
              <Nav.Link as={Link} to="/" active={location.pathname === '/'}>
                Dashboard
              </Nav.Link>
              <Nav.Link as={Link} to="/wallet" active={location.pathname === '/wallet'}>
                Wallet
              </Nav.Link>
              <Nav.Link as={Link} to="/swap" active={location.pathname === '/swap'}>
                Swap
              </Nav.Link>
              <Nav.Link as={Link} to="/liquidity" active={location.pathname === '/liquidity'}>
                Liquidity
              </Nav.Link>
              <Nav.Link as={Link} to="/staking" active={location.pathname === '/staking'}>
                Staking
              </Nav.Link>
              <Nav.Link as={Link} to="/lending" active={location.pathname === '/lending'}>
                Lending
              </Nav.Link>
              <Nav.Link as={Link} to="/multisig" active={location.pathname === '/multisig'}>
                Multisig
              </Nav.Link>
              <Nav.Link as={Link} to="/analytics" active={location.pathname === '/analytics'}>
                Analytics
              </Nav.Link>
            </Nav>
            <Nav>
              {isConnected ? (
                <div className="d-flex align-items-center">
                  <span className="me-3 text-muted">
                    {truncateAddress(wallet.getAddress())}
                  </span>
                  <Button variant="outline-danger" size="sm" onClick={handleDisconnectWallet}>
                    Disconnect
                  </Button>
                </div>
              ) : (
                <Button variant="gradient" onClick={() => setShowConnectModal(true)}>
                  Connect Wallet
                </Button>
              )}
            </Nav>
          </Navbar.Collapse>
        </Container>
      </Navbar>
      
      {/* Connect Wallet Modal */}
      <Modal show={showConnectModal} onHide={() => setShowConnectModal(false)}>
        <Modal.Header closeButton>
          <Modal.Title>Connect Wallet</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {error && (
            <Alert variant="danger" onClose={clearError} dismissible>
              {error}
            </Alert>
          )}
          
          <Form>
            <Form.Group className="mb-3">
              <Form.Label>Connection Method</Form.Label>
              <Form.Select
                value={connectMethod}
                onChange={(e) => setConnectMethod(e.target.value)}
              >
                <option value="create">Create New Wallet</option>
                <option value="import">Import Private Key</option>
                <option value="mnemonic">Import Mnemonic</option>
              </Form.Select>
            </Form.Group>
            
            {connectMethod === 'import' && (
              <Form.Group className="mb-3">
                <Form.Label>Private Key</Form.Label>
                <Form.Control
                  type="password"
                  placeholder="Enter your private key"
                  value={privateKey}
                  onChange={(e) => setPrivateKey(e.target.value)}
                />
              </Form.Group>
            )}
            
            {connectMethod === 'mnemonic' && (
              <Form.Group className="mb-3">
                <Form.Label>Mnemonic Phrase</Form.Label>
                <Form.Control
                  as="textarea"
                  rows={3}
                  placeholder="Enter your mnemonic phrase"
                  value={mnemonic}
                  onChange={(e) => setMnemonic(e.target.value)}
                />
              </Form.Group>
            )}
          </Form>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={() => setShowConnectModal(false)}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleConnectWallet}>
            Connect
          </Button>
        </Modal.Footer>
      </Modal>
    </>
  );
};

export default Navigation;