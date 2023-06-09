use crds::{SftpgoAdmin, SftpgoFolder, SftpgoServer, SftpgoUser};
use kube::CustomResourceExt;
use std::fs::File;
use std::io::Write;

fn main() {
    let file_path = "charts/sftpgo-operator/templates/crds.yaml";

    let mut file = File::create(file_path).expect("Failed to create crd yaml file on disk");

    write_crd::<SftpgoServer>(&mut file);
    write_crd::<SftpgoUser>(&mut file);
    write_crd::<SftpgoFolder>(&mut file);
    write_crd::<SftpgoAdmin>(&mut file);
}

fn write_crd<TResource: CustomResourceExt>(mut file: &mut File) {
    let crd = TResource::crd();

    serde_yaml::to_writer(&mut file, &crd).expect("Failed to write CRD to disk");
    write!(file, "\n---\n").expect("Failed to write CRD to disk");
}
