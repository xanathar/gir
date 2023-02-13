use std::io::{Result, Write};

use log::info;

use super::collect_versions;
use crate::{codegen::general, env::Env, file_saver::save_to_file};

pub fn generate(env: &Env) {
    info!(
        "Generating sys build script for {}",
        env.config.library_name
    );

    let split_build_rs = env.config.split_build_rs;
    let path = env.config.target_path.join("build.rs");

    if !split_build_rs || !path.exists() {
        info!("Generating file {:?}", path);
        save_to_file(&path, env.config.make_backup, |w| {
            generate_build_script(w, env, split_build_rs)
        });
    }

    if split_build_rs {
        let path = env.config.target_path.join("build_version.rs");
        info!("Generating file {:?}", path);
        save_to_file(&path, env.config.make_backup, |w| {
            generate_build_version(w, env)
        });
    }
}

#[allow(clippy::write_literal)]
fn generate_build_script(w: &mut dyn Write, env: &Env, split_build_rs: bool) -> Result<()> {
    let (script_cfg_attr, script_cfg_attr_not) = if env.config.optional_link_attribute {
        (r#"#[cfg(any(feature = "dox", feature = "omit_link_attribute"))]"#, r#"#[cfg(not(any(feature = "dox", feature = "omit_link_attribute")))]"#)
    } else {
        (r#"#[cfg(feature = "dox")]"#, r#"#[cfg(not(feature = "dox"))]"#)
    };

    if !split_build_rs {
        general::start_comments(w, &env.config)?;
        writeln!(w)?;
    }

    writeln!(w, "{}", script_cfg_attr_not)?;
    writeln!(w, "{}", "use std::process;")?;

    if split_build_rs {
        writeln!(w)?;
        writeln!(w, "mod build_version;")?;
    }

    writeln!(w, "{}", script_cfg_attr)?;
    writeln!(w, "{}", "fn main() {} // prevent linking libraries to avoid documentation or self-link failure")?;
    writeln!(w)?;
    writeln!(w, "{}", script_cfg_attr_not)?;
    writeln!(w, "{}", r##"fn main() {
        if let Err(s) = system_deps::Config::new().probe() {
            println!("cargo:warning={s}");
            process::exit(1);
        }
    }"##)?;
    writeln!(w)
}

fn generate_build_version(w: &mut dyn Write, env: &Env) -> Result<()> {
    general::start_comments(w, &env.config)?;
    writeln!(w)?;
    writeln!(w, "pub fn version() -> &'static str {{")?;
    write_version(w, env, false)?;
    writeln!(w, "}}")
}

fn write_version(w: &mut dyn Write, env: &Env, for_let: bool) -> Result<()> {
    let versions = collect_versions(env);

    for (version, lib_version) in versions.iter().rev() {
        write!(
            w,
            "if cfg!({}) {{\n\t\t\"{}\"\n\t}} else ",
            version.to_cfg(None),
            lib_version
        )?;
    }
    let end = if for_let { ";" } else { "" };
    writeln!(w, "{{\n\t\t\"{}\"\n\t}}{}", env.config.min_cfg_version, end)
}
