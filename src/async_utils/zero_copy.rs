use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::borrow::Cow;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

/// ゼロコピーバッファ
///
/// データのコピーを最小限に抑えるためのバッファ。
/// 参照カウントによるメモリ共有を活用し、不要なコピーを回避する。
#[derive(Clone, Debug)]
pub struct ZeroCopyBuffer {
    /// 内部データ
    inner: Bytes,
}

impl ZeroCopyBuffer {
    /// 新しいZeroCopyBufferを作成
    pub fn new(data: impl Into<Bytes>) -> Self {
        Self { inner: data.into() }
    }

    /// 空のZeroCopyBufferを作成
    pub fn empty() -> Self {
        Self {
            inner: Bytes::new(),
        }
    }

    /// バッファの長さを取得
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// バッファが空かどうかを確認
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// バッファの内容を取得
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// バッファの一部を取得
    pub fn slice(&self, range: impl std::ops::RangeBounds<usize>) -> Self {
        Self {
            inner: self.inner.slice(range),
        }
    }

    /// バッファを結合
    pub fn concat(&self, other: &Self) -> Self {
        let mut buf = BytesMut::with_capacity(self.len() + other.len());
        buf.put_slice(&self.inner);
        buf.put_slice(&other.inner);
        Self {
            inner: buf.freeze(),
        }
    }

    /// バッファをBytesに変換
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }
}

impl From<Vec<u8>> for ZeroCopyBuffer {
    fn from(vec: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(vec),
        }
    }
}

impl From<&[u8]> for ZeroCopyBuffer {
    fn from(slice: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(slice),
        }
    }
}

impl From<Bytes> for ZeroCopyBuffer {
    fn from(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }
}

impl AsRef<[u8]> for ZeroCopyBuffer {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Deref for ZeroCopyBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

/// 可変ゼロコピーバッファ
///
/// 書き込み可能なゼロコピーバッファ。
/// 必要に応じて内部バッファを拡張する。
#[derive(Debug)]
pub struct ZeroCopyBufferMut {
    /// 内部データ
    inner: BytesMut,
}

impl ZeroCopyBufferMut {
    /// 新しいZeroCopyBufferMutを作成
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: BytesMut::with_capacity(capacity),
        }
    }

    /// 既存のデータからZeroCopyBufferMutを作成
    pub fn from_data(data: impl AsRef<[u8]>) -> Self {
        let mut buf = BytesMut::with_capacity(data.as_ref().len());
        buf.put_slice(data.as_ref());
        Self { inner: buf }
    }

    /// バッファの長さを取得
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// バッファが空かどうかを確認
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// バッファの容量を取得
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// バッファの内容を取得
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }

    /// バッファの内容を可変参照として取得
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    /// バッファにデータを追加
    pub fn put_slice(&mut self, data: &[u8]) {
        self.inner.put_slice(data);
    }

    /// バッファに別のバッファを追加
    pub fn put_buffer(&mut self, buf: &ZeroCopyBuffer) {
        self.inner.put_slice(buf.as_bytes());
    }

    /// バッファをフリーズしてZeroCopyBufferに変換
    pub fn freeze(self) -> ZeroCopyBuffer {
        ZeroCopyBuffer {
            inner: self.inner.freeze(),
        }
    }

    /// バッファをクリア
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// バッファの容量を確保
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }
}

impl AsRef<[u8]> for ZeroCopyBufferMut {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl Deref for ZeroCopyBufferMut {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl DerefMut for ZeroCopyBufferMut {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_bytes_mut()
    }
}

/// ゼロコピーデータ
///
/// 所有権を持つデータと参照のどちらも格納できる汎用コンテナ。
/// 不要なコピーを回避するために使用する。
#[derive(Debug, Clone)]
pub enum ZeroCopyData<'a, T: 'a + ?Sized> {
    /// 所有権を持つデータ
    Owned(T),
    /// 参照
    Borrowed(&'a T),
    /// 参照カウント
    Shared(Arc<T>),
}

impl<'a, T: Clone + ?Sized> ZeroCopyData<'a, T> {
    /// データを取得（必要に応じてクローン）
    pub fn into_owned(self) -> T
    where
        T: Sized,
    {
        match self {
            ZeroCopyData::Owned(data) => data,
            ZeroCopyData::Borrowed(data) => data.clone(),
            ZeroCopyData::Shared(data) => (*data).clone(),
        }
    }
}

impl<'a, T: ?Sized> Deref for ZeroCopyData<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ZeroCopyData::Owned(data) => data,
            ZeroCopyData::Borrowed(data) => *data,
            ZeroCopyData::Shared(data) => data.as_ref(),
        }
    }
}

impl<'a> ZeroCopyData<'a, [u8]> {
    /// バイトスライスからZeroCopyDataを作成
    pub fn from_bytes(data: &'a [u8]) -> Self {
        ZeroCopyData::Borrowed(data)
    }

