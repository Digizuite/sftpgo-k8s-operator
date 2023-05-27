use crds::{SftpgoServer, SftpgoUser};
use kube::CustomResourceExt;

fn main() {
    write_crd::<SftpgoServer>();
    write_crd::<SftpgoUser>();
}

fn write_crd<TResource: CustomResourceExt>() {
    let crd = TResource::crd();

    let file_path = format!(
        "chart/templates/{}.yaml",
        crd.metadata.name.clone().unwrap()
    );

    let file = std::fs::File::create(file_path).expect("Failed to create crd yaml file on disk");

    serde_yaml::to_writer(file, &crd).expect("Failed to write CRD to disk");
}
