use std::error::Error;
use tracing_chrome::ChromeLayerBuilder;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_tree::HierarchicalLayer;

pub(crate) fn init() -> Result<(), Box<dyn Error>> {
    let (chrome_layer, _guard) = ChromeLayerBuilder::new().file("chrome-trace.json").build();

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(
            HierarchicalLayer::new(4)
                .with_targets(true)
                .with_bracketed_fields(true),
        )
        .with(chrome_layer)
        .init();

    Ok(())
}
