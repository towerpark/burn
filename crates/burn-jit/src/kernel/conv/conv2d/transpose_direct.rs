use cubecl::{
    cpa,
    ir::{
        Builtin, Elem, IntKind, Item, KernelDefinition, Scope, Variable, VariableKind, Visibility,
    },
    prelude::*,
    CubeCountSettings, Execution, InputInfo, KernelExpansion, KernelIntegrator, KernelSettings,
    OutputInfo,
};
use std::marker::PhantomData;

use crate::{
    element::JitElement,
    kernel::{self, Kernel},
    ops::{
        numeric::{empty_device, zeros_device},
        reshape,
    },
    tensor::JitTensor,
    IntElement, JitRuntime,
};
use burn_tensor::{ops::ConvTransposeOptions, Shape};

#[derive(new)]
struct Conv2dTransposeEagerKernel<R, E> {
    _runtime: PhantomData<R>,
    _elem: PhantomData<E>,
}

#[derive(new)]
struct Conv2dTransposeComputeShader<E> {
    input: Variable,
    weight: Variable,
    bias: Variable,
    output: Variable,
    _elem: PhantomData<E>,
}

impl<E: JitElement> Conv2dTransposeComputeShader<E> {
    fn expand(self, scope: &mut Scope) {
        let input = self.input;
        let weight = self.weight;
        let bias = self.bias;
        let output = self.output;
        let id = Variable::builtin(Builtin::AbsolutePos);

        let input_stride_0 = scope.create_local(u32::as_elem());
        let input_stride_1 = scope.create_local(u32::as_elem());
        let input_stride_2 = scope.create_local(u32::as_elem());
        let input_stride_3 = scope.create_local(u32::as_elem());
        let input_shape_0 = scope.create_local(u32::as_elem());
        let input_shape_1 = scope.create_local(u32::as_elem());
        let input_shape_2 = scope.create_local(u32::as_elem());
        let input_shape_3 = scope.create_local(u32::as_elem());
        cpa!(scope, input_stride_0 = stride(input, 0u32));
        cpa!(scope, input_stride_1 = stride(input, 1u32));
        cpa!(scope, input_stride_2 = stride(input, 2u32));
        cpa!(scope, input_stride_3 = stride(input, 3u32));
        cpa!(scope, input_shape_0 = shape(input, 0u32));
        cpa!(scope, input_shape_1 = shape(input, 1u32));
        cpa!(scope, input_shape_2 = shape(input, 2u32));
        cpa!(scope, input_shape_3 = shape(input, 3u32));

        let output_stride_0 = scope.create_local(u32::as_elem());
        let output_stride_1 = scope.create_local(u32::as_elem());
        let output_stride_2 = scope.create_local(u32::as_elem());
        let output_stride_3 = scope.create_local(u32::as_elem());
        let output_shape_0 = scope.create_local(u32::as_elem());
        let output_shape_1 = scope.create_local(u32::as_elem());
        let output_shape_2 = scope.create_local(u32::as_elem());
        let output_shape_3 = scope.create_local(u32::as_elem());
        cpa!(scope, output_stride_0 = stride(output, 0u32));
        cpa!(scope, output_stride_1 = stride(output, 1u32));
        cpa!(scope, output_stride_2 = stride(output, 2u32));
        cpa!(scope, output_stride_3 = stride(output, 3u32));
        cpa!(scope, output_shape_0 = shape(output, 0u32));
        cpa!(scope, output_shape_1 = shape(output, 1u32));
        cpa!(scope, output_shape_2 = shape(output, 2u32));
        cpa!(scope, output_shape_3 = shape(output, 3u32));

        let weight_stride_0 = scope.create_local(u32::as_elem());
        let weight_stride_1 = scope.create_local(u32::as_elem());
        let weight_stride_2 = scope.create_local(u32::as_elem());
        let weight_stride_3 = scope.create_local(u32::as_elem());
        let in_channels = scope.create_local(u32::as_elem());
        let weight_shape_1 = scope.create_local(u32::as_elem());
        let kernel_size_0 = scope.create_local(u32::as_elem());
        let kernel_size_1 = scope.create_local(u32::as_elem());
        cpa!(scope, weight_stride_0 = stride(weight, 0u32));
        cpa!(scope, weight_stride_1 = stride(weight, 1u32));
        cpa!(scope, weight_stride_2 = stride(weight, 2u32));
        cpa!(scope, weight_stride_3 = stride(weight, 3u32));
        cpa!(scope, in_channels = shape(weight, 0u32));
        cpa!(scope, weight_shape_1 = shape(weight, 1u32));
        cpa!(scope, kernel_size_0 = shape(weight, 2u32));
        cpa!(scope, kernel_size_1 = shape(weight, 3u32));

        let conv_stride_0 = Variable::new(VariableKind::GlobalScalar(0), Item::new(u32::as_elem()));
        let conv_stride_1 = Variable::new(VariableKind::GlobalScalar(1), Item::new(u32::as_elem()));
        let dilation_0 = Variable::new(VariableKind::GlobalScalar(2), Item::new(u32::as_elem()));
        let dilation_1 = Variable::new(VariableKind::GlobalScalar(3), Item::new(u32::as_elem()));
        let padding_0 = Variable::new(VariableKind::GlobalScalar(4), Item::new(u32::as_elem()));
        let padding_1 = Variable::new(VariableKind::GlobalScalar(5), Item::new(u32::as_elem()));
        let groups = Variable::new(VariableKind::GlobalScalar(6), Item::new(u32::as_elem()));

        let stride_0_i = scope.create_local(Elem::Int(IntKind::I32));
        let stride_1_i = scope.create_local(Elem::Int(IntKind::I32));
        cpa!(scope, stride_0_i = cast(conv_stride_0));
        cpa!(scope, stride_1_i = cast(conv_stride_1));

        let oc_out = scope.create_local(u32::as_elem());
        let oc = scope.create_local(u32::as_elem());

        let b = scope.create_local(u32::as_elem());
        let oh = scope.create_local(u32::as_elem());
        let ow = scope.create_local(u32::as_elem());
        let k = scope.create_local(u32::as_elem());
        let g = scope.create_local(u32::as_elem());

        let ic_start = scope.create_local(u32::as_elem());
        let ic_end = scope.create_local(u32::as_elem());
        let ic_tmp = scope.create_local(u32::as_elem());

        cpa!(scope, b = id / output_stride_0);
        cpa!(scope, b = b % output_shape_0);

        cpa!(scope, oc_out = id / output_stride_1);
        cpa!(scope, oc_out = oc_out % output_shape_1);

        cpa!(scope, oh = id / output_stride_2);
        cpa!(scope, oh = oh % output_shape_2);

        cpa!(scope, ow = id / output_stride_3);
        cpa!(scope, ow = ow % output_shape_3);

        cpa!(scope, k = oc_out / weight_shape_1);
        cpa!(scope, g = k % groups);
        cpa!(scope, oc = weight_shape_1 * g);
        cpa!(scope, oc = oc_out - oc);

        cpa!(scope, ic_tmp = in_channels / groups);
        cpa!(scope, ic_start = g * ic_tmp);
        cpa!(scope, ic_end = ic_start + ic_tmp);

        let tmp_u = scope.create_local(u32::as_elem());
        let tmp_i = scope.create_local(Elem::Int(IntKind::I32));
        let zero_i = scope.zero(Elem::Int(IntKind::I32));
        let one_i = scope.create_with_value(1, Elem::Int(IntKind::I32));

        let kms_u = scope.create_local(u32::as_elem());
        let kms_0 = scope.create_local(Elem::Int(IntKind::I32));
        let kms_1 = scope.create_local(Elem::Int(IntKind::I32));
        let ih_start_tmp = scope.create_local(Elem::Int(IntKind::I32));
        let iw_start_tmp = scope.create_local(Elem::Int(IntKind::I32));
        let ih_start = scope.create_local(u32::as_elem());
        let iw_start = scope.create_local(u32::as_elem());
        let ih_end = scope.create_local(u32::as_elem());
        let iw_end = scope.create_local(u32::as_elem());

        cpa!(scope, kms_u = kernel_size_0 * dilation_0);
        cpa!(scope, kms_0 = cast(kms_u));
        cpa!(scope, kms_0 = kms_0 - stride_0_i);
        cpa!(scope, kms_u = kernel_size_1 * dilation_1);
        cpa!(scope, kms_1 = cast(kms_u));
        cpa!(scope, kms_1 = kms_1 - stride_1_i);

        cpa!(scope, tmp_u = oh + padding_0);
        cpa!(scope, tmp_i = cast(tmp_u));
        cpa!(scope, ih_start_tmp = tmp_i - kms_0);
        cpa!(scope, ih_start_tmp = ih_start_tmp / stride_0_i);
        cpa!(scope, tmp_u = ow + padding_1);
        cpa!(scope, tmp_i = cast(tmp_u));
        cpa!(scope, iw_start_tmp = tmp_i - kms_1);
        cpa!(scope, iw_start_tmp = iw_start_tmp / stride_1_i);

        cpa!(scope, tmp_i = max(ih_start_tmp, zero_i));
        cpa!(scope, ih_start = cast(tmp_i));
        cpa!(scope, tmp_i = kms_0 + ih_start_tmp);
        cpa!(scope, tmp_i += one_i);
        cpa!(scope, tmp_i = max(tmp_i, zero_i));
        cpa!(scope, tmp_u = cast(tmp_i));
        cpa!(scope, ih_end = min(tmp_u, input_shape_2));

        cpa!(scope, tmp_i = max(iw_start_tmp, zero_i));
        cpa!(scope, iw_start = cast(tmp_i));
        cpa!(scope, tmp_i = kms_1 + iw_start_tmp);
        cpa!(scope, tmp_i += one_i);
        cpa!(scope, tmp_i = max(tmp_i, zero_i));
        cpa!(scope, tmp_u = cast(tmp_i));
        cpa!(scope, iw_end = min(tmp_u, input_shape_3));

        let index_input = scope.create_local(u32::as_elem());
        let index_weight = scope.create_local(u32::as_elem());

        let index_input_b = scope.create_local(u32::as_elem());
        let index_input_ic = scope.create_local(u32::as_elem());
        let index_input_ih = scope.create_local(u32::as_elem());
        let index_input_iw = scope.create_local(u32::as_elem());
        let index_weight_ic = scope.create_local(u32::as_elem());
        let index_weight_oc = scope.create_local(u32::as_elem());
        let index_weight_kh = scope.create_local(u32::as_elem());
        let index_weight_kw = scope.create_local(u32::as_elem());

        cpa!(scope, index_input_b = b * input_stride_0);
        cpa!(scope, index_weight_oc = oc * weight_stride_1);

        let prod = scope.create_local(output.item);
        let prod_tmp = scope.create_local(output.item);
        let sum = scope.create_local(output.item);
        cpa!(scope, sum = bias[oc_out]);

        let kh = scope.create_local(u32::as_elem());
        let kw = scope.create_local(u32::as_elem());
        let numerator_h_base = scope.create_local(u32::as_elem());
        let numerator_h = scope.create_local(u32::as_elem());
        let numerator_w_base = scope.create_local(u32::as_elem());
        let numerator_w = scope.create_local(u32::as_elem());
        let numerator_tmp = scope.create_local(u32::as_elem());
        let numerator_mod = scope.create_local(u32::as_elem());
        let zero = scope.zero(u32::as_elem());
        let divisible = scope.create_local(Elem::Bool);
        let not_neg = scope.create_local(Elem::Bool);
        let cond = scope.create_local(Elem::Bool);

        cpa!(scope, numerator_h_base = oh + padding_0);
        cpa!(scope, numerator_w_base = ow + padding_1);

        cpa!(
            scope,
            range(ic_start, ic_end).for_each(|ic, scope| {
                cpa!(scope, index_input_ic = ic * input_stride_1);
                cpa!(scope, index_weight_ic = ic * weight_stride_0);

                cpa!(
                    scope,
                    range(ih_start, ih_end).for_each(|ih, scope| {
                        cpa!(scope, numerator_tmp = ih * conv_stride_0);
                        cpa!(scope, not_neg = numerator_h_base >= numerator_tmp);
                        cpa!(scope, numerator_h = numerator_h_base - numerator_tmp);

                        cpa!(scope, numerator_mod = numerator_h % dilation_0);
                        cpa!(scope, divisible = numerator_mod == zero);
                        cpa!(scope, cond = not_neg && divisible);

                        cpa!(scope, if(cond).then(|scope|{
                            cpa!(scope, kh = numerator_h / dilation_0);
                            cpa!(scope, index_input_ih = ih * input_stride_2);
                            cpa!(scope, index_weight_kh = kh * weight_stride_2);

                            cpa!(
                                scope,
                                range(iw_start, iw_end).for_each(|iw, scope| {
                                    cpa!(scope, numerator_tmp = iw * conv_stride_1);
                                    cpa!(scope, not_neg = numerator_w_base >= numerator_tmp);
                                    cpa!(scope, numerator_w = numerator_w_base - numerator_tmp);

                                    cpa!(scope, numerator_mod = numerator_w % dilation_1);
                                    cpa!(scope, divisible = numerator_mod == zero);
                                    cpa!(scope, cond = not_neg && divisible);

                                    cpa!(scope, if(cond).then(|scope|{
                                        cpa!(scope, kw = numerator_w / dilation_1);
                                        cpa!(scope, index_input_iw = iw * input_stride_3);
                                        cpa!(scope, index_weight_kw = kw * weight_stride_3);

                                        cpa!(scope, index_input = index_input_b);
                                        cpa!(scope, index_input += index_input_ic);
                                        cpa!(scope, index_input += index_input_ih);
                                        cpa!(scope, index_input += index_input_iw);

                                        cpa!(scope, index_weight = index_weight_ic);
                                        cpa!(scope, index_weight += index_weight_oc);
                                        cpa!(scope, index_weight += index_weight_kh);
                                        cpa!(scope, index_weight += index_weight_kw);

                                        cpa!(scope, prod = input[index_input]);
                                        cpa!(scope, prod_tmp = weight[index_weight]);
                                        cpa!(scope, prod *= prod_tmp);
                                        cpa!(scope, sum += prod);
                                    }));
                                })
                            );

                        }));
                    })
                );
            })
        );

        cpa!(scope, output[id] = sum);
    }
}

