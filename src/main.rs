fn main() {
    let code = rtw::run_cli_env().unwrap_or_else(|error| {
        eprintln!("rtw: {error:#}");
        2
    });
    std::process::exit(code)
}
