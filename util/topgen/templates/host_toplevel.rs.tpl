// Copyright lowRISC contributors.
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

// This file was generated automatically.
// Please do not modify content of this file directly.
// File generated by using template: "host_toplevel.rs.tpl"
// To regenerate execute command "make -C hw top" from
// project top level directory.

#![allow(dead_code)]

use crate::with_unknown;

with_unknown! {
${helper.pinmux_peripheral_in.render_host()}

${helper.pinmux_insel.render_host()}

${helper.pinmux_mio_out.render_host()}

${helper.pinmux_outsel.render_host()}

${helper.direct_pads.render_host()}

${helper.muxed_pads.render_host()}
}

#[allow(non_camel_case_types)]
pub mod ujson_alias {
    use super::*;
    // Create aliases for the C names of these types so that the ujson
    // created structs can access these structures by their C names.
    pub type pinmux_peripheral_in_t = ${helper.pinmux_peripheral_in.short_name.as_rust_type()};
    pub type pinmux_insel_t = ${helper.pinmux_insel.short_name.as_rust_type()};
    pub type pinmux_mio_out_t = ${helper.pinmux_mio_out.short_name.as_rust_type()};
    pub type pinmux_outsel_t = ${helper.pinmux_outsel.short_name.as_rust_type()};
}
