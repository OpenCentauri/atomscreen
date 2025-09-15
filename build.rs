fn main() {
    slint_build::compile_with_config("ui/main.slint",
slint_build::CompilerConfiguration::new()
            .with_style("material-dark".into())
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer))
            .expect("Slint build failed");
}
