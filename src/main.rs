use pytv::Convert;

fn main() {
    let convert = Convert::from_args();
    println!("{:#?}", convert);
    convert
        .convert_to_file()
        .unwrap_or_else(|err| eprintln!("Error: {}", err));
}
