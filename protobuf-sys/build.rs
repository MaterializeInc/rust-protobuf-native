// Copyright Materialize, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository, or online at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let include_paths = [
        PathBuf::from(env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()).join("include"),
        PathBuf::from("src"),
    ];
    autocxx_build::Builder::new("src/lib.rs", &include_paths)
        .build()?
        .flag_if_supported("-std=c++14")
        .compile("protobuf-sys");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!(
        "cargo:rustc-link-search=native={}/lib",
        env::var("DEP_PROTOBUF_SRC_ROOT").unwrap()
    );

    for lib in [
        "absl_bad_any_cast_impl",
        "absl_bad_optional_access",
        "absl_bad_variant_access",
        "absl_base",
        "absl_city",
        "absl_civil_time",
        "absl_cord",
        "absl_cordz_functions",
        "absl_cordz_handle",
        "absl_cordz_info",
        "absl_cordz_sample_token",
        "absl_cord_internal",
        "absl_crc32c",
        "absl_crc_cord_state",
        "absl_crc_cpu_detect",
        "absl_crc_internal",
        "absl_debugging_internal",
        "absl_demangle_internal",
        "absl_die_if_null",
        "absl_examine_stack",
        "absl_exponential_biased",
        "absl_failure_signal_handler",
        "absl_flags_commandlineflag",
        "absl_flags_commandlineflag_internal",
        "absl_flags_config",
        "absl_flags_internal",
        "absl_flags_marshalling",
        "absl_flags_parse",
        "absl_flags_private_handle_accessor",
        "absl_flags_program_name",
        "absl_flags_reflection",
        "absl_flags_usage",
        "absl_flags_usage_internal",
        "absl_graphcycles_internal",
        "absl_hash",
        "absl_hashtablez_sampler",
        "absl_int128",
        "absl_kernel_timeout_internal",
        "absl_leak_check",
        "absl_log_entry",
        "absl_log_flags",
        "absl_log_globals",
        "absl_log_initialize",
        "absl_log_internal_check_op",
        "absl_log_internal_conditions",
        "absl_log_internal_fnmatch",
        "absl_log_internal_format",
        "absl_log_internal_globals",
        "absl_log_internal_log_sink_set",
        "absl_log_internal_message",
        "absl_log_internal_nullguard",
        "absl_log_internal_proto",
        "absl_log_severity",
        "absl_log_sink",
        "absl_low_level_hash",
        "absl_malloc_internal",
        "absl_periodic_sampler",
        "absl_random_distributions",
        "absl_random_internal_distribution_test_util",
        "absl_random_internal_platform",
        "absl_random_internal_pool_urbg",
        "absl_random_internal_randen",
        "absl_random_internal_randen_hwaes",
        "absl_random_internal_randen_hwaes_impl",
        "absl_random_internal_randen_slow",
        "absl_random_internal_seed_material",
        "absl_random_seed_gen_exception",
        "absl_random_seed_sequences",
        "absl_raw_hash_set",
        "absl_raw_logging_internal",
        "absl_scoped_set_env",
        "absl_spinlock_wait",
        "absl_stacktrace",
        "absl_status",
        "absl_statusor",
        "absl_strerror",
        "absl_strings",
        "absl_strings_internal",
        "absl_string_view",
        "absl_str_format_internal",
        "absl_symbolize",
        "absl_synchronization",
        "absl_throw_delegate",
        "absl_time",
        "absl_time_zone",
        "absl_vlog_config_internal",
        "protobuf",
        "utf8_validity",
    ] {
        println!("cargo:rustc-link-lib=static={lib}");
    }

    Ok(())
}
