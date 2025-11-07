use crate::database::DbPool;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Queue manager for AI requests
pub struct QueueManager {
    pool: DbPool,
    max_concurrent: Arc<Mutex<i32>>,
    current_running: Arc<Mutex<i32>>,
    pending_queue: Arc<Mutex<VecDeque<i64>>>,
}

impl QueueManager {
    pub fn new(pool: DbPool, max_concurrent: i32) -> Self {
        Self {
            pool,
            max_concurrent: Arc::new(Mutex::new(max_concurrent)),
            current_running: Arc::new(Mutex::new(0)),
            pending_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
    
    /// Update max concurrent limit
    pub fn set_max_concurrent(&self, max: i32) {
        *self.max_concurrent.lock().unwrap() = max;
    }
    
    /// Add a request to the queue
    pub fn enqueue(&self, message: &str, agent_name: Option<&str>) -> Result<i64> {
        let queue_id = crate::database::add_queue_item(&self.pool, message, agent_name)?;
        
        // Check if we can process immediately
        let can_process = {
            let current = *self.current_running.lock().unwrap();
            let max = *self.max_concurrent.lock().unwrap();
            current < max
        };
        
        if can_process {
            // Will be processed immediately (no need to do anything, will be processed)
        } else {
            // Add to pending queue
            self.pending_queue.lock().unwrap().push_back(queue_id);
        }
        
        Ok(queue_id)
    }
    
    /// Check if a request can be processed
    pub fn can_process(&self) -> bool {
        let current = *self.current_running.lock().unwrap();
        let max = *self.max_concurrent.lock().unwrap();
        current < max
    }
    
    /// Mark a request as started
    pub fn start_processing(&self, queue_id: i64) -> Result<()> {
        crate::database::update_queue_item_status(&self.pool, queue_id, "processing", None)?;
        *self.current_running.lock().unwrap() += 1;
        Ok(())
    }
    
    /// Mark a request as completed
    pub fn complete(&self, queue_id: i64, response: &str) -> Result<()> {
        crate::database::update_queue_item_status(&self.pool, queue_id, "completed", Some(response))?;
        *self.current_running.lock().unwrap() -= 1;
        
        // Process next pending item if available
        let _ = self.pending_queue.lock().unwrap().pop_front();
        
        Ok(())
    }
    
    /// Mark a request as failed
    pub fn fail(&self, queue_id: i64, error: &str) -> Result<()> {
        crate::database::update_queue_item_status(&self.pool, queue_id, "failed", Some(error))?;
        *self.current_running.lock().unwrap() -= 1;
        
        // Process next pending item if available
        let _ = self.pending_queue.lock().unwrap().pop_front();
        
        Ok(())
    }
    
    /// Get next pending item ID (if available and can process)
    pub fn get_next_pending(&self) -> Option<i64> {
        if !self.can_process() {
            return None;
        }
        
        self.pending_queue.lock().unwrap().pop_front()
    }
}

/// Global queue manager instance
static QUEUE_MANAGER: std::sync::OnceLock<Arc<Mutex<Option<Arc<QueueManager>>>>> = std::sync::OnceLock::new();

/// Initialize the queue manager
pub fn init_queue_manager(pool: DbPool, max_concurrent: i32) {
    let manager = Arc::new(QueueManager::new(pool, max_concurrent));
    QUEUE_MANAGER.get_or_init(|| Arc::new(Mutex::new(Some(manager))));
}

/// Get the queue manager instance
pub fn get_queue_manager() -> Result<Arc<QueueManager>> {
    let manager_guard = QUEUE_MANAGER
        .get_or_init(|| Arc::new(Mutex::new(None)))
        .lock()
        .unwrap();
    
    manager_guard
        .as_ref()
        .ok_or_else(|| anyhow!("Queue manager not initialized"))
        .map(|m| Arc::clone(m))
}

