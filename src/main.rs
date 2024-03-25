use pytv::Convert;

fn main() {
    let convert = Convert::from_args();
    convert
        .convert_to_file()
        .unwrap_or_else(|err| eprintln!("Error: {}", err));
}
