use crate::error::Error;
use snow::{Builder, HandshakeState, TransportState};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// セキュアチャネル
///
/// Noiseプロトコルを使用したエンドツーエンドの暗号化通信チャネル。
/// 前方秘匿性と認証を提供する。
pub struct SecureChannel {
    /// トランスポート状態
    transport: Option<TransportState>,
    /// ハンドシェイク状態
    handshake: Option<HandshakeState>,
    /// リモートの公開鍵
    remote_pubkey: Option<[u8; 32]>,
    /// ローカルの静的キーペア
    local_keypair: snow::Keypair,
}

/// セキュアチャネルの役割
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// イニシエーター（接続を開始する側）
    Initiator,
    /// レスポンダー（接続を受け入れる側）
    Responder,
}

impl SecureChannel {
    /// 新しいSecureChannelを作成
    pub fn new(
        role: Role,
        local_private_key: &[u8],
        remote_public_key: Option<&[u8]>,
    ) -> Result<Self, Error> {
        // Noiseプロトコルのパターンを設定
        // XX: 相互認証と前方秘匿性を提供
        // 25519: Curve25519を使用した鍵交換
        // ChaChaPoly: ChaCha20-Poly1305を使用した認証付き暗号化
        // BLAKE2s: BLAKE2sを使用したハッシュ関数
        let builder = Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse()?)
            .map_err(|e| Error::CryptoError(format!("Failed to create Noise builder: {}", e)))?;

        // ローカルの静的キーペアを設定
        let local_keypair = snow::Keypair::from_private_key(local_private_key)
            .map_err(|e| Error::CryptoError(format!("Failed to create keypair: {}", e)))?;

        // リモートの公開鍵を設定
        let remote_pubkey = remote_public_key.map(|key| {
            let mut pubkey = [0u8; 32];
            pubkey.copy_from_slice(key);
            pubkey
        });

        // 役割に応じてハンドシェイク状態を作成
        let handshake = match role {
            Role::Initiator => {
                let mut builder = builder.local_private_key(&local_keypair.private);

                if let Some(remote_key) = remote_pubkey {
                    builder = builder.remote_public_key(&remote_key);
                }

                builder
                    .build_initiator()
                    .map_err(|e| Error::CryptoError(format!("Failed to build initiator: {}", e)))?
            }
            Role::Responder => builder
                .local_private_key(&local_keypair.private)
                .build_responder()
                .map_err(|e| Error::CryptoError(format!("Failed to build responder: {}", e)))?,
        };

