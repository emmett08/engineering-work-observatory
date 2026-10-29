# ADR 0001: Explicit Rust collection and accounting pipeline

Status: accepted for the initial implementation.

## Context

The application collects GitHub and local EAL/EngArg evidence, retains immutable study records in SQLite, and calculates effort, acceptance and retained-output summaries under a frozen protocol. Collection retries, incomplete pagination, duplicate calls, missing usage and analysis cut-offs can change the meaning of a result. Those decisions need explicit contracts and focused tests.

The user requested Rust and asked whether [Aspect](https://aspect-rs.org/) would be a suitable design choice. The assessment below concerns inspected source revision [`89beaa9057b3f2b73093fc31d3219420e0bb6182`](https://github.com/yijunyu/aspect-rs/tree/89beaa9057b3f2b73093fc31d3219420e0bb6182). It is not a claim about every version or every use of aspect-oriented programming.

## Decision

Use typed Rust modules and explicit functions for collection, imports, validation, transactions, snapshots and calculations. Use blocking `reqwest` for the local command workflow, bundled SQLite through `rusqlite`, content-addressed raw evidence, and `tracing` for diagnostic instrumentation. Pin Rust 1.99.1 and commit the application lockfile. Preserve the repository's GPL license.

Do not add Aspect as a core dependency. Retry, cache, transaction and evidence-completeness policies remain explicit in their owning modules. This is ordinary modular code; it does not require a generic plugin framework or a trait for every module.

## Aspect assessment

Aspect modularises cross-cutting function behaviour such as logging and timing. Its workspace is version `0.1.0`, Rust 2021. The attribute-macro approach is documented for stable Rust; its optional compiler-driver package separately requires nightly and `rustc-dev`. [Workspace manifest](https://github.com/yijunyu/aspect-rs/blob/89beaa9057b3f2b73093fc31d3219420e0bb6182/Cargo.toml), [driver manifest](https://github.com/yijunyu/aspect-rs/blob/89beaa9057b3f2b73093fc31d3219420e0bb6182/aspect-rustc-driver/Cargo.toml).

Three inspected details make it unsuitable for the collector's correctness policies:

1. The standard caching aspect retains configuration but its `around` method prints a cache message and always proceeds. It does not implement the advertised cache behaviour. [Caching implementation](https://github.com/yijunyu/aspect-rs/blob/89beaa9057b3f2b73093fc31d3219420e0bb6182/aspect-std/src/caching.rs).
2. Generated async wrappers call `before`, await the function, and call `after` or `after_error`; they bypass `around`. An `around` policy therefore does not have the same contract across synchronous and async functions. [Code generator](https://github.com/yijunyu/aspect-rs/blob/89beaa9057b3f2b73093fc31d3219420e0bb6182/aspect-macros/src/codegen.rs).
3. Generated synchronous `Result` handling formats the original error into an `AspectError` and string-converts the advice error back into the declared return error. The observatory needs to preserve distinctions such as rate limiting, source unavailability and invalid records. [Same code generator](https://github.com/yijunyu/aspect-rs/blob/89beaa9057b3f2b73093fc31d3219420e0bb6182/aspect-macros/src/codegen.rs).

These source findings are more relevant here than advertised test counts or zero-overhead claims. Aspect also does not supply the observatory's domain schema, provenance or human-effort observations.

## Alternatives and consequences

| Alternative | Benefit | Cost for this application |
|---|---|---|
| One Rust file or script | Fast initial prototype | External formats, persistence and accounting rules would change together; isolated testing and review become harder. |
| Explicit local Rust modules | Narrow change boundaries, direct control flow, reproducible local operation | Some logging and error context are written explicitly; one process is not a multi-user service. |
| Aspect-centred implementation | Declarative cross-cutting instrumentation | Additional generated semantics and the inspected limitations would require validation without replacing the domain work. |
| Hosted service and server database | Concurrent users, remote intake forms and event delivery | Deployment, authentication, scheduling and operations exceed the current local workflow. |

Choose the explicit local application. Keep raw evidence separate from typed records and derived snapshots so adapters can evolve without rewriting the measurements. Tests must cover real failure boundaries rather than merely asserting module structure.

An optional Aspect experiment remains possible for a bounded diagnostic function after checking expanded code and behaviour. It must not determine task identity, labour, collection completeness, deduplication or accounting results. Revisit hosting when concurrent operators need it, and revisit this decision if a measured maintenance problem makes another design preferable.
