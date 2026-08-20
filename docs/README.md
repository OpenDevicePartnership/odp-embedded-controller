# ODP Embedded Controller documentation

This directory contains the project-level mdBook described by
[ODP RFC 0044](https://github.com/OpenDevicePartnership/governance/blob/main/rfc/0044-documentation-philosophy.md).

To build and review the book locally:

```text
mdbook serve docs --open
```

To perform a one-time build:

```text
mdbook build docs
```

The generated site is written to `docs/book/`.

The EC interface specification was migrated from the
[ODP documentation repository](https://github.com/OpenDevicePartnership/documentation/tree/main/guide_book/src/specs)
at commit `0a6b5cf70ddd1e90280d575a485184a3c1e2fd30`.
