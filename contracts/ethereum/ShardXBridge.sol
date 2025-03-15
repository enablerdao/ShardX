// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";

/**
 * @title ShardXBridge
 * @dev イーサリアムとShardX間のアセット転送を可能にするブリッジコントラクト
 */
contract ShardXBridge is Ownable, Pausable, ReentrancyGuard {
    using SafeERC20 for IERC20;

    // イベント
    event Deposit(address indexed token, address indexed from, string toShardXAddress, uint256 amount, bytes32 depositId);
    event Withdrawal(address indexed token, address indexed to, uint256 amount, bytes32 withdrawalId);
    event ValidatorAdded(address indexed validator);
    event ValidatorRemoved(address indexed validator);
    event ThresholdChanged(uint256 oldThreshold, uint256 newThreshold);

    // 構造体
    struct WithdrawalRequest {
        address token;
        address recipient;
        uint256 amount;
        uint256 timestamp;
        mapping(address => bool) validatorApprovals;
        uint256 approvalCount;
        bool executed;
    }

    // マッピング
    mapping(bytes32 => WithdrawalRequest) public withdrawalRequests;
    mapping(address => bool) public validators;
    mapping(address => bool) public supportedTokens;

    // 状態変数
    uint256 public validatorThreshold;
    uint256 public validatorCount;
    uint256 public withdrawalRequestCount;
    uint256 public depositCount;

    /**
     * @dev コンストラクタ
     * @param _initialValidators 初期バリデータのアドレス配列
     * @param _threshold 必要な承認数の閾値
     */
    constructor(address[] memory _initialValidators, uint256 _threshold) {
        require(_threshold > 0 && _threshold <= _initialValidators.length, "Invalid threshold");
        
        for (uint256 i = 0; i < _initialValidators.length; i++) {
            require(_initialValidators[i] != address(0), "Invalid validator address");
            validators[_initialValidators[i]] = true;
        }
        
        validatorCount = _initialValidators.length;
        validatorThreshold = _threshold;
    }

    /**
     * @dev サポートするトークンを追加
     * @param _token トークンのアドレス
     */
    function addSupportedToken(address _token) external onlyOwner {
        require(_token != address(0), "Invalid token address");
        require(!supportedTokens[_token], "Token already supported");
        
        supportedTokens[_token] = true;
    }

    /**
     * @dev サポートするトークンを削除
     * @param _token トークンのアドレス
     */
    function removeSupportedToken(address _token) external onlyOwner {
        require(supportedTokens[_token], "Token not supported");
        
        supportedTokens[_token] = false;
    }

    /**
     * @dev バリデータを追加
     * @param _validator バリデータのアドレス
     */
    function addValidator(address _validator) external onlyOwner {
        require(_validator != address(0), "Invalid validator address");
        require(!validators[_validator], "Validator already exists");
        
        validators[_validator] = true;
        validatorCount++;
        
        emit ValidatorAdded(_validator);
    }

    /**
     * @dev バリデータを削除
     * @param _validator バリデータのアドレス
     */
    function removeValidator(address _validator) external onlyOwner {
        require(validators[_validator], "Validator does not exist");
        require(validatorCount > validatorThreshold, "Cannot remove validator below threshold");
        
        validators[_validator] = false;
        validatorCount--;
        
        emit ValidatorRemoved(_validator);
    }

    /**
     * @dev 閾値を変更
     * @param _newThreshold 新しい閾値
     */
    function changeThreshold(uint256 _newThreshold) external onlyOwner {
        require(_newThreshold > 0 && _newThreshold <= validatorCount, "Invalid threshold");
        
        uint256 oldThreshold = validatorThreshold;
        validatorThreshold = _newThreshold;
        
        emit ThresholdChanged(oldThreshold, _newThreshold);
    }

    /**
     * @dev トークンをデポジットしてShardXに転送
     * @param _token トークンのアドレス
     * @param _amount 金額
     * @param _toShardXAddress ShardXの宛先アドレス
     */
    function deposit(address _token, uint256 _amount, string calldata _toShardXAddress) 
        external 
        nonReentrant 
        whenNotPaused 
    {
        require(supportedTokens[_token], "Token not supported");
        require(_amount > 0, "Amount must be greater than 0");
        require(bytes(_toShardXAddress).length > 0, "Invalid ShardX address");
        
        // トークンを転送
        IERC20(_token).safeTransferFrom(msg.sender, address(this), _amount);
        
        // デポジットIDを生成
        bytes32 depositId = keccak256(abi.encodePacked(
            _token, 
            msg.sender, 
            _toShardXAddress, 
            _amount, 
            block.timestamp, 
            depositCount
        ));
        
        depositCount++;
        
        // イベントを発行
        emit Deposit(_token, msg.sender, _toShardXAddress, _amount, depositId);
    }

    /**
     * @dev 引き出しリクエストを作成（バリデータのみ）
     * @param _withdrawalId 引き出しID
     * @param _token トークンのアドレス
     * @param _recipient 受取人のアドレス
     * @param _amount 金額
     */
    function requestWithdrawal(
        bytes32 _withdrawalId,
        address _token,
        address _recipient,
        uint256 _amount
    ) 
        external 
        nonReentrant 
        whenNotPaused 
    {
        require(validators[msg.sender], "Not a validator");
        require(supportedTokens[_token], "Token not supported");
        require(_recipient != address(0), "Invalid recipient");
        require(_amount > 0, "Amount must be greater than 0");
        
        // 新しい引き出しリクエストを作成
        WithdrawalRequest storage request = withdrawalRequests[_withdrawalId];
        
        // 新しいリクエストの場合
        if (request.timestamp == 0) {
            request.token = _token;
            request.recipient = _recipient;
            request.amount = _amount;
            request.timestamp = block.timestamp;
            request.approvalCount = 0;
            request.executed = false;
            withdrawalRequestCount++;
        } else {
            // 既存のリクエストの場合、パラメータが一致することを確認
            require(request.token == _token, "Token mismatch");
            require(request.recipient == _recipient, "Recipient mismatch");
            require(request.amount == _amount, "Amount mismatch");
            require(!request.executed, "Already executed");
        }
        
        // このバリデータがまだ承認していない場合
        if (!request.validatorApprovals[msg.sender]) {
            request.validatorApprovals[msg.sender] = true;
            request.approvalCount++;
            
            // 閾値に達したら実行
            if (request.approvalCount >= validatorThreshold && !request.executed) {
                request.executed = true;
                
                // トークンを転送
                IERC20(request.token).safeTransfer(request.recipient, request.amount);
                
                // イベントを発行
                emit Withdrawal(request.token, request.recipient, request.amount, _withdrawalId);
            }
        }
    }

    /**
     * @dev コントラクトを一時停止（緊急時用）
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @dev コントラクトの一時停止を解除
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @dev 引き出しリクエストの状態を取得
     * @param _withdrawalId 引き出しID
     * @return token トークンのアドレス
     * @return recipient 受取人のアドレス
     * @return amount 金額
     * @return timestamp タイムスタンプ
     * @return approvalCount 承認数
     * @return executed 実行済みかどうか
     */
    function getWithdrawalRequest(bytes32 _withdrawalId) 
        external 
        view 
        returns (
            address token,
            address recipient,
            uint256 amount,
            uint256 timestamp,
            uint256 approvalCount,
            bool executed
        ) 
    {
        WithdrawalRequest storage request = withdrawalRequests[_withdrawalId];
        return (
            request.token,
            request.recipient,
            request.amount,
            request.timestamp,
            request.approvalCount,
            request.executed
        );
    }

    /**
     * @dev バリデータの承認状態を確認
     * @param _withdrawalId 引き出しID
     * @param _validator バリデータのアドレス
     * @return 承認済みかどうか
     */
    function isApprovedByValidator(bytes32 _withdrawalId, address _validator) 
        external 
        view 
        returns (bool) 
    {
        return withdrawalRequests[_withdrawalId].validatorApprovals[_validator];
    }
}