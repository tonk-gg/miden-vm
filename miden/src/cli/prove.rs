use super::data::{InputFile, OutputFile, ProgramFile, ProofFile};
use miden::ProofOptions;
use std::{io::Write, path::PathBuf, time::Instant};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Prove", about = "Prove a miden program")]
pub struct ProveCmd {
    /// Path to .masm assembly file
    #[structopt(short = "a", long = "assembly", parse(from_os_str))]
    assembly_file: PathBuf,
    /// Path to input file
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    input_file: Option<PathBuf>,
    /// Number of ouptuts
    #[structopt(short = "n", long = "num-outputs", default_value = "16")]
    num_outputs: usize,
    /// Path to output file
    #[structopt(short = "o", long = "output", parse(from_os_str))]
    output_file: Option<PathBuf>,
    /// Path to proof file
    #[structopt(short = "p", long = "proof", parse(from_os_str))]
    proof_file: Option<PathBuf>,
    /// Security level for execution proofs generated by the VM
    #[structopt(short = "s", long = "security", default_value = "96bits")]
    security: String,
}

impl ProveCmd {
    pub fn get_proof_security(&self) -> ProofOptions {
        match self.security.as_str() {
            "96bits" => ProofOptions::with_96_bit_security(),
            "128bits" => ProofOptions::with_128_bit_security(),
            other => panic!("{} is not a valid security setting", other),
        }
    }

    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");
        println!("Prove program");
        println!("============================================================");

        // configure logging
        env_logger::Builder::new()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter_level(log::LevelFilter::Debug)
            .init();

        // load program from file and compile
        let program = ProgramFile::read(&self.assembly_file)?;

        // load input data from file
        let input_data = InputFile::read(&self.input_file, &self.assembly_file)?;

        let program_hash: [u8; 32] = program.hash().into();
        println!("Proving program with hash {}...", hex::encode(program_hash));
        let now = Instant::now();

        // fetch the stack and program inputs from the arguments
        let stack_inputs = input_data.parse_stack_inputs()?;
        let advice_provider = input_data.parse_advice_provider()?;

        // execute program and generate proof
        let (stack_outputs, proof) =
            prover::prove(&program, stack_inputs, advice_provider, &self.get_proof_security())
                .map_err(|err| format!("Failed to prove program - {:?}", err))?;

        println!(
            "Program with hash {} proved in {} ms",
            hex::encode(program_hash),
            now.elapsed().as_millis()
        );

        // write proof to file
        ProofFile::write(proof, &self.proof_file, &self.assembly_file)?;

        // provide outputs
        if let Some(output_path) = &self.output_file {
            // write all outputs to specified file.
            OutputFile::write(&stack_outputs, output_path)?;
        } else {
            // if no output path was provided, get the stack outputs for printing to the screen.
            let stack = stack_outputs.stack_truncated(self.num_outputs).to_vec();

            // write all outputs to default location if none was provided
            OutputFile::write(&stack_outputs, &self.assembly_file.with_extension("outputs"))?;

            // print stack outputs to screen.
            println!("Output: {:?}", stack);
        }

        Ok(())
    }
}
