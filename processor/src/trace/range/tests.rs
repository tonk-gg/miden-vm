use super::{
    super::{Digest, ExecutionTrace, Process, NUM_RAND_ROWS},
    Felt, FieldElement, P0_COL_IDX, P1_COL_IDX,
};
use rand_utils::rand_value;
use vm_core::{Operation, ProgramInputs};
use winterfell::Trace;

// TODO: when all u32 operations are updated to use 4 range checks, this test will need to be
// updated.
#[test]
fn p0_trace() {
    // --- Range check 256_u32 (2 16-bit range checks: 0 and 256) -----------------------------------
    let stack = [1, 255];
    let operations = vec![Operation::U32add];

    let inputs = ProgramInputs::new(&stack, &[], vec![]).unwrap();
    let mut process = Process::new(inputs);

    for operation in operations.iter() {
        process.execute_op(*operation).unwrap();
    }

    let mut trace = ExecutionTrace::new(process, Digest::new([Felt::ZERO; 4]));
    let alpha = rand_value::<Felt>();
    let rand_elements = vec![alpha];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let p0 = aux_columns.get_column(P0_COL_IDX);

    assert_eq!(trace.length(), p0.len());

    // 256 8-bit rows are needed to for each value 0-255. 64 8-bit rows are needed to check 256
    // increments of 255 in the 16-bit portion of the table, for a total of 256 + 63 = 319 rows.
    let len_8bit = 319;
    // 259 16-bit rows are needed for 0, 255, 256, ... 255 increments of 255 ..., 65535. (0 and 256
    // are range-checked, 65535 is the max, and the rest are "bridge" values.)
    let len_16bit = 259;
    // The range checker is padded at the beginning, so the padding must be skipped.
    let start_8bit = trace.length() - len_8bit - len_16bit - NUM_RAND_ROWS;
    let start_16bit = trace.length() - len_16bit - NUM_RAND_ROWS;

    // The padded portion of the column should be all ones.
    let expected_padding = vec![Felt::ONE; start_8bit];
    assert_eq!(expected_padding, p0[..start_8bit]);

    // The first value in the 8-bit portion should be one.
    assert_eq!(Felt::ONE, p0[start_8bit]);

    // At the start of the 16-bit portion, the value of `p0` should include all the 8-bit lookups:
    // 1 lookup of one; 1 lookup of 254; 256 lookups of 255.
    // Therefore, the value should be: (alpha + 1) * (alpha + 254) + (alpha + 255)^256
    let mut acc_255 = alpha + Felt::new(255);
    for _ in 0..8 {
        acc_255 *= acc_255;
    }
    let expected_acc = (alpha + Felt::ONE) * (alpha + Felt::new(254)) * acc_255;
    assert_eq!(expected_acc, p0[start_16bit]);

    // The final value at the end of the 16-bit portion should be 1. This will be the last row
    // before the random row.
    assert_eq!(Felt::ONE, p0[p0.len() - 1 - NUM_RAND_ROWS]);
}

#[test]
fn p1_trace() {
    // --- Range check 256_u32 (2 16-bit range checks: 0 and 1) -----------------------------------
    let stack = [1, 255];
    let operations = vec![Operation::U32add];

    let inputs = ProgramInputs::new(&stack, &[], vec![]).unwrap();
    let mut process = Process::new(inputs);

    for operation in operations.iter() {
        process.execute_op(*operation).unwrap();
    }

    let mut trace = ExecutionTrace::new(process, Digest::new([Felt::ZERO; 4]));
    let alpha = rand_value::<Felt>();
    let rand_elements = vec![alpha];
    let aux_columns = trace.build_aux_segment(&[], &rand_elements).unwrap();
    let p1 = aux_columns.get_column(P1_COL_IDX);

    assert_eq!(trace.length(), p1.len());

    // 256 8-bit rows are needed to for each value 0-255. 64 8-bit rows are needed to check 256
    // increments of 255 in the 16-bit portion of the table, for a total of 256 + 63 = 319 rows.
    let len_8bit = 319;
    // 259 16-bit rows are needed for 0, 255, 256, ... 255 increments of 255 ..., 65535. (0 and 256
    // are range-checked, 65535 is the max, and the rest are "bridge" values.
    let len_16bit = 259;
    // The range checker is padded at the beginning, so the padding must be skipped.
    let padding_end = trace.length() - len_8bit - len_16bit - NUM_RAND_ROWS;
    let start_16bit = trace.length() - len_16bit - NUM_RAND_ROWS;

    // The values in p1 should be one for the length of the 8-bit table.
    let expected_8bit = vec![Felt::ONE; len_8bit];
    assert_eq!(expected_8bit, p1[padding_end..start_16bit]);

    // Once the 16-bit portion of the table starts, the first value will be one.
    assert_eq!(p1[start_16bit], Felt::ONE);
    // We include 1 lookup of 0, so the next value should be multiplied by alpha.
    let mut expected = alpha;
    assert_eq!(expected, p1[start_16bit + 1]);
    // Then we have a bridge row for 255 where the value does not change
    assert_eq!(expected, p1[start_16bit + 2]);
    // Then we include 1 lookup of 256, so it should be multiplied by alpha + 256.
    expected *= alpha + Felt::new(256);
    assert_eq!(expected, p1[start_16bit + 3]);
}
