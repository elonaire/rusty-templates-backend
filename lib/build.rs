fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("../services/acl-service/src/grpc/proto/acl.proto")?;
    let acl_proto_path = get_canonical_path_external("acl")?;
    let email_proto_path = get_canonical_path_external("email")?;
    let files_proto_path = get_canonical_path_external("files")?;

    tonic_build::compile_protos(acl_proto_path)?;
    tonic_build::compile_protos(email_proto_path)?;
    tonic_build::compile_protos(files_proto_path)?;
    Ok(())
}

/// Get the canonical path of a proto file. The protofile_name is the name of the proto file without the extension.
fn get_canonical_path_external(protofile_name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let raw_path = format!("./src/integration/grpc/proto/{}.proto", protofile_name);

    std::path::Path::new(&raw_path).canonicalize()
}
