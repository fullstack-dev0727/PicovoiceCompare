
use leopard::LeopardBuilder;
use leopard::Leopard;

pub async fn fire() -> String {

    let access_key = "nhdZq87rVCiTpYkm2tIVPy6r71TC3Vrk+OkL+iAnhIu+svJzEismGQ==";
    let leopard: Leopard = LeopardBuilder::new(access_key).init().expect("Unable to create Leopard");
    if let Ok(transcript) = leopard.process_file("test.dat") {
        println!("{}", transcript);
        return String::from(transcript);
    } else {
        return "".to_string();
    }

}