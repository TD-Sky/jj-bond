use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{self, AtomicBool},
    },
    task::Poll,
};

use futures_util::{Stream, stream, task::AtomicWaker};
use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
    event::{ModifyKind, RenameMode},
    recommended_watcher,
};

use crate::utils::jj::JJHandle;

#[derive(Debug)]
pub struct NotifyGitChange {
    _watcher: RecommendedWatcher,
    token: Arc<Token>,
}

#[derive(Debug)]
struct Token {
    changed: AtomicBool,
    waker: AtomicWaker,
}

impl NotifyGitChange {
    pub fn new(jj_handle: JJHandle) -> notify::Result<Self> {
        let token = Arc::new(Token {
            changed: AtomicBool::new(false),
            waker: AtomicWaker::new(),
        });

        let watcher = {
            let token = token.clone();
            let mut watcher =
                recommended_watcher(move |ev_res: notify::Result<Event>| match ev_res {
                    Ok(ev)
                        if ev
                            .paths
                            .iter()
                            .any(|v| v.file_name().is_some_and(|s| s == "checkout"))
                            && matches!(
                                ev.kind,
                                EventKind::Modify(ModifyKind::Name(RenameMode::Both))
                            ) =>
                    {
                        token.changed.store(true, atomic::Ordering::Release);
                        token.waker.wake();
                    }
                    Ok(_) => {}
                    Err(e) => tracing::error!(error = %e, "watch .jj/working_copy failed"),
                })?;
            watcher.watch(
                &jj_handle.jj_joins(["working_copy"]),
                RecursiveMode::NonRecursive,
            )?;

            watcher
        };

        Ok(Self {
            _watcher: watcher,
            token,
        })
    }

    pub fn into_stream(self) -> impl Stream<Item = ()> {
        stream::unfold(self, |mut notify| async {
            {
                let notify = Pin::new(&mut notify);
                notify.await;
            };

            Some(((), notify))
        })
    }
}

impl Future for NotifyGitChange {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.token.changed.swap(false, atomic::Ordering::Acquire) {
            return Poll::Ready(());
        }

        self.token.waker.register(cx.waker());

        if self.token.changed.swap(false, atomic::Ordering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
