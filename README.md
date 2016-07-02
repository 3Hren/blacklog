# Planning for 0.2.0
- [ ] Fully featured pattern layout.
  - [x] Proof of concept.
  - [x] Metalist placeholder.
  - [x] Precision for floats.
  - [x] +/#/0 flags.
  - [x] Precision for meta.
  - [x] Hex, Bin, Debug, etc. types for meta placeholders.
  - [ ] Optional meta.
  - [ ] Module, line, thread placeholders.
- [ ] Layouts.
  - [ ] Pattern.
  - [ ] JSON.
- [ ] Outputs
  - [ ] Null.
  - [ ] Term.
  - [ ] File (with routing).
  - [ ] TCP.
  - [ ] UDP.  
- [ ] Filters.
  - [ ] Whatever.
  - [ ] Severity.
  - [ ] Burst.
  - [ ] Composite.
- [ ] Loggers.
  - [ ] Sync.
  - [ ] Async logger.
    - [x] Proof of concept.
    - [ ] Finish.
  - [ ] Runtime severity threshold change.
  - [ ] Runtime filter change.
  - [ ] Reset at runtime.
- [ ] Macro or compiler-plugin.
  - [x] Proof of concept.
  - [ ] Compiler plugin.
- [ ] External configuration.
  - [x] From JSON.
  - [ ] Proper error variants, no unwraps.
- [ ] Integration with default log crate.
- [ ] Builder pattern for each category.
- [ ] Stable Rust support.

- [ ] Implement owned timestamp generator instead of interpretation.
- [ ] Make token generator for pattern layout to act like lightweight pattern.
- [-] Severity as a Trait.
- [ ] Scoped logging (probably in conjunction with tracing sub-library).
- [?] Inflector.

# Features
- [ ] Fast in its category (prove with benchmarks with and without meta).
- [ ] Structured (examples).
- [ ] Thread-safe and clonable (example).
- [ ] Asynchronous + synchronous (benchmarks).
- [ ] Custom pattern layout (lot of examples).
- [ ] JSON.
- [ ] Colored output.
- [ ] Syslog, TCP, UDP, term, file (example).
- [ ] Configurable from YAML (example).
- [ ] Extendable (proof with null appender, promiscuity filter, hash layout).
- [ ] Composable (example with wrappers).
- [ ] Default Log Crate integration (example).
- [ ] Rate filter (example and use-case).
- [ ] Filters aware (example how to implement hierarchical logger).