        Ok(Self {
            transport: None,
            handshake: Some(handshake),
            remote_pubkey,
            local_keypair,
        })
    }

    /// ハンドシェイクを実行
    pub async fn perform_handshake<T>(&mut self, stream: &mut T) -> Result<(), Error>
    where
        T: AsyncRead + AsyncWrite + Unpin,
    {
        let mut handshake = self
            .handshake
            .take()
            .ok_or_else(|| Error::CryptoError("Handshake already completed".to_string()))?;

        // バッファを準備
        let mut buffer = [0u8; 65535];

        // -> e
        let len = handshake
            .write_message(&[], &mut buffer)
            .map_err(|e| Error::CryptoError(format!("Failed to write handshake message: {}", e)))?;

        stream
            .write_all(&(len as u16).to_be_bytes())
            .await
            .map_err(|e| Error::IoError(format!("Failed to write message length: {}", e)))?;

        stream
            .write_all(&buffer[..len])
            .await
            .map_err(|e| Error::IoError(format!("Failed to write handshake message: {}", e)))?;

        // <- e, ee, s, es
        let mut len_buf = [0u8; 2];
        stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| Error::IoError(format!("Failed to read message length: {}", e)))?;

        let len = u16::from_be_bytes(len_buf) as usize;
        stream
            .read_exact(&mut buffer[..len])
            .await
            .map_err(|e| Error::IoError(format!("Failed to read handshake message: {}", e)))?;

        let _payload_len = handshake
            .read_message(&buffer[..len], &mut buffer)
            .map_err(|e| Error::CryptoError(format!("Failed to read handshake message: {}", e)))?;

        // -> s, se
        let len = handshake
            .write_message(&[], &mut buffer)
            .map_err(|e| Error::CryptoError(format!("Failed to write handshake message: {}", e)))?;

        stream
            .write_all(&(len as u16).to_be_bytes())
            .await
            .map_err(|e| Error::IoError(format!("Failed to write message length: {}", e)))?;

        stream
            .write_all(&buffer[..len])
            .await
            .map_err(|e| Error::IoError(format!("Failed to write handshake message: {}", e)))?;

        // ハンドシェイクを完了してトランスポート状態に移行
        let transport = handshake.into_transport_mode().map_err(|e| {
            Error::CryptoError(format!("Failed to transition to transport mode: {}", e))
        })?;

        self.transport = Some(transport);

        Ok(())
    }

    /// 暗号化されたメッセージを送信
    pub async fn send<T>(&mut self, stream: &mut T, message: &[u8]) -> Result<(), Error>
    where
        T: AsyncWrite + Unpin,
    {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| Error::CryptoError("Handshake not completed".to_string()))?;

        // バッファを準備
        let max_len = message.len() + 16; // メッセージ + MACタグ
        let mut buffer = vec![0u8; max_len];

        // メッセージを暗号化
        let len = transport
            .write_message(message, &mut buffer)
            .map_err(|e| Error::CryptoError(format!("Failed to encrypt message: {}", e)))?;

        // 暗号化されたメッセージを送信
        stream
            .write_all(&(len as u16).to_be_bytes())
            .await
            .map_err(|e| Error::IoError(format!("Failed to write message length: {}", e)))?;

        stream
            .write_all(&buffer[..len])
            .await
            .map_err(|e| Error::IoError(format!("Failed to write encrypted message: {}", e)))?;

        Ok(())
    }

    /// 暗号化されたメッセージを受信
    pub async fn receive<T>(&mut self, stream: &mut T) -> Result<Vec<u8>, Error>
    where
        T: AsyncRead + Unpin,
    {
        let transport = self
            .transport
            .as_mut()
            .ok_or_else(|| Error::CryptoError("Handshake not completed".to_string()))?;

        // メッセージ長を読み取り
        let mut len_buf = [0u8; 2];
        stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| Error::IoError(format!("Failed to read message length: {}", e)))?;

        let len = u16::from_be_bytes(len_buf) as usize;

        // 暗号化されたメッセージを読み取り
        let mut buffer = vec![0u8; len];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(|e| Error::IoError(format!("Failed to read encrypted message: {}", e)))?;

        // メッセージを復号
        let mut payload = vec![0u8; len];
        let payload_len = transport
            .read_message(&buffer, &mut payload)
            .map_err(|e| Error::CryptoError(format!("Failed to decrypt message: {}", e)))?;

        payload.truncate(payload_len);

        Ok(payload)
    }

    /// リモートの公開鍵を取得
    pub fn remote_public_key(&self) -> Option<[u8; 32]> {
        self.remote_pubkey
    }

    /// ローカルの公開鍵を取得
    pub fn local_public_key(&self) -> [u8; 32] {
        self.local_keypair.public
    }
}

/// セキュアストリーム
///
/// 暗号化された非同期ストリーム。
/// AsyncReadとAsyncWriteを実装し、自動的にメッセージの暗号化と復号を行う。
pub struct SecureStream<T> {
    /// 内部ストリーム
    inner: T,
    /// セキュアチャネル
    channel: SecureChannel,
    /// 受信バッファ
    read_buffer: Vec<u8>,
    /// 受信バッファの位置
    read_pos: usize,
}

