use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;
use std::sync::Arc;

/// メモリアリーナ
/// 
/// 高速なメモリ割り当てのためのアリーナアロケータ。
/// 同じサイズのオブジェクトを効率的に割り当てるために使用する。
pub struct Arena<T> {
    /// 現在のブロック
    current_block: RefCell<ArenaBlock<T>>,
    /// ブロックサイズ
    block_size: usize,
    /// 割り当てられたオブジェクトの数
    allocated: RefCell<usize>,
    /// 解放されたオブジェクトの数
    freed: RefCell<usize>,
}

/// アリーナブロック
struct ArenaBlock<T> {
    /// メモリブロック
    memory: NonNull<u8>,
    /// レイアウト
    layout: Layout,
    /// 次の空き位置
    next_free: usize,
    /// 容量
    capacity: usize,
    /// ファントムデータ
    _phantom: PhantomData<T>,
}

impl<T> Arena<T> {
    /// 新しいArenaを作成
    pub fn new(block_size: Option<usize>) -> Self {
        let block_size = block_size.unwrap_or(1024);
        
        Self {
            current_block: RefCell::new(ArenaBlock::new(block_size)),
            block_size,
            allocated: RefCell::new(0),
            freed: RefCell::new(0),
        }
    }
    
    /// オブジェクトを割り当て
    pub fn allocate(&self, value: T) -> &mut T {
        let mut current_block = self.current_block.borrow_mut();
        
        // 現在のブロックに空きがなければ新しいブロックを作成
        if current_block.is_full() {
            let new_block = ArenaBlock::new(self.block_size);
            *current_block = new_block;
        }
        
        // オブジェクトを割り当て
        let ptr = current_block.allocate(value);
        
        // 割り当て数をインクリメント
        *self.allocated.borrow_mut() += 1;
        
        // 安全でない操作：生ポインタを参照に変換
        unsafe { &mut *ptr }
    }
    
    /// 割り当てられたオブジェクトの数を取得
    pub fn allocated_count(&self) -> usize {
        *self.allocated.borrow()
    }
    
    /// 解放されたオブジェクトの数を取得
    pub fn freed_count(&self) -> usize {
        *self.freed.borrow()
    }
    
    /// 使用中のオブジェクトの数を取得
    pub fn active_count(&self) -> usize {
        self.allocated_count() - self.freed_count()
    }
}

impl<T> ArenaBlock<T> {
    /// 新しいArenaBlockを作成
    fn new(capacity: usize) -> Self {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        
        // メモリレイアウトを作成
        let layout = Layout::from_size_align(size * capacity, align)
            .expect("Invalid layout");
        
        // メモリを割り当て
        let memory = unsafe {
            NonNull::new(alloc(layout)).expect("Memory allocation failed")
        };
        
        Self {
            memory,
            layout,
            next_free: 0,
            capacity,
            _phantom: PhantomData,
        }
    }
    
    /// オブジェクトを割り当て
    fn allocate(&mut self, value: T) -> *mut T {
        assert!(!self.is_full(), "Arena block is full");
        
        let size = mem::size_of::<T>();
        let offset = self.next_free * size;
        
        // 次の空き位置を計算
        self.next_free += 1;
        
        // オブジェクトのポインタを計算
        let ptr = unsafe {
            self.memory.as_ptr().add(offset) as *mut T
        };
        
        // オブジェクトを書き込み
        unsafe {
            ptr.write(value);
        }
        
        ptr
    }
    
    /// ブロックが満杯かどうかを確認
    fn is_full(&self) -> bool {
        self.next_free >= self.capacity
    }
}

impl<T> Drop for ArenaBlock<T> {
    fn drop(&mut self) {
        // ブロック内のすべてのオブジェクトを破棄
        let size = mem::size_of::<T>();
        
        for i in 0..self.next_free {
            let offset = i * size;
            let ptr = unsafe {
                self.memory.as_ptr().add(offset) as *mut T
            };
            
            // オブジェクトを破棄
            unsafe {
                ptr.drop_in_place();
            }
        }
        
        // メモリを解放
        unsafe {
            dealloc(self.memory.as_ptr(), self.layout);
        }
    }
}

/// 共有アリーナ
/// 
/// 複数のスレッドで共有できるアリーナ。
pub struct SharedArena<T> {
    /// 内部アリーナ
    inner: Arc<Arena<T>>,
}

impl<T> SharedArena<T> {
    /// 新しいSharedArenaを作成
    pub fn new(block_size: Option<usize>) -> Self {
        Self {
            inner: Arc::new(Arena::new(block_size)),
        }
    }
    
    /// オブジェクトを割り当て
    pub fn allocate(&self, value: T) -> &mut T {
        self.inner.allocate(value)
    }
    
    /// 割り当てられたオブジェクトの数を取得
    pub fn allocated_count(&self) -> usize {
        self.inner.allocated_count()
    }
    
    /// 解放されたオブジェクトの数を取得
    pub fn freed_count(&self) -> usize {
        self.inner.freed_count()
    }
    
    /// 使用中のオブジェクトの数を取得
    pub fn active_count(&self) -> usize {
        self.inner.active_count()
    }
}

impl<T> Clone for SharedArena<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena_allocation() {
        let arena = Arena::<u32>::new(Some(10));
        
        // オブジェクトを割り当て
        let a = arena.allocate(1);
        let b = arena.allocate(2);
        let c = arena.allocate(3);
        
        // 値を確認
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);
        
        // 割り当て数を確認
        assert_eq!(arena.allocated_count(), 3);
        assert_eq!(arena.freed_count(), 0);
        assert_eq!(arena.active_count(), 3);
        
        // 値を変更
        *a = 10;
        *b = 20;
        *c = 30;
        
        // 変更後の値を確認
        assert_eq!(*a, 10);
        assert_eq!(*b, 20);
        assert_eq!(*c, 30);
    }
    
    #[test]
    fn test_arena_block_overflow() {
        let arena = Arena::<String>::new(Some(2));
        
        // 最初のブロックを満杯にする
        let a = arena.allocate("a".to_string());
        let b = arena.allocate("b".to_string());
        
        // 新しいブロックが作成される
        let c = arena.allocate("c".to_string());
        
        // 値を確認
        assert_eq!(*a, "a");
        assert_eq!(*b, "b");
        assert_eq!(*c, "c");
        
        // 割り当て数を確認
        assert_eq!(arena.allocated_count(), 3);
    }
    
    #[test]
    fn test_shared_arena() {
        let arena = SharedArena::<u32>::new(Some(10));
        
        // オブジェクトを割り当て
        let a = arena.allocate(1);
        let b = arena.allocate(2);
        
        // 値を確認
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        
        // クローンを作成
        let arena_clone = arena.clone();
        
        // クローンからオブジェクトを割り当て
        let c = arena_clone.allocate(3);
        
        // 値を確認
        assert_eq!(*c, 3);
        
        // 割り当て数を確認（共有されているため合計は3）
        assert_eq!(arena.allocated_count(), 3);
        assert_eq!(arena_clone.allocated_count(), 3);
    }
}