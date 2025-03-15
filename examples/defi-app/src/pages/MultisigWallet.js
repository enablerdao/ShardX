import React, { useState, useEffect } from 'react';
import { Container, Row, Col, Card, Button, Form, Table, Badge, Modal, Alert, Tabs, Tab } from 'react-bootstrap';
import { useShardX } from '../contexts/ShardXContext';
import { formatAmount, formatTimestamp, truncateAddress } from '../utils/helpers';

const MultisigWallet = () => {
  const { isConnected, wallet, multisigManager, loading, error, clearError } = useShardX();
  
  const [wallets, setWallets] = useState([]);
  const [selectedWallet, setSelectedWallet] = useState(null);
  const [transactions, setTransactions] = useState([]);
  const [showCreateWalletModal, setShowCreateWalletModal] = useState(false);
  const [showCreateTxModal, setShowCreateTxModal] = useState(false);
  const [showSignTxModal, setShowSignTxModal] = useState(false);
  const [selectedTransaction, setSelectedTransaction] = useState(null);
  
  // 新しいウォレットのフォームデータ
  const [walletName, setWalletName] = useState('');
  const [signers, setSigners] = useState(['']);
  const [requiredSignatures, setRequiredSignatures] = useState(1);
  const [createWalletError, setCreateWalletError] = useState('');
  
  // 新しいトランザクションのフォームデータ
  const [recipient, setRecipient] = useState('');
  const [amount, setAmount] = useState('');
  const [data, setData] = useState('');
  const [createTxError, setCreateTxError] = useState('');
  
  // 署名のフォームデータ
  const [signature, setSignature] = useState('');
  const [signTxError, setSignTxError] = useState('');
  
  // タブの状態
  const [activeTab, setActiveTab] = useState('my-wallets');
  
  useEffect(() => {
    if (isConnected) {
      loadWallets();
    }
  }, [isConnected]);
  
  useEffect(() => {
    if (selectedWallet) {
      loadTransactions(selectedWallet.id);
    }
  }, [selectedWallet]);
  
  const loadWallets = async () => {
    try {
      // 所有者のウォレットを取得
      const ownerWallets = await multisigManager.getWalletsByOwner(wallet.getAddress());
      
      // 署名者のウォレットを取得
      const signerWallets = await multisigManager.getWalletsBySigner(wallet.getAddress());
      
      // 重複を除去して結合
      const allWallets = [...ownerWallets];
      
      for (const signerWallet of signerWallets) {
        if (!allWallets.some(w => w.id === signerWallet.id)) {
          allWallets.push(signerWallet);
        }
      }
      
      setWallets(allWallets);
      
      // 最初のウォレットを選択
      if (allWallets.length > 0 && !selectedWallet) {
        setSelectedWallet(allWallets[0]);
      }
    } catch (err) {
      console.error('Failed to load wallets:', err);
    }
  };
  
  const loadTransactions = async (walletId) => {
    try {
      const txs = await multisigManager.getWalletTransactions(walletId);
      setTransactions(txs);
    } catch (err) {
      console.error('Failed to load transactions:', err);
    }
  };
  
  const handleAddSigner = () => {
    setSigners([...signers, '']);
  };
  
  const handleRemoveSigner = (index) => {
    const newSigners = [...signers];
    newSigners.splice(index, 1);
    setSigners(newSigners);
    
    // 必要な署名数を調整
    if (requiredSignatures > newSigners.length) {
      setRequiredSignatures(newSigners.length);
    }
  };
  
  const handleSignerChange = (index, value) => {
    const newSigners = [...signers];
    newSigners[index] = value;
    setSigners(newSigners);
  };
  
  const handleCreateWallet = async () => {
    setCreateWalletError('');
    
    try {
      // 入力を検証
      if (!walletName) {
        setCreateWalletError('Wallet name is required');
        return;
      }
      
      const validSigners = signers.filter(s => s.trim() !== '');
      
      if (validSigners.length === 0) {
        setCreateWalletError('At least one signer is required');
        return;
      }
      
      if (requiredSignatures <= 0 || requiredSignatures > validSigners.length) {
        setCreateWalletError(`Required signatures must be between 1 and ${validSigners.length}`);
        return;
      }
      
      // ウォレットを作成
      const newWallet = await multisigManager.createWallet(
        walletName,
        wallet.getAddress(),
        validSigners,
        requiredSignatures
      );
      
      // ウォレットリストを更新
      await loadWallets();
      
      // 新しいウォレットを選択
      setSelectedWallet(newWallet);
      
      // モーダルを閉じる
      setShowCreateWalletModal(false);
      
      // フォームをリセット
      setWalletName('');
      setSigners(['']);
      setRequiredSignatures(1);
    } catch (err) {
      console.error('Failed to create wallet:', err);
      setCreateWalletError(err.message);
    }
  };
  
  const handleCreateTransaction = async () => {
    setCreateTxError('');
    
    try {
      // 入力を検証
      if (!recipient) {
        setCreateTxError('Recipient address is required');
        return;
      }
      
      if (!amount || parseFloat(amount) <= 0) {
        setCreateTxError('Amount must be greater than 0');
        return;
      }
      
      // トランザクションを作成
      const tx = await multisigManager.createTransaction(
        selectedWallet.id,
        wallet.getAddress(),
        recipient,
        amount,
        data || null
      );
      
      // トランザクションリストを更新
      await loadTransactions(selectedWallet.id);
      
      // モーダルを閉じる
      setShowCreateTxModal(false);
      
      // フォームをリセット
      setRecipient('');
      setAmount('');
      setData('');
    } catch (err) {
      console.error('Failed to create transaction:', err);
      setCreateTxError(err.message);
    }
  };
  
  const handleSignTransaction = async () => {
    setSignTxError('');
    
    try {
      // 署名を検証
      if (!signature) {
        setSignTxError('Signature is required');
        return;
      }
      
      // トランザクションに署名
      await multisigManager.signTransaction(
        selectedTransaction.id,
        wallet.getAddress(),
        signature
      );
      
      // トランザクションリストを更新
      await loadTransactions(selectedWallet.id);
      
      // モーダルを閉じる
      setShowSignTxModal(false);
      
      // フォームをリセット
      setSignature('');
      setSelectedTransaction(null);
    } catch (err) {
      console.error('Failed to sign transaction:', err);
      setSignTxError(err.message);
    }
  };
  
  const handleRejectTransaction = async (txId) => {
    try {
      await multisigManager.rejectTransaction(txId, wallet.getAddress());
      
      // トランザクションリストを更新
      await loadTransactions(selectedWallet.id);
    } catch (err) {
      console.error('Failed to reject transaction:', err);
    }
  };
  
  const openSignModal = (tx) => {
    setSelectedTransaction(tx);
    setShowSignTxModal(true);
  };
  
  const getStatusBadge = (status) => {
    switch (status) {
      case 'Pending':
        return <Badge bg="warning">Pending</Badge>;
      case 'Executed':
        return <Badge bg="success">Executed</Badge>;
      case 'Rejected':
        return <Badge bg="danger">Rejected</Badge>;
      case 'Expired':
        return <Badge bg="secondary">Expired</Badge>;
      default:
        return <Badge bg="info">{status}</Badge>;
    }
  };
  
  const canSign = (tx) => {
    if (tx.status !== 'Pending') {
      return false;
    }
    
    // 既に署名済みかチェック
    return !Object.keys(tx.signatures).includes(wallet.getAddress());
  };
  
  if (!isConnected) {
    return (
      <Container>
        <Card className="text-center p-5 my-5">
          <h2 className="mb-3">Wallet Not Connected</h2>
          <p className="lead mb-4">Please connect your wallet to use multisig features</p>
          <Button variant="gradient" size="lg">Connect Wallet</Button>
        </Card>
      </Container>
    );
  }
  
  return (
    <Container>
      <h1 className="mb-4">Multisig Wallet</h1>
      
      {error && (
        <Alert variant="danger" onClose={clearError} dismissible>
          {error}
        </Alert>
      )}
      
      <Tabs
        activeKey={activeTab}
        onSelect={(k) => setActiveTab(k)}
        className="mb-4"
      >
        <Tab eventKey="my-wallets" title="My Wallets">
          <Card className="mb-4">
            <Card.Header className="d-flex justify-content-between align-items-center">
              <h5 className="mb-0">My Multisig Wallets</h5>
              <Button variant="primary" onClick={() => setShowCreateWalletModal(true)}>
                Create Wallet
              </Button>
            </Card.Header>
            <Card.Body>
              {wallets.length === 0 ? (
                <div className="text-center p-4">
                  <p className="text-muted">No multisig wallets found</p>
                  <Button variant="primary" onClick={() => setShowCreateWalletModal(true)}>
                    Create Your First Multisig Wallet
                  </Button>
                </div>
              ) : (
                <div className="table-responsive">
                  <Table hover>
                    <thead>
                      <tr>
                        <th>Name</th>
                        <th>ID</th>
                        <th>Signers</th>
                        <th>Required Signatures</th>
                        <th>Balance</th>
                        <th>Actions</th>
                      </tr>
                    </thead>
                    <tbody>
                      {wallets.map((w) => (
                        <tr key={w.id} className={selectedWallet && selectedWallet.id === w.id ? 'table-primary' : ''}>
                          <td>{w.name}</td>
                          <td>{truncateAddress(w.id, 8, 8)}</td>
                          <td>{w.signers.length}</td>
                          <td>{w.required_signatures} of {w.signers.length}</td>
                          <td>{formatAmount(w.balance)} SHX</td>
                          <td>
                            <Button
                              variant="outline-primary"
                              size="sm"
                              onClick={() => setSelectedWallet(w)}
                            >
                              Select
                            </Button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </Table>
                </div>
              )}
            </Card.Body>
          </Card>
          
          {selectedWallet && (
            <>
              <Card className="mb-4">
                <Card.Header className="d-flex justify-content-between align-items-center">
                  <h5 className="mb-0">Wallet Details: {selectedWallet.name}</h5>
                  <Button
                    variant="primary"
                    onClick={() => setShowCreateTxModal(true)}
                    disabled={selectedWallet.signers.indexOf(wallet.getAddress()) === -1}
                  >
                    Create Transaction
                  </Button>
                </Card.Header>
                <Card.Body>
                  <Row>
                    <Col md={6}>
                      <p><strong>ID:</strong> {selectedWallet.id}</p>
                      <p><strong>Owner:</strong> {truncateAddress(selectedWallet.owner_id)}</p>
                      <p><strong>Balance:</strong> {formatAmount(selectedWallet.balance)} SHX</p>
                      <p><strong>Required Signatures:</strong> {selectedWallet.required_signatures} of {selectedWallet.signers.length}</p>
                    </Col>
                    <Col md={6}>
                      <p><strong>Signers:</strong></p>
                      <ul>
                        {selectedWallet.signers.map((signer, index) => (
                          <li key={index}>
                            {truncateAddress(signer)}
                            {signer === wallet.getAddress() && (
                              <Badge bg="info" className="ms-2">You</Badge>
                            )}
                          </li>
                        ))}
                      </ul>
                    </Col>
                  </Row>
                </Card.Body>
              </Card>
              
              <Card>
                <Card.Header>
                  <h5 className="mb-0">Transactions</h5>
                </Card.Header>
                <Card.Body>
                  {transactions.length === 0 ? (
                    <div className="text-center p-4">
                      <p className="text-muted">No transactions found</p>
                      {selectedWallet.signers.indexOf(wallet.getAddress()) !== -1 && (
                        <Button variant="primary" onClick={() => setShowCreateTxModal(true)}>
                          Create Transaction
                        </Button>
                      )}
                    </div>
                  ) : (
                    <div className="table-responsive">
                      <Table hover>
                        <thead>
                          <tr>
                            <th>ID</th>
                            <th>Recipient</th>
                            <th>Amount</th>
                            <th>Status</th>
                            <th>Signatures</th>
                            <th>Created</th>
                            <th>Actions</th>
                          </tr>
                        </thead>
                        <tbody>
                          {transactions.map((tx) => (
                            <tr key={tx.id}>
                              <td>{truncateAddress(tx.id, 8, 8)}</td>
                              <td>{truncateAddress(tx.to)}</td>
                              <td>{formatAmount(tx.amount)} SHX</td>
                              <td>{getStatusBadge(tx.status)}</td>
                              <td>{Object.keys(tx.signatures).length} / {tx.required_signatures}</td>
                              <td>{formatTimestamp(tx.created_at * 1000)}</td>
                              <td>
                                {canSign(tx) && (
                                  <Button
                                    variant="outline-primary"
                                    size="sm"
                                    className="me-2"
                                    onClick={() => openSignModal(tx)}
                                  >
                                    Sign
                                  </Button>
                                )}
                                {tx.status === 'Pending' && (
                                  <Button
                                    variant="outline-danger"
                                    size="sm"
                                    onClick={() => handleRejectTransaction(tx.id)}
                                  >
                                    Reject
                                  </Button>
                                )}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </Table>
                    </div>
                  )}
                </Card.Body>
              </Card>
            </>
          )}
        </Tab>
        
        <Tab eventKey="all-transactions" title="All Transactions">
          <Card>
            <Card.Header>
              <h5 className="mb-0">All Multisig Transactions</h5>
            </Card.Header>
            <Card.Body>
              {/* 全トランザクションの表示 */}
              {/* 実装は省略 */}
              <p className="text-center text-muted">Coming soon...</p>
            </Card.Body>
          </Card>
        </Tab>
      </Tabs>
      
      {/* Create Wallet Modal */}
      <Modal show={showCreateWalletModal} onHide={() => setShowCreateWalletModal(false)}>
        <Modal.Header closeButton>
          <Modal.Title>Create Multisig Wallet</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {createWalletError && (
            <Alert variant="danger" onClose={() => setCreateWalletError('')} dismissible>
              {createWalletError}
            </Alert>
          )}
          
          <Form>
            <Form.Group className="mb-3">
              <Form.Label>Wallet Name</Form.Label>
              <Form.Control
                type="text"
                placeholder="Enter wallet name"
                value={walletName}
                onChange={(e) => setWalletName(e.target.value)}
              />
            </Form.Group>
            
            <Form.Group className="mb-3">
              <Form.Label>Signers</Form.Label>
              {signers.map((signer, index) => (
                <div key={index} className="d-flex mb-2">
                  <Form.Control
                    type="text"
                    placeholder="Enter signer address"
                    value={signer}
                    onChange={(e) => handleSignerChange(index, e.target.value)}
                  />
                  {index > 0 && (
                    <Button
                      variant="outline-danger"
                      className="ms-2"
                      onClick={() => handleRemoveSigner(index)}
                    >
                      <i className="bi bi-trash"></i>
                    </Button>
                  )}
                </div>
              ))}
              <Button
                variant="outline-primary"
                className="mt-2"
                onClick={handleAddSigner}
              >
                Add Signer
              </Button>
            </Form.Group>
            
            <Form.Group className="mb-3">
              <Form.Label>Required Signatures</Form.Label>
              <Form.Control
                type="number"
                min="1"
                max={signers.length}
                value={requiredSignatures}
                onChange={(e) => setRequiredSignatures(parseInt(e.target.value))}
              />
              <Form.Text className="text-muted">
                Number of signatures required to execute transactions (1-{signers.length})
              </Form.Text>
            </Form.Group>
          </Form>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={() => setShowCreateWalletModal(false)}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleCreateWallet} disabled={loading}>
            {loading ? 'Creating...' : 'Create Wallet'}
          </Button>
        </Modal.Footer>
      </Modal>
      
      {/* Create Transaction Modal */}
      <Modal show={showCreateTxModal} onHide={() => setShowCreateTxModal(false)}>
        <Modal.Header closeButton>
          <Modal.Title>Create Transaction</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {createTxError && (
            <Alert variant="danger" onClose={() => setCreateTxError('')} dismissible>
              {createTxError}
            </Alert>
          )}
          
          <Form>
            <Form.Group className="mb-3">
              <Form.Label>From Wallet</Form.Label>
              <Form.Control
                type="text"
                value={selectedWallet ? selectedWallet.name : ''}
                disabled
              />
            </Form.Group>
            
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
              <Form.Text className="text-muted">
                Available balance: {selectedWallet ? formatAmount(selectedWallet.balance) : '0'} SHX
              </Form.Text>
            </Form.Group>
            
            <Form.Group className="mb-3">
              <Form.Label>Data (Optional)</Form.Label>
              <Form.Control
                as="textarea"
                rows={3}
                placeholder="Enter transaction data"
                value={data}
                onChange={(e) => setData(e.target.value)}
              />
            </Form.Group>
          </Form>
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={() => setShowCreateTxModal(false)}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleCreateTransaction} disabled={loading}>
            {loading ? 'Creating...' : 'Create Transaction'}
          </Button>
        </Modal.Footer>
      </Modal>
      
      {/* Sign Transaction Modal */}
      <Modal show={showSignTxModal} onHide={() => setShowSignTxModal(false)}>
        <Modal.Header closeButton>
          <Modal.Title>Sign Transaction</Modal.Title>
        </Modal.Header>
        <Modal.Body>
          {signTxError && (
            <Alert variant="danger" onClose={() => setSignTxError('')} dismissible>
              {signTxError}
            </Alert>
          )}
          
          {selectedTransaction && (
            <>
              <div className="mb-3">
                <p><strong>Transaction ID:</strong> {selectedTransaction.id}</p>
                <p><strong>Recipient:</strong> {selectedTransaction.to}</p>
                <p><strong>Amount:</strong> {formatAmount(selectedTransaction.amount)} SHX</p>
                <p><strong>Signatures:</strong> {Object.keys(selectedTransaction.signatures).length} / {selectedTransaction.required_signatures}</p>
              </div>
              
              <Form>
                <Form.Group className="mb-3">
                  <Form.Label>Your Signature</Form.Label>
                  <Form.Control
                    type="text"
                    placeholder="Enter your signature"
                    value={signature}
                    onChange={(e) => setSignature(e.target.value)}
                  />
                  <Form.Text className="text-muted">
                    In a real implementation, this would be generated automatically by your wallet.
                  </Form.Text>
                </Form.Group>
              </Form>
            </>
          )}
        </Modal.Body>
        <Modal.Footer>
          <Button variant="secondary" onClick={() => setShowSignTxModal(false)}>
            Cancel
          </Button>
          <Button variant="primary" onClick={handleSignTransaction} disabled={loading}>
            {loading ? 'Signing...' : 'Sign Transaction'}
          </Button>
        </Modal.Footer>
      </Modal>
    </Container>
  );
};

export default MultisigWallet;