    /// Vec<u8>からZeroCopyDataを作成
    pub fn from_vec(data: Vec<u8>) -> ZeroCopyData<'static, [u8]> {
        ZeroCopyData::Owned(data)
    }

    /// Arc<[u8]>からZeroCopyDataを作成
    pub fn from_arc(data: Arc<[u8]>) -> ZeroCopyData<'static, [u8]> {
        ZeroCopyData::Shared(data)
    }

    /// ZeroCopyBufferからZeroCopyDataを作成
    pub fn from_buffer(buffer: ZeroCopyBuffer) -> ZeroCopyData<'static, [u8]> {
        let bytes = buffer.into_bytes();
        ZeroCopyData::Owned(bytes.to_vec())
    }

    /// データの長さを取得
    pub fn len(&self) -> usize {
        self.deref().len()
    }

    /// データが空かどうかを確認
    pub fn is_empty(&self) -> bool {
        self.deref().is_empty()
    }

    /// データをCow<[u8]>に変換
    pub fn to_cow(&self) -> Cow<'_, [u8]> {
        match self {
            ZeroCopyData::Owned(data) => Cow::Borrowed(data),
            ZeroCopyData::Borrowed(data) => Cow::Borrowed(*data),
            ZeroCopyData::Shared(data) => Cow::Borrowed(data.as_ref()),
        }
    }
}

impl<'a> From<&'a [u8]> for ZeroCopyData<'a, [u8]> {
    fn from(data: &'a [u8]) -> Self {
        ZeroCopyData::Borrowed(data)
    }
}

impl From<Vec<u8>> for ZeroCopyData<'static, [u8]> {
    fn from(data: Vec<u8>) -> Self {
        ZeroCopyData::Owned(data)
    }
}

impl From<Arc<[u8]>> for ZeroCopyData<'static, [u8]> {
    fn from(data: Arc<[u8]>) -> Self {
        ZeroCopyData::Shared(data)
    }
}

impl From<ZeroCopyBuffer> for ZeroCopyData<'static, [u8]> {
    fn from(buffer: ZeroCopyBuffer) -> Self {
        ZeroCopyData::from_buffer(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_copy_buffer() {
        // 新しいバッファを作成
        let data = vec![1, 2, 3, 4, 5];
        let buffer = ZeroCopyBuffer::new(data.clone());

        // 内容を確認
        assert_eq!(buffer.as_bytes(), &[1, 2, 3, 4, 5]);
        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());

        // スライスを取得
        let slice = buffer.slice(1..4);
        assert_eq!(slice.as_bytes(), &[2, 3, 4]);

        // バッファを結合
        let other = ZeroCopyBuffer::new(vec![6, 7, 8]);
        let combined = buffer.concat(&other);
        assert_eq!(combined.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);

        // クローンを作成
        let clone = buffer.clone();
        assert_eq!(clone.as_bytes(), buffer.as_bytes());

        // Bytesに変換
        let bytes = buffer.into_bytes();
        assert_eq!(&bytes[..], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_zero_copy_buffer_mut() {
        // 新しい可変バッファを作成
        let mut buffer = ZeroCopyBufferMut::new(10);

        // データを追加
        buffer.put_slice(&[1, 2, 3]);
        assert_eq!(buffer.as_bytes(), &[1, 2, 3]);

        // さらにデータを追加
        buffer.put_slice(&[4, 5]);
        assert_eq!(buffer.as_bytes(), &[1, 2, 3, 4, 5]);

        // 別のバッファを追加
        let other = ZeroCopyBuffer::new(vec![6, 7, 8]);
        buffer.put_buffer(&other);
        assert_eq!(buffer.as_bytes(), &[1, 2, 3, 4, 5, 6, 7, 8]);

        // バッファをクリア
        buffer.clear();
        assert!(buffer.is_empty());

        // 新しいデータを追加
        buffer.put_slice(&[9, 10]);
        assert_eq!(buffer.as_bytes(), &[9, 10]);

        // フリーズしてZeroCopyBufferに変換
        let frozen = buffer.freeze();
        assert_eq!(frozen.as_bytes(), &[9, 10]);
    }

    #[test]
    fn test_zero_copy_data() {
        // 所有権を持つデータ
        let owned_data = vec![1, 2, 3, 4, 5];
        let owned = ZeroCopyData::from(owned_data.clone());
        assert_eq!(&*owned, &[1, 2, 3, 4, 5]);

        // 参照
        let borrowed_data = [6, 7, 8, 9, 10];
        let borrowed = ZeroCopyData::from(&borrowed_data[..]);
        assert_eq!(&*borrowed, &[6, 7, 8, 9, 10]);

        // 参照カウント
        let shared_data: Arc<[u8]> = Arc::new([11, 12, 13, 14, 15]);
        let shared = ZeroCopyData::from(shared_data.clone());
        assert_eq!(&*shared, &[11, 12, 13, 14, 15]);

        // 所有権を取得
        let owned_result = owned.into_owned();
        assert_eq!(owned_result, owned_data);

        // Cowに変換
        let borrowed_cow = borrowed.to_cow();
        assert_eq!(&*borrowed_cow, &[6, 7, 8, 9, 10]);
    }
}
