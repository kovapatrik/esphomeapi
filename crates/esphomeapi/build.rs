use std::io::Result;

fn main() -> Result<()> {
  protobuf_codegen::Codegen::new()
    .protoc()
    .includes(["src/protos"])
    .input("src/protos/api_options.proto")
    .input("src/protos/api.proto")
    .cargo_out_dir("protos")
    .run_from_script();

  Ok(())
}
