import React, { createContext, useState, useEffect, useContext } from 'react';
import { ShardXClient, Wallet, TransactionManager, MultisigManager, AIPredictionManager } from 'shardx-sdk';

// Create context
const ShardXContext = createContext();

// Provider component
export const ShardXProvider = ({ children }) => {
  const [client, setClient] = useState(null);
  const [wallet, setWallet] = useState(null);
  const [txManager, setTxManager] = useState(null);
  const [multisigManager, setMultisigManager] = useState(null);
  const [aiManager, setAiManager] = useState(null);
  const [isConnected, setIsConnected] = useState(false);
  const [balance, setBalance] = useState('0');
  const [transactions, setTransactions] = useState([]);
  const [nodeInfo, setNodeInfo] = useState(null);
  const [networkStats, setNetworkStats] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  // Initialize client
  useEffect(() => {
    const initClient = () => {
      try {
        const newClient = new ShardXClient({
          baseUrl: process.env.REACT_APP_API_URL || 'http://localhost:54868/api/v1',
        });
        
        setClient(newClient);
        
        // Initialize managers
        setTxManager(new TransactionManager(newClient));
        setMultisigManager(new MultisigManager(newClient));
        setAiManager(new AIPredictionManager(newClient));
        
        // Load node info and network stats
        loadNodeInfo(newClient);
        loadNetworkStats(newClient);
      } catch (err) {
        console.error('Failed to initialize ShardX client:', err);
        setError('Failed to initialize ShardX client');
      }
    };
    
    initClient();
  }, []);

  // Load node info
  const loadNodeInfo = async (clientInstance) => {
    try {
      const info = await clientInstance.getNodeInfo();
      setNodeInfo(info);
    } catch (err) {
      console.error('Failed to load node info:', err);
    }
  };

  // Load network stats
  const loadNetworkStats = async (clientInstance) => {
    try {
      const stats = await clientInstance.getNetworkStats();
      setNetworkStats(stats);
    } catch (err) {
      console.error('Failed to load network stats:', err);
    }
  };

  // Connect wallet
  const connectWallet = (privateKey) => {
    try {
      if (!client) {
        throw new Error('Client not initialized');
      }
      
      const newWallet = privateKey
        ? new Wallet({ client, privateKey })
        : Wallet.createRandom(client);
      
      setWallet(newWallet);
      setIsConnected(true);
      
      // Load wallet data
      loadWalletData(newWallet);
      
      return newWallet;
    } catch (err) {
      console.error('Failed to connect wallet:', err);
      setError('Failed to connect wallet');
      return null;
    }
  };

  // Create wallet from mnemonic
  const createWalletFromMnemonic = (mnemonic) => {
    try {
      if (!client) {
        throw new Error('Client not initialized');
      }
      
      const newWallet = Wallet.fromMnemonic(mnemonic, client);
      
      setWallet(newWallet);
      setIsConnected(true);
      
      // Load wallet data
      loadWalletData(newWallet);
      
      return newWallet;
    } catch (err) {
      console.error('Failed to create wallet from mnemonic:', err);
      setError('Failed to create wallet from mnemonic');
      return null;
    }
  };

  // Load wallet data
  const loadWalletData = async (walletInstance) => {
    setLoading(true);
    
    try {
      // Get balance
      const walletBalance = await walletInstance.getBalance();
      setBalance(walletBalance);
      
      // Get transactions
      const walletTransactions = await walletInstance.getTransactions();
      setTransactions(walletTransactions);
    } catch (err) {
      console.error('Failed to load wallet data:', err);
      setError('Failed to load wallet data');
    } finally {
      setLoading(false);
    }
  };

  // Disconnect wallet
  const disconnectWallet = () => {
    setWallet(null);
    setIsConnected(false);
    setBalance('0');
    setTransactions([]);
  };

  // Send transaction
  const sendTransaction = async (to, amount, data) => {
    setLoading(true);
    
    try {
      if (!wallet) {
        throw new Error('Wallet not connected');
      }
      
      const tx = await wallet.createTransaction(to, amount, data);
      
      // Wait for confirmation
      const confirmedTx = await txManager.waitForConfirmation(tx.id);
      
      // Reload wallet data
      await loadWalletData(wallet);
      
      return confirmedTx;
    } catch (err) {
      console.error('Failed to send transaction:', err);
      setError('Failed to send transaction');
      return null;
    } finally {
      setLoading(false);
    }
  };

  // Create multisig wallet
  const createMultisigWallet = async (name, signers, requiredSignatures) => {
    setLoading(true);
    
    try {
      if (!wallet) {
        throw new Error('Wallet not connected');
      }
      
      const multisigWallet = await multisigManager.createWallet(
        wallet,
        name,
        signers,
        requiredSignatures
      );
      
      return multisigWallet;
    } catch (err) {
      console.error('Failed to create multisig wallet:', err);
      setError('Failed to create multisig wallet');
      return null;
    } finally {
      setLoading(false);
    }
  };

  // Get AI prediction
  const getPrediction = async (pair, period = 'hour') => {
    setLoading(true);
    
    try {
      const prediction = await aiManager.getPrediction(pair, period);
      return prediction;
    } catch (err) {
      console.error('Failed to get prediction:', err);
      setError('Failed to get prediction');
      return null;
    } finally {
      setLoading(false);
    }
  };

  // Refresh data
  const refreshData = async () => {
    if (client) {
      await loadNodeInfo(client);
      await loadNetworkStats(client);
    }
    
    if (wallet) {
      await loadWalletData(wallet);
    }
  };

  // Clear error
  const clearError = () => {
    setError(null);
  };

  // Context value
  const value = {
    client,
    wallet,
    txManager,
    multisigManager,
    aiManager,
    isConnected,
    balance,
    transactions,
    nodeInfo,
    networkStats,
    loading,
    error,
    connectWallet,
    createWalletFromMnemonic,
    disconnectWallet,
    sendTransaction,
    createMultisigWallet,
    getPrediction,
    refreshData,
    clearError,
  };

  return (
    <ShardXContext.Provider value={value}>
      {children}
    </ShardXContext.Provider>
  );
};

// Custom hook to use the ShardX context
export const useShardX = () => {
  const context = useContext(ShardXContext);
  
  if (context === undefined) {
    throw new Error('useShardX must be used within a ShardXProvider');
  }
  
  return context;
};