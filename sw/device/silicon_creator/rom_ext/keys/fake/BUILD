# Copyright lowRISC contributors.
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0

package(default_visibility = ["//visibility:public"])

cc_library(
    name = "fake",
    srcs = [
        "sigverify_rsa_keys_fake.c",
        "//sw/device/silicon_creator/rom_ext:sigverify_keys.h",
    ],
    hdrs = [
        "rom_ext_dev_key_0_rsa_3072_exp_f4.h",
        "rom_ext_test_key_0_rsa_3072_exp_f4.h",
    ],
    deps = [
        "//hw/ip/otp_ctrl/data:otp_ctrl_regs",
        "//sw/device/lib/base:macros",
        "//sw/device/silicon_creator/lib/sigverify",
    ],
)

cc_test(
    name = "fake_unittest",
    srcs = ["sigverify_rsa_keys_fake_unittest.cc"],
    deps = [
        ":fake",
        "//hw/top_earlgrey/sw/autogen:top_earlgrey",
        "//sw/device/silicon_creator/rom_ext:sigverify_keys",
        "//sw/device/silicon_creator/testing:rom_test",
        "@googletest//:gtest_main",
    ],
)

filegroup(
    name = "rom_ext_dev_private_key_0",
    srcs = ["rom_ext_dev_key_0_rsa_3072_exp_f4.der"],
)

filegroup(
    name = "rom_ext_test_private_key_0",
    srcs = ["rom_ext_test_key_0_rsa_3072_exp_f4.der"],
)
