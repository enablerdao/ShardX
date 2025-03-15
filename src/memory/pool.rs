use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

/// メモリプール
/// 
/// 同じサイズのオブジェクトを効率的に再利用するためのメモリプール。
/// アロケーションのオーバーヘッドを削減し、メモリフラグメンテーションを防ぐ。
pub struct MemoryPool<T> {
    /// フリーリスト
    free_list: Mutex<Option<NonNull<FreeNode>>>,
    /// チャンクリスト
    chunks: Mutex<Vec<NonNull<u8>>>,
    /// チャンクサイズ
    chunk_size: usize,
    /// チャンク内のオブジェクト数
    objects_per_chunk: usize,
    /// 割り当てられたオブジェクトの数
    allocated: Mutex<usize>,
    /// 解放されたオブジェクトの数
    freed: Mutex<usize>,
    /// ファントムデータ
    _phantom: PhantomData<T>,
}

/// フリーノード
struct FreeNode {
    /// 次のノード
    next: Option<NonNull<FreeNode>>,
}

/// プールから割り当てられたオブジェクト
pub struct PooledObject<T> {
    /// オブジェクトへのポインタ
    ptr: NonNull<T>,
    /// プールへの参照
    pool: Arc<MemoryPool<T>>,
}

unsafe impl<T: Send> Send for MemoryPool<T> {}
unsafe impl<T: Sync> Sync for MemoryPool<T> {}

impl<T> MemoryPool<T> {
    /// 新しいMemoryPoolを作成
    pub fn new(chunk_size: Option<usize>) -> Arc<Self> {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        
        // フリーノードのサイズを考慮
        let node_size = std::cmp::max(size, mem::size_of::<FreeNode>());
        
        // チャンクサイズを決定
        let chunk_size = chunk_size.unwrap_or(4096);
        let objects_per_chunk = chunk_size / node_size;
        
        assert!(objects_per_chunk > 0, "Chunk size too small for object type");
        
        Arc::new(Self {
            free_list: Mutex::new(None),
            chunks: Mutex::new(Vec::new()),
            chunk_size,
            objects_per_chunk,
            allocated: Mutex::new(0),
            freed: Mutex::new(0),
            _phantom: PhantomData,
        })
    }
    
    /// オブジェクトを割り当て
    pub fn allocate(self: &Arc<Self>, value: T) -> PooledObject<T> {
        // フリーリストからノードを取得
        let ptr = {
            let mut free_list = self.free_list.lock().unwrap();
            
            match *free_list {
                Some(node) => {
                    // フリーリストからノードを取り出す
                    let next = unsafe { (*node.as_ptr()).next };
                    *free_list = next;
                    
                    // ノードをTのポインタに変換
                    NonNull::new(node.as_ptr() as *mut T).unwrap()
                },
                None => {
                    // 新しいチャンクを割り当て
                    self.allocate_chunk();
                    
                    // 再帰的に割り当て
                    return self.allocate(value);
                }
            }
        };
        
        // 割り当て数をインクリメント
        *self.allocated.lock().unwrap() += 1;
        
        // オブジェクトを書き込み
        unsafe {
            ptr.as_ptr().write(value);
        }
        
        PooledObject {
            ptr,
            pool: self.clone(),
        }
    }
    
    /// 新しいチャンクを割り当て
    fn allocate_chunk(&self) {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        
        // フリーノードのサイズを考慮
        let node_size = std::cmp::max(size, mem::size_of::<FreeNode>());
        
        // チャンクのレイアウトを作成
        let layout = Layout::from_size_align(self.chunk_size, align)
            .expect("Invalid layout");
        
        // チャンクを割り当て
        let chunk = unsafe {
            NonNull::new(alloc(layout)).expect("Memory allocation failed")
        };
        
        // チャンクをフリーリストに追加
        let mut free_list = self.free_list.lock().unwrap();
        
        for i in 0..self.objects_per_chunk {
            let offset = i * node_size;
            let node_ptr = unsafe {
                chunk.as_ptr().add(offset) as *mut FreeNode
            };
            
            // フリーノードを初期化
            let node = NonNull::new(node_ptr).unwrap();
            unsafe {
                (*node.as_ptr()).next = *free_list;
            }
            
            *free_list = Some(node);
        }
        
        // チャンクリストに追加
        self.chunks.lock().unwrap().push(chunk);
    }
    
    /// オブジェクトを解放
    fn free(&self, ptr: NonNull<T>) {
        // オブジェクトを破棄
        unsafe {
            ptr.as_ptr().drop_in_place();
        }
        
        // フリーリストに追加
        let node_ptr = ptr.as_ptr() as *mut FreeNode;
        let node = NonNull::new(node_ptr).unwrap();
        
        let mut free_list = self.free_list.lock().unwrap();
        unsafe {
            (*node.as_ptr()).next = *free_list;
        }
        *free_list = Some(node);
        
        // 解放数をインクリメント
        *self.freed.lock().unwrap() += 1;
    }
    
    /// 割り当てられたオブジェクトの数を取得
    pub fn allocated_count(&self) -> usize {
        *self.allocated.lock().unwrap()
    }
    
