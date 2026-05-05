use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Graceful shutdown manager that handles SIGINT/SIGTERM signals
/// and coordinates shutdown across all components.
pub struct GracefulShutdown {
    /// Token that cancels when shutdown is requested
    cancel_token: CancellationToken,
    /// Flag indicating shutdown was requested
    shutdown_requested: Arc<AtomicBool>,
    /// Broadcast channel for shutdown events
    #[allow(dead_code)]
    shutdown_tx: broadcast::Sender<()>,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown manager and start listening for signals.
    /// This spawns a background task to handle OS signals.
    pub fn new() -> Self {
        let cancel_token = CancellationToken::new();
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let (shutdown_tx, _) = broadcast::channel(1);

        let cancel_clone = cancel_token.clone();
        let shutdown_flag = shutdown_requested.clone();
        let tx = shutdown_tx.clone();

        // Spawn signal handler task
        tokio::spawn(async move {
            use tokio::signal::unix::{signal, SignalKind};

            // Register for SIGINT and SIGTERM
            let mut sigint = match signal(SignalKind::interrupt()) {
                Ok(s) => s,
                Err(e) => {
                    warn!(error = %e, "Failed to register SIGINT handler");
                    return;
                }
            };

            let mut sigterm = match signal(SignalKind::terminate()) {
                Ok(s) => s,
                Err(e) => {
                    warn!(error = %e, "Failed to register SIGTERM handler");
                    return;
                }
            };

            tokio::select! {
                _ = sigint.recv() => {
                    info!("Received SIGINT (Ctrl+C)");
                }
                _ = sigterm.recv() => {
                    info!("Received SIGTERM");
                }
                _ = cancel_clone.cancelled() => {
                    // Already cancelled by someone else
                    return;
                }
            }

            shutdown_flag.store(true, Ordering::SeqCst);
            let _ = tx.send(());
            cancel_clone.cancel();
        });

        Self {
            cancel_token,
            shutdown_requested,
            shutdown_tx,
        }
    }

    /// Get the cancellation token for passing to tasks
    pub fn token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// Check if shutdown was requested
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }

}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_graceful_shutdown_creation() {
        // Just verify we can create a GracefulShutdown without panicking
        // In test environment, signal handlers may not be available
        let _shutdown = GracefulShutdown::new();
    }

    #[tokio::test]
    async fn test_graceful_shutdown_is_not_requested_initially() {
        let shutdown = GracefulShutdown::new();
        assert!(!shutdown.is_shutdown_requested());
    }

    #[tokio::test]
    async fn test_cancellation_token_clone() {
        let shutdown = GracefulShutdown::new();
        let token = shutdown.token();
        // Token should be cloneable
        let _token_clone = token.clone();
    }

    #[tokio::test]
    async fn test_cancellation() {
        let shutdown = GracefulShutdown::new();
        let token = shutdown.token();
        assert!(!token.is_cancelled());
        token.cancel();
        assert!(token.is_cancelled());
    }
}
