# ONNX to Burn Conversion Tool: Development Guide

This guide offers in-depth design insights and step-by-step procedures for developers working on the
ONNX to Burn conversion tool. This tool allows the importation of ONNX models into the Burn deep
learning framework written in Rust. It converts both ONNX models to Rust source code and model
weights to Burn state files.

## Table of Contents

1. [Design Overview](#Design-Overview)
   1. [Design Goals](#Design-Goals)
   2. [Design Decisions](#Design-Decisions)
2. [Adding New Operators](#Adding-New-Operators)
3. [Testing](#Testing)
4. [Resources](#Resources)

## Design Overview

### Design Goals

- Perform best-effort conversion of ONNX models to Rust source code via Burn APIs.
- Convert ONNX model weights to Burn state files.
- Support ONNX models generated by PyTorch (ONNX Opset 16).
- Produce easy-to-understand and modifiable models.
- Ensure the generated models are trainable using Burn APIs.

### Design Decisions

- Limit interaction with ONNX to the Intermediate Representation (IR) stage to simplify the process.
- Ensure operator behavior consistency across different OpSet versions.
- Exclude any ONNX/Protobuf-specific logic from the Burn graph.

The conversion process involves three main stages:

1. Convert ONNX model to Intermediate Representation (IR).
2. Translate IR to a Burn graph.
3. Generate Rust source code from the Burn graph.

## Adding New Operators

To extend `burn-import` with support for new ONNX operators, follow these steps:

1. **Create PyTorch Script**: Place a PyTorch script using the new operator under
   `./burn-import/onnx-tests/tests/<op>/<op>.py`. Make sure to print both input and output tensors
   for end-to-end testing.

2. **Generate ONNX Model**: Run the PyTorch script to produce an ONNX model.

3. **Visualize ONNX Model**: Use [Netron](https://github.com/lutzroeder/netron) to verify the ONNX
   model contains the expected operators.

4. **Generate IR and Burn Graph**: Navigate to `./burn-import/` and run:

   ```
   cargo r -- ./onnx-tests/tests/<op>/<op>.onnx ./out
   ```

5. **Implement Missing Operators**: If you encounter an error stating that an operator is
   unsupported, implement it. The `./out/my-model.graph.txt` should provide relevant information.

6. **Inspect Generated Files**: The `my-model.graph.txt` contains IR details, `my-model.rs` holds
   the Burn model in Rust code, and `my-model.json` includes the model data.

7. **Add End-to-End Test**: Include the test in `./burn-import/onnx-tests/tests/onnx_tests.rs`.
   Further details can be found in the [onnx-tests README](./onnx-tests/README.md).

## Testing

- Unit tests for the Burn graph to Rust source code conversion are mandatory.
- End-to-end tests should include a test ONNX model and its expected output for each operator.

## Resources

1. [PyTorch to ONNX](https://pytorch.org/docs/stable/onnx.html)
2. [ONNX to PyTorch](https://github.com/ENOT-AutoDL/onnx2torch)
3. [ONNX Introduction](https://onnx.ai/onnx/intro/)
4. [ONNX Operators](https://onnx.ai/onnx/operators/index.html)
5. [ONNX Protos](https://onnx.ai/onnx/api/classes.html)
6. [ONNX Optimizer](https://github.com/onnx/optimizer)
7. [Netron](https://github.com/lutzroeder/netron)
