include!("src/cli.rs");

fn main() {
    let outdir = match std::env::var_os("OUT_DIR") {
        Some(outdir) => outdir,
        None => return,
    };

    let mut app = get_app();
    app.gen_completions("oh", clap::Shell::Bash, outdir);
}