    /// 解放されたオブジェクトの数を取得
    pub fn freed_count(&self) -> usize {
        *self.freed.lock().unwrap()
    }
    
    /// 使用中のオブジェクトの数を取得
    pub fn active_count(&self) -> usize {
        self.allocated_count() - self.freed_count()
    }
}

impl<T> Drop for MemoryPool<T> {
    fn drop(&mut self) {
        // すべてのチャンクを解放
        let chunks = std::mem::take(&mut *self.chunks.lock().unwrap());
        
        for chunk in chunks {
            let layout = Layout::from_size_align(self.chunk_size, mem::align_of::<T>())
                .expect("Invalid layout");
            
            unsafe {
                dealloc(chunk.as_ptr(), layout);
            }
        }
    }
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr.as_ptr() }
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        // オブジェクトをプールに返却
        self.pool.free(self.ptr);
    }
}

/// オブジェクトプール
/// 
/// 特定の型のオブジェクトを再利用するためのプール。
pub struct ObjectPool<T> {
    /// メモリプール
    memory_pool: Arc<MemoryPool<UnsafeCell<T>>>,
}

impl<T> ObjectPool<T> {
    /// 新しいObjectPoolを作成
    pub fn new(chunk_size: Option<usize>) -> Self {
        Self {
            memory_pool: MemoryPool::new(chunk_size),
        }
    }
    
    /// オブジェクトを取得
    pub fn get(&self, value: T) -> PooledRef<T> {
        let cell = self.memory_pool.allocate(UnsafeCell::new(value));
        PooledRef { cell }
    }
    
    /// 割り当てられたオブジェクトの数を取得
    pub fn allocated_count(&self) -> usize {
        self.memory_pool.allocated_count()
    }
    
    /// 解放されたオブジェクトの数を取得
    pub fn freed_count(&self) -> usize {
        self.memory_pool.freed_count()
    }
    
    /// 使用中のオブジェクトの数を取得
    pub fn active_count(&self) -> usize {
        self.memory_pool.active_count()
    }
}

/// プールから取得したオブジェクトへの参照
pub struct PooledRef<T> {
    /// UnsafeCellへの参照
    cell: PooledObject<UnsafeCell<T>>,
}

impl<T> std::ops::Deref for PooledRef<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.get() }
    }
}

impl<T> std::ops::DerefMut for PooledRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell.get() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::<u32>::new(None);
        
        // オブジェクトを割り当て
        let a = pool.allocate(1);
        let b = pool.allocate(2);
        let c = pool.allocate(3);
        
        // 値を確認
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
        
        // 割り当て数を確認
        assert_eq!(pool.allocated_count(), 3);
        assert_eq!(pool.freed_count(), 0);
        assert_eq!(pool.active_count(), 3);
        
        // 値を変更
        *a = 10;
        *b = 20;
        *c = 30;
        
        // 変更後の値を確認
        assert_eq!(*a, 10);
        assert_eq!(*b, 20);
        assert_eq!(*c, 30);
        
        // オブジェクトを解放
        drop(a);
        
        // 解放後の割り当て数を確認
        assert_eq!(pool.allocated_count(), 3);
        assert_eq!(pool.freed_count(), 1);
        assert_eq!(pool.active_count(), 2);
        
        // 新しいオブジェクトを割り当て（解放されたメモリが再利用される）
        let d = pool.allocate(4);
        
        // 値を確認
        assert_eq!(*d, 4);
        
        // 割り当て数を確認
        assert_eq!(pool.allocated_count(), 4);
        assert_eq!(pool.freed_count(), 1);
        assert_eq!(pool.active_count(), 3);
    }
    
    #[test]
    fn test_object_pool() {
        let pool = ObjectPool::<String>::new(None);
        
        // オブジェクトを取得
        let mut a = pool.get("a".to_string());
        let mut b = pool.get("b".to_string());
        
        // 値を確認
        assert_eq!(*a, "a");
        assert_eq!(*b, "b");
        
        // 値を変更
        *a = "A".to_string();
        *b = "B".to_string();
        
        // 変更後の値を確認
        assert_eq!(*a, "A");
        assert_eq!(*b, "B");
        
        // 割り当て数を確認
        assert_eq!(pool.allocated_count(), 2);
        assert_eq!(pool.freed_count(), 0);
        assert_eq!(pool.active_count(), 2);
        
        // オブジェクトを解放
        drop(a);
        
        // 解放後の割り当て数を確認
        assert_eq!(pool.allocated_count(), 2);
        assert_eq!(pool.freed_count(), 1);
        assert_eq!(pool.active_count(), 1);
        
        // 新しいオブジェクトを取得（解放されたメモリが再利用される）
        let c = pool.get("c".to_string());
        
        // 値を確認
        assert_eq!(*c, "c");
        
        // 割り当て数を確認
        assert_eq!(pool.allocated_count(), 3);
        assert_eq!(pool.freed_count(), 1);
        assert_eq!(pool.active_count(), 2);
    }
}