impl<T> SecureStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// 新しいSecureStreamを作成
    pub fn new(inner: T, channel: SecureChannel) -> Self {
        Self {
            inner,
            channel,
            read_buffer: Vec::new(),
            read_pos: 0,
        }
    }

    /// ハンドシェイクを実行
    pub async fn perform_handshake(&mut self) -> Result<(), Error> {
        self.channel.perform_handshake(&mut self.inner).await
    }

    /// 内部ストリームを取得
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// リモートの公開鍵を取得
    pub fn remote_public_key(&self) -> Option<[u8; 32]> {
        self.channel.remote_public_key()
    }

    /// ローカルの公開鍵を取得
    pub fn local_public_key(&self) -> [u8; 32] {
        self.channel.local_public_key()
    }
}

impl<T> AsyncRead for SecureStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // バッファが空の場合は新しいメッセージを受信
        if self.read_pos >= self.read_buffer.len() {
            match futures::ready!(std::pin::Pin::new(&mut self.as_mut()).poll_fill_buf(cx)) {
                Ok(_) => {}
                Err(e) => return std::task::Poll::Ready(Err(e)),
            }
        }

        // バッファからデータをコピー
        let remaining = self.read_buffer.len() - self.read_pos;
        let to_copy = std::cmp::min(remaining, buf.remaining());

        buf.put_slice(&self.read_buffer[self.read_pos..self.read_pos + to_copy]);
        self.read_pos += to_copy;

        std::task::Poll::Ready(Ok(()))
    }
}

impl<T> AsyncWrite for SecureStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        // メッセージを暗号化して送信
        let fut = self.channel.send(&mut self.inner, buf);
        let mut fut = std::pin::pin!(fut);

        match futures::ready!(fut.poll(cx)) {
            Ok(()) => std::task::Poll::Ready(Ok(buf.len())),
            Err(e) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to send encrypted message: {}", e),
            ))),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl<T> SecureStream<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// 内部バッファを埋める
    fn poll_fill_buf(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // 新しいメッセージを受信
        let fut = self.channel.receive(&mut self.inner);
        let mut fut = std::pin::pin!(fut);

        match futures::ready!(fut.poll(cx)) {
            Ok(data) => {
                self.read_buffer = data;
                self.read_pos = 0;
                std::task::Poll::Ready(Ok(()))
            }
            Err(e) => std::task::Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to receive encrypted message: {}", e),
            ))),
        }
    }
}

/// セキュアチャネルファクトリ
///
/// セキュアチャネルを作成するためのファクトリ。
pub struct SecureChannelFactory {
    /// ローカルの静的キーペア
    local_keypair: snow::Keypair,
}

impl SecureChannelFactory {
    /// 新しいSecureChannelFactoryを作成
    pub fn new(private_key: Option<&[u8]>) -> Result<Self, Error> {
        let local_keypair = match private_key {
            Some(key) => snow::Keypair::from_private_key(key)
                .map_err(|e| Error::CryptoError(format!("Failed to create keypair: {}", e)))?,
            None => snow::Keypair::generate()
                .map_err(|e| Error::CryptoError(format!("Failed to generate keypair: {}", e)))?,
        };

        Ok(Self { local_keypair })
    }

    /// イニシエーターとしてセキュアチャネルを作成
    pub fn create_initiator(
        &self,
        remote_public_key: Option<&[u8]>,
    ) -> Result<SecureChannel, Error> {
        SecureChannel::new(
            Role::Initiator,
            &self.local_keypair.private,
            remote_public_key,
        )
    }

    /// レスポンダーとしてセキュアチャネルを作成
    pub fn create_responder(&self) -> Result<SecureChannel, Error> {
        SecureChannel::new(Role::Responder, &self.local_keypair.private, None)
    }

    /// ローカルの公開鍵を取得
    pub fn local_public_key(&self) -> &[u8; 32] {
        &self.local_keypair.public
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, AsyncReadExt, AsyncWriteExt};

