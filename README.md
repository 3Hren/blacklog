# Planning
- [x] MVP Pattern layout.
- [x] MVP Terminal appender.
- [x] MVP Logger.
- [ ] Fully featured pattern layout.
- [ ] Fully featured file appender.
- [x] Asynchronous logger.
- [x] Macro or compiler-plugin.
- [ ] Reset at runtime.
- [ ] Configuration from YAML.
- [ ] Integration with default singleton log crate.
- [ ] ...
- [ ] Stable Rust support.

- [ ] Implement owned timestamp generator instead of interpretation.
- [ ] Make token generator for pattern layout to act like lightweight pattern.
- [-] Severity as a Trait.
- [ ] Scoped logging (probably in conjunction with tracing sub-library).
- [?] Inflector.

# Features
- [ ] Fast it its category (prove with benchmarks with and without meta).
- [ ] Structured (examples).
- [ ] Thread-safe and clonable (example).
- [ ] Asynchronous + synchronous (benchmarks).
- [ ] Custom pattern layout (lot of examples).
- [ ] JSON.
- [ ] Colored output.
- [ ] Syslog, TCP, UDP, terminal, file (example).
- [ ] Configurable from YAML (example).
- [ ] Extendable (proof with null appender, promiscuity filter, hash layout).
- [ ] Composable (example with wrappers).
- [ ] Default Log Crate integration (example).
- [ ] Rate filter (example and use-case).
- [ ] Filters aware (example how to implement hierarchical logger).
