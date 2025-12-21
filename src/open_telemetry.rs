use opentelemetry::sdk::{trace, Resource};
use opentelemetry::trace::TraceError;
use opentelemetry::{runtime, KeyValue};
use opentelemetry_otlp::WithExportConfig;

pub fn init_trace() -> Result<trace::Tracer, TraceError> {
    // Use an AlwaysOn sampler during debugging so short-lived child spans
    // are exported to Jaeger reliably. For production you may want to
    // switch back to a ratio or parent-based sampler.
    let cfg = trace::config()
        .with_sampler(trace::Sampler::AlwaysOn)
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "spacetraders-rs-tracers",
        )]));

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_export_config(opentelemetry_otlp::ExportConfig {
                    timeout: std::time::Duration::from_secs(3),

                    ..Default::default()
                })
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(cfg)
        .install_batch(runtime::Tokio)
}