    #[tokio::test]
    async fn test_secure_channel_handshake() {
        // イニシエーターとレスポンダーのキーペアを生成
        let initiator_keypair = snow::Keypair::generate().unwrap();
        let responder_keypair = snow::Keypair::generate().unwrap();

        // 双方向チャネルを作成
        let (client, server) = duplex(1024);

        // イニシエーターとレスポンダーのセキュアチャネルを作成
        let mut initiator = SecureChannel::new(
            Role::Initiator,
            &initiator_keypair.private,
            Some(&responder_keypair.public),
        )
        .unwrap();

        let mut responder = SecureChannel::new(
            Role::Responder,
            &responder_keypair.private,
            Some(&initiator_keypair.public),
        )
        .unwrap();

        // 並行してハンドシェイクを実行
        let initiator_task = tokio::spawn(async move {
            initiator
                .perform_handshake(&mut client.clone())
                .await
                .unwrap();
            initiator
        });

        let responder_task = tokio::spawn(async move {
            responder
                .perform_handshake(&mut server.clone())
                .await
                .unwrap();
            responder
        });

        // ハンドシェイクの完了を待機
        let mut initiator = initiator_task.await.unwrap();
        let mut responder = responder_task.await.unwrap();

        // イニシエーターからレスポンダーにメッセージを送信
        let message = b"Hello, world!";
        initiator.send(&mut client, message).await.unwrap();

        // レスポンダーがメッセージを受信
        let received = responder.receive(&mut server).await.unwrap();
        assert_eq!(received, message);

        // レスポンダーからイニシエーターにメッセージを送信
        let message = b"Hello, back!";
        responder.send(&mut server, message).await.unwrap();

        // イニシエーターがメッセージを受信
        let received = initiator.receive(&mut client).await.unwrap();
        assert_eq!(received, message);
    }

    #[tokio::test]
    async fn test_secure_stream() {
        // イニシエーターとレスポンダーのキーペアを生成
        let initiator_keypair = snow::Keypair::generate().unwrap();
        let responder_keypair = snow::Keypair::generate().unwrap();

        // 双方向チャネルを作成
        let (client, server) = duplex(1024);

        // イニシエーターとレスポンダーのセキュアチャネルを作成
        let initiator = SecureChannel::new(
            Role::Initiator,
            &initiator_keypair.private,
            Some(&responder_keypair.public),
        )
        .unwrap();

        let responder = SecureChannel::new(
            Role::Responder,
            &responder_keypair.private,
            Some(&initiator_keypair.public),
        )
        .unwrap();

        // セキュアストリームを作成
        let mut client_stream = SecureStream::new(client, initiator);
        let mut server_stream = SecureStream::new(server, responder);

        // 並行してハンドシェイクを実行
        let client_task = tokio::spawn(async move {
            client_stream.perform_handshake().await.unwrap();
            client_stream
        });

        let server_task = tokio::spawn(async move {
            server_stream.perform_handshake().await.unwrap();
            server_stream
        });

        // ハンドシェイクの完了を待機
        let mut client_stream = client_task.await.unwrap();
        let mut server_stream = server_task.await.unwrap();
        
        // クライアントからサーバーにメッセージを送信
        let message = b"Hello, secure world!";
        client_stream.write_all(message).await.unwrap();
        
        // サーバーがメッセージを受信
        let mut received = vec![0u8; message.len()];
        server_stream.read_exact(&mut received).await.unwrap();
        assert_eq!(received, message);
        
        // サーバーからクライアントにメッセージを送信
        let message = b"Hello, secure client!";
        server_stream.write_all(message).await.unwrap();
        
        // クライアントがメッセージを受信
        let mut received = vec![0u8; message.len()];
        client_stream.read_exact(&mut received).await.unwrap();
        assert_eq!(received, message);
    }
    
