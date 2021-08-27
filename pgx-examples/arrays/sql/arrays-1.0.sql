/* 
This file is auto generated by pgx.

The ordering of items is not stable, it is driven by a dependency graph.
*/

-- src/lib.rs:81
-- arrays::i32_array_with_nulls
CREATE OR REPLACE FUNCTION arrays."i32_array_with_nulls"() RETURNS integer[] /* alloc::vec::Vec<core::option::Option<i32>> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'i32_array_with_nulls_wrapper';

-- src/lib.rs:76
-- arrays::i32_array_no_nulls
CREATE OR REPLACE FUNCTION arrays."i32_array_no_nulls"() RETURNS integer[] /* alloc::vec::Vec<i32> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'i32_array_no_nulls_wrapper';

-- src/lib.rs:9
-- arrays::sq_euclid_pgx
CREATE OR REPLACE FUNCTION arrays."sq_euclid_pgx"(
	"a" real[], /* pgx::datum::array::Array<f32> */
	"b" real[] /* pgx::datum::array::Array<f32> */
) RETURNS real /* f32 */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'sq_euclid_pgx_wrapper';

-- src/lib.rs:61
-- arrays::static_names
CREATE OR REPLACE FUNCTION arrays."static_names"() RETURNS text[] /* alloc::vec::Vec<core::option::Option<&str>> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'static_names_wrapper';

-- src/lib.rs:86
-- arrays::strip_nulls
CREATE OR REPLACE FUNCTION arrays."strip_nulls"(
	"input" integer[] /* alloc::vec::Vec<core::option::Option<i32>> */
) RETURNS integer[] /* alloc::vec::Vec<i32> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'strip_nulls_wrapper';

-- src/lib.rs:66
-- arrays::static_names_set
CREATE OR REPLACE FUNCTION arrays."static_names_set"() RETURNS SETOF text[] /* alloc::vec::Vec<core::option::Option<&str>> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'static_names_set_wrapper';

-- src/lib.rs:32
-- arrays::default_array
CREATE OR REPLACE FUNCTION arrays."default_array"() RETURNS integer[] /* alloc::vec::Vec<i32> */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'default_array_wrapper';

-- src/lib.rs:18
-- arrays::approx_distance_pgx
CREATE OR REPLACE FUNCTION arrays."approx_distance_pgx"(
	"compressed" bigint[], /* pgx::datum::array::Array<i64> */
	"distances" double precision[] /* pgx::datum::array::Array<f64> */
) RETURNS double precision /* f64 */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'approx_distance_pgx_wrapper';

-- src/lib.rs:48
-- arrays::sum_vec
CREATE OR REPLACE FUNCTION arrays."sum_vec"(
	"input" integer[] /* alloc::vec::Vec<core::option::Option<i32>> */
) RETURNS bigint /* i64 */
STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'sum_vec_wrapper';

-- src/lib.rs:37
-- arrays::sum_array
-- requires:
--   default_array
CREATE OR REPLACE FUNCTION arrays."sum_array"(
	"input" integer[] DEFAULT default_array() /* pgx::datum::array::Array<i32> */
) RETURNS bigint /* i64 */
 STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'sum_array_wrapper';

-- src/lib.rs:95
-- arrays::SomeStruct
CREATE TYPE arrays.SomeStruct;

-- src/lib.rs:95
-- arrays::somestruct_in
CREATE OR REPLACE FUNCTION arrays."somestruct_in"(
	"input" cstring /* &std::ffi::c_str::CStr */
) RETURNS arrays.SomeStruct /* arrays::SomeStruct */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'somestruct_in_wrapper';

-- src/lib.rs:95
-- arrays::somestruct_out
CREATE OR REPLACE FUNCTION arrays."somestruct_out"(
	"input" arrays.SomeStruct /* arrays::SomeStruct */
) RETURNS cstring /* &std::ffi::c_str::CStr */
IMMUTABLE PARALLEL SAFE STRICT
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'somestruct_out_wrapper';

-- src/lib.rs:95
-- arrays::SomeStruct
CREATE TYPE arrays.SomeStruct (
	INTERNALLENGTH = variable,
	INPUT = arrays.somestruct_in, /* arrays::somestruct_in */
	OUTPUT = arrays.somestruct_out, /* arrays::somestruct_out */
	STORAGE = extended
);

-- src/lib.rs:98
-- arrays::return_vec_of_customtype
CREATE OR REPLACE FUNCTION arrays."return_vec_of_customtype"() RETURNS arrays.SomeStruct[] /* alloc::vec::Vec<arrays::SomeStruct> */
STRICT
SET search_path TO @extschema@
LANGUAGE c /* Rust */
AS 'MODULE_PATHNAME', 'return_vec_of_customtype_wrapper';