impl<R: JitRuntime, E: JitElement> Kernel for Conv2dTransposeEagerKernel<R, E> {
    fn define(&self) -> KernelDefinition {
        let mut scope = Scope::root();
        let item = E::cube_elem().into();

        let input = Variable::new(VariableKind::GlobalInputArray(0), item);
        let weight = Variable::new(VariableKind::GlobalInputArray(1), item);
        let bias = Variable::new(VariableKind::GlobalInputArray(2), item);
        let output = Variable::new(VariableKind::GlobalOutputArray(0), item);

        scope.write_global_custom(output);

        Conv2dTransposeComputeShader {
            input,
            weight,
            bias,
            output,
            _elem: PhantomData::<E>,
        }
        .expand(&mut scope);

        let input = InputInfo::Array {
            item,
            visibility: Visibility::Read,
        };
        let weight = InputInfo::Array {
            item,
            visibility: Visibility::Read,
        };
        let bias = InputInfo::Array {
            item,
            visibility: Visibility::Read,
        };
        let scalars = InputInfo::Scalar {
            elem: u32::as_elem(),
            size: 7,
        };

        let output = OutputInfo::Array { item };

        let info = KernelExpansion {
            inputs: vec![input, weight, bias, scalars],
            outputs: vec![output],
            scope,
        };

        let settings = KernelSettings::default();
        KernelIntegrator::new(info).integrate(settings)
    }