    #[tokio::test]
    async fn test_secure_channel_factory() {
        // ファクトリを作成
        let factory = SecureChannelFactory::new(None).unwrap();
        
        // 公開鍵を取得
        let public_key = factory.local_public_key();
        assert_eq!(public_key.len(), 32);
        
        // イニシエーターとレスポンダーを作成
        let initiator = factory.create_initiator(None).unwrap();
        let responder = factory.create_responder().unwrap();
        
        // 公開鍵を確認
        assert_eq!(initiator.local_public_key(), *factory.local_public_key());
        assert_eq!(responder.local_public_key(), *factory.local_public_key());
    }
    
    #[tokio::test]
    async fn test_secure_channel_with_predefined_keys() {
        // 事前定義されたキーペアを生成
        let initiator_keypair = snow::Keypair::generate().unwrap();
        let responder_keypair = snow::Keypair::generate().unwrap();
        
        // ファクトリを作成
        let initiator_factory = SecureChannelFactory::new(Some(&initiator_keypair.private)).unwrap();
        let responder_factory = SecureChannelFactory::new(Some(&responder_keypair.private)).unwrap();
        
        // 公開鍵を確認
        assert_eq!(*initiator_factory.local_public_key(), initiator_keypair.public);
        assert_eq!(*responder_factory.local_public_key(), responder_keypair.public);
        
        // 双方向チャネルを作成
        let (client, server) = duplex(1024);
        
        // イニシエーターとレスポンダーを作成
        let mut initiator = initiator_factory
            .create_initiator(Some(&responder_keypair.public))
            .unwrap();
        
        let mut responder = responder_factory
            .create_responder()
            .unwrap();
        
        // 並行してハンドシェイクを実行
        let initiator_task = tokio::spawn(async move {
            initiator
                .perform_handshake(&mut client.clone())
                .await
                .unwrap();
            initiator
        });
        
        let responder_task = tokio::spawn(async move {
            responder
                .perform_handshake(&mut server.clone())
                .await
                .unwrap();
            responder
        });
        
        // ハンドシェイクの完了を待機
        let mut initiator = initiator_task.await.unwrap();
        let mut responder = responder_task.await.unwrap();
        
        // メッセージを交換
        let message = b"Secure message with predefined keys";
        initiator.send(&mut client, message).await.unwrap();
        
        let received = responder.receive(&mut server).await.unwrap();
        assert_eq!(received, message);
    }
    
    #[tokio::test]
    async fn test_secure_stream_into_inner() {
        // イニシエーターとレスポンダーのキーペアを生成
        let initiator_keypair = snow::Keypair::generate().unwrap();
        let responder_keypair = snow::Keypair::generate().unwrap();
        
        // 双方向チャネルを作成
        let (client, server) = duplex(1024);
        
        // イニシエーターとレスポンダーのセキュアチャネルを作成
        let initiator = SecureChannel::new(
            Role::Initiator,
            &initiator_keypair.private,
            Some(&responder_keypair.public),
        )
        .unwrap();
        
        // セキュアストリームを作成
        let client_stream = SecureStream::new(client, initiator);
        
        // 内部ストリームを取得
        let inner = client_stream.into_inner();
        
        // 内部ストリームを使用
        let mut inner_clone = inner.clone();
        inner_clone.write_all(b"Direct message").await.unwrap();
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // イニシエーターとレスポンダーのキーペアを生成
        let initiator_keypair = snow::Keypair::generate().unwrap();
        
        // 不正な秘密鍵でセキュアチャネルを作成しようとする
        let invalid_key = [0u8; 10]; // 短すぎる鍵
        let result = SecureChannel::new(Role::Initiator, &invalid_key, None);
        assert!(result.is_err());
        
        // ハンドシェイク前にメッセージを送信しようとする
        let mut channel = SecureChannel::new(
            Role::Initiator,
            &initiator_keypair.private,
            None,
        )
        .unwrap();
        
        let (mut client, _) = duplex(1024);
        let result = channel.send(&mut client, b"Premature message").await;
        assert!(result.is_err());
        
        // ハンドシェイク前にメッセージを受信しようとする
        let result = channel.receive(&mut client).await;
        assert!(result.is_err());
    }
}