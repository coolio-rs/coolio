use std::env;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::fs::File;

fn main() {
  println!("cargo:rerun-if-changed=res/gtk-light.css");
  let theme = include_str!("res/gtk-light.css");
  let out_dir = env::var("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("../../../gtk-light.css");
  let mut f = File::create(&dest_path).unwrap();

  f.write_all(theme.as_bytes()).unwrap();

  println!("cargo:rerun-if-changed=src/icons.gresource");
  let status = Command::new("glib-compile-resources")
    .arg("src/icons.gresource")
    .arg("--target=src/icons.bin")
    .spawn()
    .expect("Failed running glib-compile-resources")
    .wait()
    .unwrap();
  assert!(status.success());
}