    fn id(&self) -> cubecl::KernelId {
        cubecl::KernelId::new::<Self>()
    }
}

/// Perform a 2D convolution transposition using the direct algorithm.
///
/// * `input` - The input feature map
/// * `weight` - The weights (filter) applied to each kernel
/// * `bias` - The bias added to each channel
/// * `options` - The options to use for the convolution
///
#[allow(clippy::extra_unused_type_parameters)]
pub fn conv_transpose2d_direct<R: JitRuntime, E: JitElement, I: IntElement>(
    input: JitTensor<R, E>,
    weight: JitTensor<R, E>,
    bias: Option<JitTensor<R, E>>,
    options: ConvTransposeOptions<2>,
) -> JitTensor<R, E> {
    let input = kernel::into_contiguous(input);
    let weight = kernel::into_contiguous(weight);
    let [batch_size, _, in_height, in_width] = input.shape.dims();
    let [_, out_channels, kernel_0, kernel_1] = weight.shape.dims();

    let out_0 = (in_height - 1) * options.stride[0]
        + options.dilation[0] * (kernel_0 - 1)
        + options.padding_out[0]
        - 2 * options.padding[0]
        + 1;
    let out_1 = (in_width - 1) * options.stride[1]
        + options.dilation[1] * (kernel_1 - 1)
        + options.padding_out[1]
        - 2 * options.padding[1]
        + 1;

    let shape_out = Shape::new([batch_size, out_channels * options.groups, out_0, out_1]);

    let output = empty_device(
        input.client.clone(),
        input.device.clone(),
        shape_out.clone(),
    );

    let bias = match bias {
        Some(bias) => {
            let shape = Shape::from([bias.shape.dims[0], 1, 1, 1]);
            reshape(bias, shape)
        }
        None => {
            let shape = Shape::from([output.shape.dims[0], 1, 1, 1]);
            zeros_device(input.client.clone(), input.device.clone(), shape)
        }
    };

    let kernel = Conv2dTransposeEagerKernel::<R, E>::new();

    Execution::start(kernel, input.client.clone())
        .inputs(&[
            input.as_handle_ref(),
            weight.as_handle_ref(),
            bias.as_handle_ref(),
        ])
        .outputs(&[output.as_handle_ref()])
        .with_scalars(&[
            options.stride[0] as u32,
            options.stride[1] as u32,
            options.dilation[0] as u32,
            options.dilation[1] as u32,
            options.padding[0] as u32,
            options.padding[1] as u32,
            options.groups as u32,
        ])
        .execute(CubeCountSettings::Output { pos: 0 });

    output
}
