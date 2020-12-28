#![feature(type_ascription)]

#[macro_use]extern crate serde_derive;

use std::{
    path::{Path, MAIN_SEPARATOR},
    result::Result,
    str::from_utf8,
    error::Error,
    fs::{File, create_dir},
    io::prelude::*,
};

use log::{info, debug, trace};
use simple_logger::SimpleLogger;
use cargo_toml::{Manifest, Dependency, DependencyDetail};
use std::process::{Command, Stdio};
use tinytemplate::TinyTemplate;
use clap::{App, Arg};
use regex::Regex;

static PYLIB_RS_IN: &'static str = include_str!("pylib.rs.in");
static SETUP_PY_IN: &'static str = include_str!("setup.py.in");
static MANIFEST_IN: &'static str = include_str!("MANIFEST.in");
static INIT_PY: &'static str = include_str!("__init__.py.in");

#[derive(Serialize)]
struct Context {
    module_name: String,
    rs_lib_mod: String,
    package_name: String,
}


fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().init().unwrap();
    //let parser = ArgParser::new("cargo-pyinit".into());
    let matches = App::new("cargo-pyinit")
        .version("0.0.1")
        .about("cargo extension to create Python extensions.")
        .arg(
            Arg::with_name("cargo-path")
                .index(1)
                //.about("The path to initialize.  If not specified, '.' will be inferred.")
                .required(false)
                .default_value(".")
        )
        .arg(
            Arg::with_name("path")
                .index(2)
                //.about("The path to initialize.  If not specified, '.' will be inferred.")
                .required(false)
                .default_value(".")
        )
        .get_matches();

    // Running under cargo messes with the parameter order, so we have to handle it differently.s
    let mut path = matches.value_of("cargo-path").unwrap();
    if path == "pyinit" {
        path = matches.value_of("path").unwrap();
    }
    trace!("Manifest file contents: {}", path);

    let cargo_toml_path_str = format!("{}{}Cargo.toml", path, MAIN_SEPARATOR);
    let cargo_toml_path = Path::new(cargo_toml_path_str.as_str());

    // Creates the Cargo.toml in the specified directory if it doesn't exist.
    if !cargo_toml_path.exists() {
        info!("Creating {}", cargo_toml_path.display());
        let output = Command::new("cargo")
            .args(&["init", path, "--lib"])
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to run cargo");
        info!("{}", from_utf8(&output.stdout).unwrap());
    }

    info!("Reading TOML file.");

    let mut manifest = Manifest::from_path(cargo_toml_path).unwrap();
    

    // Add the PyO3 dependency to the Cargo.toml
    let default_dependency_block = Dependency::Detailed(DependencyDetail {
        version: Some(String::from("0.13")),
        registry: None,
        registry_index: None,
        path: None,
        git: None,
        branch: None,
        tag: None,
        rev: None,
        features: vec![String::from("extension-module")],
        optional: false,
        default_features: None,
        package: None,
    });

    // If PyO3 is already declared, we don't do anything to it, otherwise insert the default settings for it.
    info!("Adding pyo3 dependency");
    let _dep = manifest.dependencies.entry(String::from("pyo3")).or_insert(default_dependency_block);
    debug!("{:#?}", manifest);

    // Get the name of the current library path (default src/lib.rs).  We want to replace this with our Pythonized lib.
    let re = Regex::new(r"^(.*/)(.*)(.rs)$").unwrap();
    let lib_rs_path = manifest.lib.clone().unwrap().path.unwrap();
    let lib_path_parts = re.captures(&lib_rs_path).unwrap();
    let pylib_rs_path = format!("{}{}{}", &lib_path_parts[1], "pylib", &lib_path_parts[3]);   // Replace the name of the library file with pylib
    let rs_lib_mod = format!("{}", &lib_path_parts[2]);
    assert!(rs_lib_mod != "pylib");

    info!("Setting the library to start at {}", pylib_rs_path);
    // Change the path for the library.
    let mut new_lib = manifest.lib.clone().unwrap();
    new_lib.path = Some(pylib_rs_path.clone());

    // Set the library types.  cdylib allows it to be used with Python, and lib allows it to be used in Rust.
    new_lib.crate_type = vec![String::from("cdylib"), String::from("lib")];
    manifest.lib = Some(new_lib);

    let mut tt = TinyTemplate::new();
    tt.add_template("pylib.rs", PYLIB_RS_IN)?;
    tt.add_template("setup.py", SETUP_PY_IN)?;
    tt.add_template("MANIFEST.in", MANIFEST_IN)?;
    tt.add_template("__init__.py", INIT_PY)?;


    let package_name = manifest.package.clone().unwrap().name.clone();
    let module_name = package_name.replace("-", "_");

    let context = Context{ module_name: module_name.clone(), rs_lib_mod: rs_lib_mod, package_name: package_name };

    // Render the pylib.rs
    let pylib_rs = tt.render("pylib.rs", &context)?;
    let mut pylib_file =  File::create(format!("{}{}{}", path, MAIN_SEPARATOR, pylib_rs_path))?;
    pylib_file.write_all(pylib_rs.as_bytes())?;

    // Render the setup.py
    let setup_py = tt.render("setup.py", &context)?;
    let mut setup_py_file =  File::create(format!("{}{}{}", path, MAIN_SEPARATOR, "setup.py"))?;
    setup_py_file.write_all(setup_py.as_bytes())?;

    // Render the MANIFEST.in
    let manifest_in = tt.render("MANIFEST.in", &context)?;
    let mut manifest_file =  File::create(format!("{}{}{}", path, MAIN_SEPARATOR, "MANIFEST.in"))?;
    manifest_file.write_all(manifest_in.as_bytes())?;

    // Update the Cargo.toml
    let toml = cargo_toml::Value::try_from(manifest).unwrap();
    let toml_str = format!("{}", toml);
    let mut cargo_file =  File::create(format!("{}{}{}", path, MAIN_SEPARATOR, "Cargo.toml"))?;
    cargo_file.write_all(toml_str.as_bytes())?;

    // Create the Python module and directory.
    create_dir(format!("{}{}{}", path, MAIN_SEPARATOR, module_name.clone()))?;
    let mut init_file =  File::create(format!("{}{}{}{}{}", path, MAIN_SEPARATOR, module_name, MAIN_SEPARATOR, "__init__.py"))?;
    let init_str = tt.render("__init__.py", &context)?;
    init_file.write_all(init_str.as_bytes())?;
    
    Ok(())
}
