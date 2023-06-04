use {
    crate::{
        bucket::Bucket, 
        // bucket_item::BucketItem, 
        bucket_map::BucketMapError,
        bucket_stats::BucketMapStats, MaxSearch, RefCount,
    },
    sdk::pubkey::Pubkey,
    std::{
        ops::RangeBounds,
        path::PathBuf,
        sync::{
            atomic::{AtomicU64, Ordering},
            Arc, RwLock, RwLockWriteGuard,
        },
    },
};

type LockedBucket<T> = RwLock<Option<Bucket<T>>>;
pub struct BucketApi<T: Clone + Copy> {
    drives: Arc<Vec<PathBuf>>,
    max_search: MaxSearch,
    pub stats: Arc<BucketMapStats>,

    bucket: LockedBucket<T>,
    count: Arc<AtomicU64>,
}
impl<T: Clone + Copy> BucketApi<T> {
    pub fn try_write(
        &self,
        pubkey: &Pubkey,
        value: (&[T], RefCount),
    ) -> Result<(), BucketMapError> {
        let mut bucket = self.get_write_bucket();
        bucket.as_mut().unwrap().try_write(pubkey, value.0, value.1)
    }

    fn get_write_bucket(&self) -> RwLockWriteGuard<Option<Bucket<T>>> {
        let mut bucket = self.bucket.write().unwrap();
        if bucket.is_none() {
            *bucket = Some(Bucket::new(
                Arc::clone(&self.drives),
                self.max_search,
                Arc::clone(&self.stats),
            ));
        } else {
            let write = bucket.as_mut().unwrap();
            write.handle_delayed_grows();
            self.count.store(write.bucket_len(), Ordering::Relaxed);
        }
        bucket
    }
}