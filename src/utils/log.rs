use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, Layer as _, Registry, layer::SubscriberExt, util::SubscriberInitExt,
};

pub fn init_log() {
    let file = {
        let dir = dirs::state_dir()
            .expect("cannot find state directory")
            .join("jj-bond");
        std::fs::create_dir_all(&dir).expect("failed to create log directory");

        let builder = RollingFileAppender::builder()
            .rotation(Rotation::HOURLY)
            .filename_prefix("jj-bond");
        let file_appender = builder
            .build(dir)
            .expect("initializing rolling self appender failed");

        let filter = EnvFilter::builder()
            .with_default_directive(Level::DEBUG.into())
            .from_env_lossy();

        tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(file_appender)
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_filter(filter)
    };

    Registry::default().with(file).init();
}
