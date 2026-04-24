fn main() {
    cynic_codegen::register_schema("linear")
        .from_sdl_file("schema.graphql")
        .unwrap();
}
