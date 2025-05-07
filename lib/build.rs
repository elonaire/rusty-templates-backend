fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("proto/acl/acl.proto")?;
    // tonic_build::compile_protos("proto/email/email.proto")?;
    // tonic_build::compile_protos("proto/files/files.proto")?;
    // tonic_build::compile_protos("proto/products/products.proto")?;
    // tonic_build::compile_protos("proto/orders/orders.proto")?;
    // tonic_build::compile_protos("proto/payments/payments.proto")?;
    let out_dir = "src/integration/grpc/out";

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(out_dir)
        .compile_protos(
            &[
                "proto/acl/acl.proto",
                "proto/email/email.proto",
                "proto/files/files.proto",
                "proto/products/products.proto",
                "proto/orders/orders.proto",
                "proto/payments/payments.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
