# linlog

A linear logic suite for all your needs.

## Roadmap

Currently planned are a command line program as well as an interactive website (hosted somewhere as a client side website) that provide some functionality of interacting with linear logic sequents and proofs. This tool is mainly meant for educational purposes, but since a major goal is the use of efficient code (within the limits of complexity classes), the tool should be usable for various other use cases.

Please feel free to make feature requests and contribute in any way. All of the code in this repository is licensed under the EUPL.

Planned features:

- Parse and print sequents
- Step-wise interactive sequent proving
- Automatic proof search (different methods for different sub-variants, e.g. MLL)
- Creation and verification of proof nets
- Convert classical/intuitionistic sequents to linear logic
    - solve them and compare proof trees
- Output sequents, proof trees and nets in various formats:
    - PDF/SVG
    - LaTeX/Typst
    - plain Unicode
    - Rocq/Lean/Agda proof
    - interactive web-view
- deeply inspired by [Click and Collect](https://www.click-and-collect.linear-logic.org), but more features planned

## Architecture

There will be three units: the CLI program (`linlog-cli`), the web version (`linlog-web`) as well as a library (`linlog-core`) for the common logic shared between the CLI and web application. The code is written using Rust, due to its high performance and great compatibiliy with WebAssembly (for the website).
